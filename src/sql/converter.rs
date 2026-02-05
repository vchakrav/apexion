//! SOQL to SQL converter

use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Expression, ForClause, OrderByField, SelectField, SoqlQuery, SoqlWithClause,
    TypeOfClause,
};

use super::date_literals::{expand_date_literal, is_date_literal};
use super::dialect::{get_dialect, SqlDialect, SqlDialectImpl};
use super::error::{ConversionError, ConversionResult, ConversionWarning};
use super::schema::SalesforceSchema;

/// Result of SOQL to SQL conversion
#[derive(Debug, Clone)]
pub struct SqlConversion {
    /// The generated SQL query
    pub sql: String,
    /// Bind parameters (for parameterized queries)
    pub parameters: Vec<SqlParameter>,
    /// Column aliases mapping SOQL field paths to result columns
    pub column_map: HashMap<String, String>,
    /// Any warnings during conversion
    pub warnings: Vec<ConversionWarning>,
    /// Security mode from WITH clause (if any)
    pub security_mode: Option<SecurityMode>,
}

/// A bind parameter in the generated SQL
#[derive(Debug, Clone, PartialEq)]
pub struct SqlParameter {
    /// Parameter name in SQL (e.g., "p1")
    pub name: String,
    /// Placeholder in SQL (e.g., "$1" for Postgres, "?1" for SQLite)
    pub placeholder: String,
    /// Original Apex variable name
    pub original_name: String,
}

/// Security mode from SOQL WITH clause
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    SecurityEnforced,
    UserMode,
    SystemMode,
}

/// How to handle bind variables in generated SQL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BindVariableMode {
    /// Replace :var with $1, $2 (Postgres) or ?1, ?2 (SQLite)
    #[default]
    Parameterized,
    /// Replace :var with a placeholder string for debugging
    Placeholder,
}

/// Configuration for SOQL to SQL conversion
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Target SQL dialect
    pub dialect: SqlDialect,
    /// How to handle bind variables
    pub bind_mode: BindVariableMode,
    /// Whether to include soft-delete filter (WHERE is_deleted = false)
    pub filter_deleted: bool,
    /// Maximum query depth for relationship traversal
    pub max_relationship_depth: u8,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            dialect: SqlDialect::Postgres,
            bind_mode: BindVariableMode::Parameterized,
            filter_deleted: false,
            max_relationship_depth: 5,
        }
    }
}

/// Main SOQL to SQL converter
pub struct SoqlToSqlConverter<'a> {
    schema: Option<&'a SalesforceSchema>,
    dialect: Box<dyn SqlDialectImpl>,
    config: ConversionConfig,
    /// Current FROM object context
    current_object: Option<String>,
    /// Table alias counter for joins
    alias_counter: u32,
    /// Collected parameters
    parameters: Vec<SqlParameter>,
    /// Collected warnings
    warnings: Vec<ConversionWarning>,
    /// Collected JOINs for relationship traversal
    joins: Vec<JoinClause>,
    /// Column aliases for SELECT
    column_map: HashMap<String, String>,
    /// Table aliases for objects
    table_aliases: HashMap<String, String>,
}

/// A JOIN clause to be added to the query
#[derive(Debug, Clone)]
struct JoinClause {
    join_type: &'static str,
    table: String,
    alias: String,
    condition: String,
}

impl<'a> SoqlToSqlConverter<'a> {
    /// Create a new converter with schema
    pub fn new(schema: &'a SalesforceSchema, config: ConversionConfig) -> Self {
        let dialect = get_dialect(config.dialect);
        Self {
            schema: Some(schema),
            dialect,
            config,
            current_object: None,
            alias_counter: 0,
            parameters: Vec::new(),
            warnings: Vec::new(),
            joins: Vec::new(),
            column_map: HashMap::new(),
            table_aliases: HashMap::new(),
        }
    }

    /// Create a converter without schema (for simple queries)
    pub fn new_without_schema(config: ConversionConfig) -> Self {
        let dialect = get_dialect(config.dialect);
        Self {
            schema: None,
            dialect,
            config,
            current_object: None,
            alias_counter: 0,
            parameters: Vec::new(),
            warnings: Vec::new(),
            joins: Vec::new(),
            column_map: HashMap::new(),
            table_aliases: HashMap::new(),
        }
    }

    /// Convert a SOQL query to SQL
    pub fn convert(&mut self, query: &SoqlQuery) -> ConversionResult<SqlConversion> {
        // Reset state
        self.parameters.clear();
        self.warnings.clear();
        self.joins.clear();
        self.column_map.clear();
        self.table_aliases.clear();
        self.alias_counter = 0;

        // Set current object context
        self.current_object = Some(query.from_clause.clone());

        // Build query parts - FROM first to establish main table alias
        let from_sql = self.convert_from_clause(&query.from_clause)?;
        let select_sql = self.convert_select_clause(&query.select_clause)?;

        // Handle WITH clause (security)
        let security_mode = query.with_clause.map(|w| {
            let mode = match w {
                SoqlWithClause::SecurityEnforced => {
                    self.warnings.push(ConversionWarning::SecurityClauseRemoved(
                        "SECURITY_ENFORCED".to_string(),
                    ));
                    SecurityMode::SecurityEnforced
                }
                SoqlWithClause::UserMode => {
                    self.warnings.push(ConversionWarning::SecurityClauseRemoved(
                        "USER_MODE".to_string(),
                    ));
                    SecurityMode::UserMode
                }
                SoqlWithClause::SystemMode => {
                    self.warnings.push(ConversionWarning::SecurityClauseRemoved(
                        "SYSTEM_MODE".to_string(),
                    ));
                    SecurityMode::SystemMode
                }
            };
            mode
        });

        // Build WHERE clause
        let where_sql = if let Some(ref where_expr) = query.where_clause {
            Some(self.convert_expression(where_expr)?)
        } else {
            None
        };

        // Add soft-delete filter if configured
        let where_sql = if self.config.filter_deleted {
            let main_alias = self.get_table_alias(&query.from_clause);
            let delete_filter = format!(
                "{}.is_deleted = {}",
                main_alias,
                self.dialect.boolean_literal(false)
            );
            match where_sql {
                Some(w) => Some(format!("({}) AND {}", w, delete_filter)),
                None => Some(delete_filter),
            }
        } else {
            where_sql
        };

        // GROUP BY
        let group_by_sql = if !query.group_by_clause.is_empty() {
            Some(self.convert_group_by(&query.group_by_clause)?)
        } else {
            None
        };

        // HAVING
        let having_sql = if let Some(ref having_expr) = query.having_clause {
            Some(self.convert_expression(having_expr)?)
        } else {
            None
        };

        // ORDER BY
        let order_by_sql = if !query.order_by_clause.is_empty() {
            Some(self.convert_order_by(&query.order_by_clause)?)
        } else {
            None
        };

        // LIMIT
        let limit_sql = if let Some(ref limit_expr) = query.limit_clause {
            Some(self.convert_expression(limit_expr)?)
        } else {
            None
        };

        // OFFSET
        let offset_sql = if let Some(ref offset_expr) = query.offset_clause {
            Some(self.convert_expression(offset_expr)?)
        } else {
            None
        };

        // FOR clause
        let for_sql = self.convert_for_clause(&query.for_clause)?;

        // Build final SQL
        let mut sql = format!("SELECT {}\nFROM {}", select_sql, from_sql);

        // Add JOINs
        for join in &self.joins {
            sql.push_str(&format!(
                "\n{} {} {} ON {}",
                join.join_type, join.table, join.alias, join.condition
            ));
        }

        if let Some(w) = where_sql {
            sql.push_str(&format!("\nWHERE {}", w));
        }
        if let Some(g) = group_by_sql {
            sql.push_str(&format!("\nGROUP BY {}", g));
        }
        if let Some(h) = having_sql {
            sql.push_str(&format!("\nHAVING {}", h));
        }
        if let Some(o) = order_by_sql {
            sql.push_str(&format!("\nORDER BY {}", o));
        }
        if limit_sql.is_some() || offset_sql.is_some() {
            let lo = self
                .dialect
                .limit_offset(limit_sql.as_deref(), offset_sql.as_deref());
            if !lo.is_empty() {
                sql.push_str(&format!("\n{}", lo));
            }
        }
        if let Some(f) = for_sql {
            sql.push_str(&format!("\n{}", f));
        }

        Ok(SqlConversion {
            sql,
            parameters: std::mem::take(&mut self.parameters),
            column_map: std::mem::take(&mut self.column_map),
            warnings: std::mem::take(&mut self.warnings),
            security_mode,
        })
    }

    /// Convert SELECT clause
    fn convert_select_clause(&mut self, fields: &[SelectField]) -> ConversionResult<String> {
        let mut columns = Vec::new();

        for field in fields {
            match field {
                SelectField::Field(path) => {
                    let (sql, alias) = self.convert_field_path(path)?;
                    if &alias != path {
                        columns.push(format!(
                            "{} AS {}",
                            sql,
                            self.dialect.quote_identifier(&alias)
                        ));
                    } else {
                        columns.push(sql);
                    }
                    self.column_map.insert(path.clone(), alias);
                }
                SelectField::AggregateFunction { name, field, alias } => {
                    // Handle COUNT() with no field or COUNT(*)
                    let agg_sql =
                        if name.to_uppercase() == "COUNT" && (field.is_empty() || field == "*") {
                            "COUNT(*)".to_string()
                        } else {
                            let (field_sql, _) = self.convert_field_path(field)?;
                            format!("{}({})", name.to_uppercase(), field_sql)
                        };
                    if let Some(a) = alias {
                        columns.push(format!(
                            "{} AS {}",
                            agg_sql,
                            self.dialect.quote_identifier(a)
                        ));
                        self.column_map.insert(a.clone(), a.clone());
                    } else {
                        columns.push(agg_sql);
                    }
                }
                SelectField::SubQuery(subquery) => {
                    let subquery_sql = self.convert_subquery(subquery)?;
                    columns.push(subquery_sql);
                }
                SelectField::TypeOf(typeof_clause) => {
                    let typeof_sql = self.convert_typeof(typeof_clause)?;
                    columns.push(typeof_sql);
                }
            }
        }

        Ok(columns.join(", "))
    }

    /// Convert a field path (e.g., "Id", "Account.Name", "Account.Owner.Name")
    fn convert_field_path(&mut self, path: &str) -> ConversionResult<(String, String)> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.len() == 1 {
            // Simple field
            let main_alias = self.get_table_alias(self.current_object.as_ref().unwrap());
            let column = self.get_column_name(self.current_object.as_ref().unwrap(), parts[0])?;
            return Ok((format!("{}.{}", main_alias, column), parts[0].to_string()));
        }

        // Relationship traversal - need schema
        if self.schema.is_none() {
            return Err(ConversionError::SchemaRequired(format!(
                "relationship traversal: {}",
                path
            )));
        }

        // Navigate through relationships
        let mut current_obj = self.current_object.clone().unwrap();
        let mut current_alias = self.get_table_alias(&current_obj);

        for (i, part) in parts[..parts.len() - 1].iter().enumerate() {
            // Find the relationship field
            let (ref_object, join_field) = self.resolve_relationship(&current_obj, part)?;

            // Check if we already have a join for this relationship
            let join_alias = self.get_or_create_join(&current_alias, &ref_object, &join_field)?;

            current_obj = ref_object;
            current_alias = join_alias;

            // Check depth
            if i as u8 >= self.config.max_relationship_depth {
                return Err(ConversionError::RelationshipDepthExceeded {
                    max: self.config.max_relationship_depth,
                    actual: i as u8 + 1,
                });
            }
        }

        // Get the final field
        let final_field = parts.last().unwrap();
        let column = self.get_column_name(&current_obj, final_field)?;

        Ok((format!("{}.{}", current_alias, column), path.to_string()))
    }

    /// Resolve a relationship name to the referenced object and join field
    fn resolve_relationship(
        &self,
        from_object: &str,
        relationship_name: &str,
    ) -> ConversionResult<(String, String)> {
        let schema = self.schema.ok_or_else(|| {
            ConversionError::SchemaRequired(format!("relationship: {}", relationship_name))
        })?;

        let obj = schema
            .get_object(from_object)
            .ok_or_else(|| ConversionError::UnknownObject(from_object.to_string()))?;

        // Look for a field with this relationship name
        for field in obj.fields() {
            if let Some(ref rel_name) = field.relationship_name {
                if rel_name.eq_ignore_ascii_case(relationship_name) {
                    if let Some(ref refs) = field.reference_to {
                        if !refs.is_empty() {
                            return Ok((refs[0].clone(), field.column_name.clone()));
                        }
                    }
                }
            }
        }

        Err(ConversionError::NotARelationship(
            relationship_name.to_string(),
        ))
    }

    /// Get or create a JOIN for a relationship
    /// from_alias: the alias of the current table (e.g., "t0" for contact)
    /// to_object: the object being joined to (e.g., "Account")
    /// join_field: the FK field on the from table (e.g., "account_id")
    fn get_or_create_join(
        &mut self,
        from_alias: &str,
        to_object: &str,
        join_field: &str,
    ) -> ConversionResult<String> {
        let schema = self
            .schema
            .ok_or_else(|| ConversionError::SchemaRequired("join creation".to_string()))?;

        let to_obj = schema
            .get_object(to_object)
            .ok_or_else(|| ConversionError::UnknownObject(to_object.to_string()))?;

        // Check if we already have this join
        let join_key = format!("{}.{}", from_alias, join_field);
        if let Some(alias) = self.table_aliases.get(&join_key) {
            return Ok(alias.clone());
        }

        // Create new join
        let alias = self.next_alias();
        let table = self.dialect.quote_identifier(&to_obj.table_name);

        // JOIN condition: from_table.fk_field = to_table.id
        self.joins.push(JoinClause {
            join_type: "LEFT JOIN",
            table,
            alias: alias.clone(),
            condition: format!("{}.{} = {}.id", from_alias, join_field, alias),
        });

        self.table_aliases.insert(join_key, alias.clone());
        Ok(alias)
    }

    /// Convert FROM clause
    fn convert_from_clause(&mut self, object_name: &str) -> ConversionResult<String> {
        let table_name = if let Some(schema) = self.schema {
            if let Some(obj) = schema.get_object(object_name) {
                obj.table_name.clone()
            } else {
                // If not in schema, use snake_case conversion
                to_snake_case(object_name)
            }
        } else {
            to_snake_case(object_name)
        };

        let alias = self.next_alias();
        self.table_aliases
            .insert(object_name.to_lowercase(), alias.clone());

        Ok(format!(
            "{} {}",
            self.dialect.quote_identifier(&table_name),
            alias
        ))
    }

    /// Convert an expression
    fn convert_expression(&mut self, expr: &Expression) -> ConversionResult<String> {
        match expr {
            Expression::Null(_) => Ok("NULL".to_string()),
            Expression::Boolean(b, _) => Ok(self.dialect.boolean_literal(*b).to_string()),
            Expression::Integer(i, _) => Ok(i.to_string()),
            Expression::Long(l, _) => Ok(l.to_string()),
            Expression::Double(d, _) => Ok(d.to_string()),
            Expression::String(s, _) => {
                // Check if this is a date literal
                if is_date_literal(s) {
                    // This is used in WHERE context, we need a field expression
                    // Date literals are typically used like: WHERE CreatedDate = TODAY
                    // So we return the literal, and the binary expression handler
                    // will call expand_date_literal
                    Ok(format!("DATE_LITERAL:{}", s))
                } else {
                    // Regular string - escape single quotes
                    Ok(format!("'{}'", s.replace('\'', "''")))
                }
            }
            Expression::Identifier(name, _) => {
                // Check if it's a date literal
                if is_date_literal(name) {
                    Ok(format!("DATE_LITERAL:{}", name))
                } else {
                    // It's a field reference
                    let (sql, _) = self.convert_field_path(name)?;
                    Ok(sql)
                }
            }
            Expression::BindVariable(name, _) => self.add_parameter(name),
            Expression::Binary(binary) => {
                self.convert_binary_expression(&binary.left, binary.operator, &binary.right)
            }
            Expression::Unary(unary) => {
                let operand = self.convert_expression(&unary.operand)?;
                match unary.operator {
                    crate::ast::UnaryOp::Not => Ok(format!("NOT ({})", operand)),
                    crate::ast::UnaryOp::Negate => Ok(format!("-({})", operand)),
                    crate::ast::UnaryOp::BitwiseNot => Ok(format!("~({})", operand)),
                }
            }
            Expression::Parenthesized(inner, _) => {
                let inner_sql = self.convert_expression(inner)?;
                Ok(format!("({})", inner_sql))
            }
            Expression::ListLiteral(items, _) | Expression::SetLiteral(items, _) => {
                let converted: Result<Vec<_>, _> =
                    items.iter().map(|e| self.convert_expression(e)).collect();
                Ok(format!("({})", converted?.join(", ")))
            }
            Expression::NewArray(new_array) => {
                // Handle array initializer as a list (used in IN clauses)
                if let Some(ref items) = new_array.initializer {
                    let converted: Result<Vec<_>, _> =
                        items.iter().map(|e| self.convert_expression(e)).collect();
                    Ok(format!("({})", converted?.join(", ")))
                } else if let Some(ref size) = new_array.size {
                    // Array with size - convert the size expression
                    self.convert_expression(size)
                } else {
                    Ok("()".to_string())
                }
            }
            Expression::FieldAccess(fa) => {
                // Convert to dotted path
                let obj = self.convert_expression(&fa.object)?;
                Ok(format!("{}.{}", obj, to_snake_case(&fa.field)))
            }
            _ => Err(ConversionError::InvalidExpression(format!(
                "Unsupported expression type in SOQL: {:?}",
                std::mem::discriminant(expr)
            ))),
        }
    }

    /// Convert a binary expression
    fn convert_binary_expression(
        &mut self,
        left: &Expression,
        op: BinaryOp,
        right: &Expression,
    ) -> ConversionResult<String> {
        // Check for date literal on the right side
        let right_str = self.convert_expression(right)?;

        if let Some(date_literal) = right_str.strip_prefix("DATE_LITERAL:") {
            // This is a date literal comparison
            let left_str = self.convert_expression(left)?;
            return expand_date_literal(date_literal, &left_str, self.dialect.as_ref());
        }

        let left_str = self.convert_expression(left)?;

        let sql_op = match op {
            BinaryOp::Equal => "=",
            BinaryOp::NotEqual => "!=",
            BinaryOp::ExactEqual => "=",
            BinaryOp::ExactNotEqual => "!=",
            BinaryOp::LessThan => "<",
            BinaryOp::GreaterThan => ">",
            BinaryOp::LessOrEqual => "<=",
            BinaryOp::GreaterOrEqual => ">=",
            BinaryOp::And => "AND",
            BinaryOp::Or => "OR",
            BinaryOp::Like => "LIKE",
            BinaryOp::In => "IN",
            BinaryOp::NotIn => "NOT IN",
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Includes => {
                return self.convert_includes_excludes(&left_str, &right_str, true);
            }
            BinaryOp::Excludes => {
                return self.convert_includes_excludes(&left_str, &right_str, false);
            }
            _ => {
                return Err(ConversionError::UnsupportedSoqlFeature(format!(
                    "Operator {:?}",
                    op
                )));
            }
        };

        Ok(format!("{} {} {}", left_str, sql_op, right_str))
    }

    /// Convert INCLUDES/EXCLUDES for multi-picklist
    fn convert_includes_excludes(
        &self,
        field: &str,
        values: &str,
        is_includes: bool,
    ) -> ConversionResult<String> {
        // Multi-select picklists are stored as semicolon-separated values
        // INCLUDES ('A', 'B') means the field contains A AND B
        // We need to check if each value is present

        // Parse the value list (it's in SQL format: ('A', 'B'))
        let values_str = values.trim();
        let values_str = values_str.strip_prefix('(').unwrap_or(values_str);
        let values_str = values_str.strip_suffix(')').unwrap_or(values_str);

        let values: Vec<&str> = values_str
            .split(',')
            .map(|s| s.trim().trim_matches('\''))
            .collect();

        let conditions: Vec<String> = values
            .iter()
            .map(|v| {
                // Check if the value is at start, middle, or end of the semicolon-separated list
                format!(
                    "({} = '{}' OR {} LIKE '{}' OR {} LIKE '{}' OR {} LIKE '{}')",
                    field,
                    v,
                    field,
                    format!("{};%", v),
                    field,
                    format!("%;{}", v),
                    field,
                    format!("%;{};%", v)
                )
            })
            .collect();

        let joined = conditions.join(if is_includes { " AND " } else { " OR " });

        if is_includes {
            Ok(format!("({})", joined))
        } else {
            Ok(format!("NOT ({})", joined))
        }
    }

    /// Convert GROUP BY clause
    fn convert_group_by(&mut self, fields: &[String]) -> ConversionResult<String> {
        let converted: Result<Vec<_>, _> = fields
            .iter()
            .map(|f| self.convert_field_path(f).map(|(sql, _)| sql))
            .collect();
        Ok(converted?.join(", "))
    }

    /// Convert ORDER BY clause
    fn convert_order_by(&mut self, fields: &[OrderByField]) -> ConversionResult<String> {
        let converted: Result<Vec<_>, _> = fields
            .iter()
            .map(|f| {
                let (field_sql, _) = self.convert_field_path(&f.field)?;
                let mut sql = field_sql;
                if !f.ascending {
                    sql.push_str(" DESC");
                }
                if let Some(nulls_first) = f.nulls_first {
                    sql.push(' ');
                    sql.push_str(if nulls_first {
                        self.dialect.nulls_first()
                    } else {
                        self.dialect.nulls_last()
                    });
                }
                Ok(sql)
            })
            .collect();
        Ok(converted?.join(", "))
    }

    /// Convert FOR clause
    fn convert_for_clause(
        &mut self,
        for_clause: &Option<ForClause>,
    ) -> ConversionResult<Option<String>> {
        match for_clause {
            None => Ok(None),
            Some(ForClause::Update) => {
                if let Some(for_update) = self.dialect.for_update() {
                    Ok(Some(for_update.to_string()))
                } else {
                    self.warnings.push(ConversionWarning::ForUpdateNotSupported);
                    Ok(None)
                }
            }
            Some(ForClause::View) => {
                self.warnings.push(ConversionWarning::SalesforceOnlyClause(
                    "FOR VIEW".to_string(),
                ));
                Ok(None)
            }
            Some(ForClause::Reference) => {
                self.warnings.push(ConversionWarning::SalesforceOnlyClause(
                    "FOR REFERENCE".to_string(),
                ));
                Ok(None)
            }
        }
    }

    /// Convert a child relationship subquery
    fn convert_subquery(&mut self, subquery: &SoqlQuery) -> ConversionResult<String> {
        let schema = self
            .schema
            .ok_or_else(|| ConversionError::SchemaRequired("subquery".to_string()))?;

        let parent_obj = self.current_object.as_ref().unwrap();
        let parent_alias = self.get_table_alias(parent_obj);

        // Find the child relationship
        let obj = schema
            .get_object(parent_obj)
            .ok_or_else(|| ConversionError::UnknownObject(parent_obj.to_string()))?;

        let child_rel = obj
            .get_child_relationship(&subquery.from_clause)
            .ok_or_else(|| {
                ConversionError::UnknownChildRelationship(
                    subquery.from_clause.clone(),
                    parent_obj.clone(),
                )
            })?;

        let child_object = &child_rel.child_object;
        let child_field = &child_rel.field;

        // Get child table info
        let child_obj = schema
            .get_object(child_object)
            .ok_or_else(|| ConversionError::UnknownObject(child_object.to_string()))?;

        let child_table = &child_obj.table_name;
        let child_alias = self.next_alias();

        // Build subquery SELECT fields as JSON object
        let field_pairs: Vec<(String, String)> = subquery
            .select_clause
            .iter()
            .filter_map(|sf| match sf {
                SelectField::Field(f) => {
                    let col = self.get_column_name(child_object, f).ok()?;
                    Some((f.clone(), format!("{}.{}", child_alias, col)))
                }
                _ => None, // Skip complex fields in subquery for now
            })
            .collect();

        let json_obj = self.dialect.json_object(&field_pairs);
        let json_agg = self.dialect.json_array_agg(&json_obj);

        // Build correlated subquery
        let mut subquery_sql = format!(
            "(SELECT {} FROM {} {} WHERE {}.{} = {}.id",
            json_agg,
            self.dialect.quote_identifier(child_table),
            child_alias,
            child_alias,
            to_snake_case(child_field),
            parent_alias
        );

        // Add WHERE clause if present
        if let Some(ref where_expr) = subquery.where_clause {
            // Save and swap context
            let old_obj = self.current_object.take();
            let old_aliases = std::mem::take(&mut self.table_aliases);

            self.current_object = Some(child_object.clone());
            self.table_aliases
                .insert(child_object.to_lowercase(), child_alias.clone());

            let where_sql = self.convert_expression(where_expr)?;
            subquery_sql.push_str(&format!(" AND {}", where_sql));

            // Restore context
            self.current_object = old_obj;
            self.table_aliases = old_aliases;
        }

        // Add ORDER BY if present
        if !subquery.order_by_clause.is_empty() {
            // Save context
            let old_obj = self.current_object.take();
            let old_aliases = std::mem::take(&mut self.table_aliases);

            self.current_object = Some(child_object.clone());
            self.table_aliases
                .insert(child_object.to_lowercase(), child_alias.clone());

            let order_sql = self.convert_order_by(&subquery.order_by_clause)?;
            subquery_sql.push_str(&format!(" ORDER BY {}", order_sql));

            self.current_object = old_obj;
            self.table_aliases = old_aliases;
        }

        // Add LIMIT if present
        if let Some(ref limit_expr) = subquery.limit_clause {
            let limit_sql = self.convert_expression(limit_expr)?;
            subquery_sql.push_str(&format!(" LIMIT {}", limit_sql));
        }

        subquery_sql.push(')');
        subquery_sql.push_str(&format!(
            " AS {}",
            self.dialect.quote_identifier(&subquery.from_clause)
        ));

        Ok(subquery_sql)
    }

    /// Convert TYPEOF clause for polymorphic fields
    fn convert_typeof(&mut self, typeof_clause: &TypeOfClause) -> ConversionResult<String> {
        let schema = self
            .schema
            .ok_or_else(|| ConversionError::SchemaRequired("TYPEOF".to_string()))?;

        let parent_obj = self.current_object.as_ref().unwrap();
        let parent_alias = self.get_table_alias(parent_obj);

        let obj = schema
            .get_object(parent_obj)
            .ok_or_else(|| ConversionError::UnknownObject(parent_obj.to_string()))?;

        let field =
            obj.get_field(&typeof_clause.field)
                .ok_or_else(|| ConversionError::UnknownField {
                    object: parent_obj.to_string(),
                    field: typeof_clause.field.clone(),
                })?;

        if !field.is_polymorphic {
            return Err(ConversionError::NotPolymorphic(typeof_clause.field.clone()));
        }

        // Get the type discriminator column
        let type_column = format!(
            "{}.{}_type",
            parent_alias,
            to_snake_case(&typeof_clause.field)
        );
        let id_column = format!("{}.{}", parent_alias, field.column_name);

        // Build CASE expressions for each field across WHEN clauses
        let mut all_fields: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for when_clause in &typeof_clause.when_clauses {
            let type_name = &when_clause.type_name;
            let type_obj = schema
                .get_object(type_name)
                .ok_or_else(|| ConversionError::UnknownObject(type_name.to_string()))?;

            // Create join for this type
            let alias = self.next_alias();
            self.joins.push(JoinClause {
                join_type: "LEFT JOIN",
                table: self.dialect.quote_identifier(&type_obj.table_name),
                alias: alias.clone(),
                condition: format!(
                    "{} = {}.id AND {} = '{}'",
                    id_column, alias, type_column, type_name
                ),
            });

            for field_name in &when_clause.fields {
                let col = self.get_column_name(type_name, field_name)?;
                all_fields
                    .entry(field_name.clone())
                    .or_default()
                    .push((type_name.clone(), format!("{}.{}", alias, col)));
            }
        }

        // Build CASE expressions
        let mut case_exprs = Vec::new();
        for (field_name, type_cols) in all_fields {
            let mut case = format!("CASE {}", type_column);
            for (type_name, col_expr) in &type_cols {
                case.push_str(&format!(" WHEN '{}' THEN {}", type_name, col_expr));
            }
            if let Some(ref else_fields) = typeof_clause.else_fields {
                if else_fields.contains(&field_name) {
                    // Use COALESCE for ELSE
                    let coalesce_cols: Vec<_> = type_cols.iter().map(|(_, c)| c.as_str()).collect();
                    case.push_str(&format!(" ELSE COALESCE({})", coalesce_cols.join(", ")));
                }
            }
            case.push_str(" END");
            case.push_str(&format!(
                " AS {}",
                self.dialect
                    .quote_identifier(&format!("{}.{}", typeof_clause.field, field_name))
            ));
            case_exprs.push(case);
        }

        Ok(case_exprs.join(", "))
    }

    /// Add a parameter and return its placeholder
    fn add_parameter(&mut self, name: &str) -> ConversionResult<String> {
        let index = self.parameters.len() + 1;
        let placeholder = match self.config.bind_mode {
            BindVariableMode::Parameterized => self.dialect.parameter_placeholder(index),
            BindVariableMode::Placeholder => format!("::{}", name),
        };

        self.parameters.push(SqlParameter {
            name: format!("p{}", index),
            placeholder: placeholder.clone(),
            original_name: name.to_string(),
        });

        Ok(placeholder)
    }

    /// Get the table alias for an object
    fn get_table_alias(&self, object_name: &str) -> String {
        self.table_aliases
            .get(&object_name.to_lowercase())
            .cloned()
            .unwrap_or_else(|| format!("t{}", self.alias_counter))
    }

    /// Generate next table alias
    fn next_alias(&mut self) -> String {
        let alias = format!("t{}", self.alias_counter);
        self.alias_counter += 1;
        alias
    }

    /// Get the SQL column name for a field
    fn get_column_name(&self, object_name: &str, field_name: &str) -> ConversionResult<String> {
        if let Some(schema) = self.schema {
            if let Some(obj) = schema.get_object(object_name) {
                if let Some(field) = obj.get_field(field_name) {
                    return Ok(field.column_name.clone());
                }
            }
        }
        // Fall back to snake_case conversion
        Ok(to_snake_case(field_name))
    }
}

/// Convert a Salesforce API name to snake_case for SQL
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut chars = s.chars().peekable();
    let mut prev_was_upper = false;
    let mut prev_was_underscore = true;

    while let Some(c) = chars.next() {
        if c == '_' {
            result.push('_');
            prev_was_underscore = true;
            prev_was_upper = false;
        } else if c.is_uppercase() {
            if !prev_was_underscore {
                let next_is_lower = chars.peek().map(|c| c.is_lowercase()).unwrap_or(false);
                if !prev_was_upper || next_is_lower {
                    result.push('_');
                }
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_was_upper = true;
            prev_was_underscore = false;
        } else {
            result.push(c.to_lowercase().next().unwrap());
            prev_was_upper = false;
            prev_was_underscore = false;
        }
    }

    result
}

/// Convenience function for simple conversions
pub fn convert_soql(
    query: &SoqlQuery,
    schema: &SalesforceSchema,
    config: ConversionConfig,
) -> ConversionResult<SqlConversion> {
    let mut converter = SoqlToSqlConverter::new(schema, config);
    converter.convert(query)
}

/// Convert SOQL without schema (only works for simple single-object queries)
pub fn convert_soql_simple(
    query: &SoqlQuery,
    dialect: SqlDialect,
) -> ConversionResult<SqlConversion> {
    let config = ConversionConfig {
        dialect,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new_without_schema(config);
    converter.convert(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn extract_soql(source: &str) -> SoqlQuery {
        let full_source = format!(
            "class Test {{ void test() {{ List<SObject> x = [{}]; }} }}",
            source
        );
        let cu = parse(&full_source).expect("Parse failed");
        if let crate::ast::TypeDeclaration::Class(class) = &cu.declarations[0] {
            if let crate::ast::ClassMember::Method(method) = &class.members[0] {
                if let Some(block) = &method.body {
                    if let crate::ast::Statement::LocalVariable(lv) = &block.statements[0] {
                        if let Some(Expression::Soql(soql)) = &lv.declarators[0].initializer {
                            return (**soql).clone();
                        }
                    }
                }
            }
        }
        panic!("Could not extract SOQL query");
    }

    #[test]
    fn test_simple_select() {
        let soql = extract_soql("SELECT Id, Name FROM Account");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("SELECT"));
        assert!(result.sql.contains("id"));
        assert!(result.sql.contains("name"));
        assert!(result.sql.contains("\"account\""));
    }

    #[test]
    fn test_where_clause() {
        let soql = extract_soql("SELECT Id FROM Account WHERE Name = 'Test'");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("WHERE"));
        assert!(result.sql.contains("name = 'Test'"));
    }

    #[test]
    fn test_bind_variable() {
        let soql = extract_soql("SELECT Id FROM Account WHERE Name = :accountName");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("$1"));
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0].original_name, "accountName");
    }

    #[test]
    fn test_sqlite_bind_variable() {
        let soql = extract_soql("SELECT Id FROM Account WHERE Name = :accountName");
        let result = convert_soql_simple(&soql, SqlDialect::Sqlite).unwrap();

        assert!(result.sql.contains("?1"));
    }

    #[test]
    fn test_order_by() {
        let soql = extract_soql("SELECT Id FROM Account ORDER BY Name DESC NULLS LAST");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("ORDER BY"));
        assert!(result.sql.contains("DESC"));
        assert!(result.sql.contains("NULLS LAST"));
    }

    #[test]
    fn test_limit_offset() {
        let soql = extract_soql("SELECT Id FROM Account LIMIT 10 OFFSET 5");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("LIMIT 10"));
        assert!(result.sql.contains("OFFSET 5"));
    }

    #[test]
    fn test_aggregate_function() {
        let soql = extract_soql("SELECT COUNT(Id) cnt FROM Account");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("COUNT("));
        assert!(result.sql.contains("AS \"cnt\""));
    }

    #[test]
    fn test_group_by() {
        let soql = extract_soql("SELECT Industry, COUNT(Id) FROM Account GROUP BY Industry");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("GROUP BY"));
        assert!(result.sql.contains("industry"));
    }

    #[test]
    fn test_for_update_postgres() {
        let soql = extract_soql("SELECT Id FROM Account FOR UPDATE");
        let result = convert_soql_simple(&soql, SqlDialect::Postgres).unwrap();

        assert!(result.sql.contains("FOR UPDATE"));
    }

    #[test]
    fn test_for_update_sqlite_warning() {
        let soql = extract_soql("SELECT Id FROM Account FOR UPDATE");
        let result = convert_soql_simple(&soql, SqlDialect::Sqlite).unwrap();

        assert!(!result.sql.contains("FOR UPDATE"));
        assert!(result
            .warnings
            .iter()
            .any(|w| matches!(w, ConversionWarning::ForUpdateNotSupported)));
    }
}
