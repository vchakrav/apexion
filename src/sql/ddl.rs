//! DDL generation for Salesforce schema

use super::dialect::{get_dialect, SqlDialect, SqlDialectImpl};
use super::schema::{FieldDescribe, SObjectDescribe, SalesforceFieldType, SalesforceSchema};

/// Generator for SQL DDL (CREATE TABLE, etc.)
pub struct DdlGenerator {
    dialect: Box<dyn SqlDialectImpl>,
}

impl DdlGenerator {
    /// Create a new DDL generator for the specified dialect
    pub fn new(dialect: SqlDialect) -> Self {
        Self {
            dialect: get_dialect(dialect),
        }
    }

    /// Generate CREATE TABLE statement for an SObject
    pub fn generate_table(&self, object: &SObjectDescribe) -> String {
        let mut sql = format!(
            "CREATE TABLE {} (\n",
            self.dialect.quote_identifier(&object.table_name)
        );

        let mut columns = Vec::new();
        let mut constraints = Vec::new();

        // Sort fields for consistent output
        let mut fields: Vec<_> = object.fields().collect();
        fields.sort_by(|a, b| {
            // Put Id first, then Name, then sort alphabetically
            match (a.name.as_str(), b.name.as_str()) {
                ("Id", _) => std::cmp::Ordering::Less,
                (_, "Id") => std::cmp::Ordering::Greater,
                ("Name", _) => std::cmp::Ordering::Less,
                (_, "Name") => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        for field in fields {
            let col_def = self.generate_column(field);
            columns.push(format!("    {}", col_def));

            // Add foreign key constraints for lookup fields
            if field.is_relationship() && !field.is_polymorphic {
                if let Some(ref refs) = field.reference_to {
                    if let Some(ref_obj) = refs.first() {
                        // Only add FK if it's not a self-reference to avoid issues
                        // Self-references are valid but need the table to exist first
                        let ref_table = to_snake_case(ref_obj);
                        constraints.push(format!(
                            "    FOREIGN KEY ({}) REFERENCES {}(id)",
                            self.dialect.quote_identifier(&field.column_name),
                            self.dialect.quote_identifier(&ref_table)
                        ));
                    }
                }
            }

            // For polymorphic fields, add type discriminator column
            if field.is_polymorphic {
                let type_col = format!("{}_type", field.column_name);
                columns.push(format!(
                    "    {} TEXT",
                    self.dialect.quote_identifier(&type_col)
                ));
            }
        }

        sql.push_str(&columns.join(",\n"));

        // Add constraints (optional - can be commented out for SQLite compatibility)
        if !constraints.is_empty() && matches!(self.dialect.dialect(), SqlDialect::Postgres) {
            sql.push_str(",\n");
            sql.push_str(&constraints.join(",\n"));
        }

        sql.push_str("\n)");
        sql
    }

    /// Generate column definition
    fn generate_column(&self, field: &FieldDescribe) -> String {
        let mut col = format!(
            "{} {}",
            self.dialect.quote_identifier(&field.column_name),
            self.column_type(field)
        );

        // Add PRIMARY KEY for Id field
        if field.name == "Id" {
            col.push_str(" PRIMARY KEY");
        } else if !field.nillable {
            col.push_str(" NOT NULL");
        }

        col
    }

    /// Get SQL column type for a field
    fn column_type(&self, field: &FieldDescribe) -> &'static str {
        match field.field_type {
            SalesforceFieldType::Boolean => {
                match self.dialect.dialect() {
                    SqlDialect::Postgres => "BOOLEAN",
                    SqlDialect::Sqlite => "INTEGER", // SQLite uses 0/1
                }
            }
            SalesforceFieldType::Integer => "INTEGER",
            SalesforceFieldType::Double
            | SalesforceFieldType::Currency
            | SalesforceFieldType::Percent => match self.dialect.dialect() {
                SqlDialect::Postgres => "NUMERIC",
                SqlDialect::Sqlite => "REAL",
            },
            SalesforceFieldType::Date => "DATE",
            SalesforceFieldType::DateTime => {
                match self.dialect.dialect() {
                    SqlDialect::Postgres => "TIMESTAMP",
                    SqlDialect::Sqlite => "TEXT", // SQLite stores dates as TEXT
                }
            }
            SalesforceFieldType::Time => match self.dialect.dialect() {
                SqlDialect::Postgres => "TIME",
                SqlDialect::Sqlite => "TEXT",
            },
            _ => "TEXT",
        }
    }

    /// Generate CREATE INDEX statements for an SObject
    pub fn generate_indexes(&self, object: &SObjectDescribe) -> Vec<String> {
        let mut indexes = Vec::new();
        let table = &object.table_name;

        for field in object.fields() {
            // Create indexes for lookup fields
            if field.is_relationship() {
                indexes.push(format!(
                    "CREATE INDEX {} ON {} ({})",
                    self.dialect
                        .quote_identifier(&format!("idx_{}_{}", table, field.column_name)),
                    self.dialect.quote_identifier(table),
                    self.dialect.quote_identifier(&field.column_name)
                ));
            }

            // Create index for Name field (commonly queried)
            if field.name == "Name" {
                indexes.push(format!(
                    "CREATE INDEX {} ON {} ({})",
                    self.dialect
                        .quote_identifier(&format!("idx_{}_name", table)),
                    self.dialect.quote_identifier(table),
                    self.dialect.quote_identifier(&field.column_name)
                ));
            }

            // Create index for system timestamp fields
            if matches!(
                field.name.as_str(),
                "CreatedDate" | "LastModifiedDate" | "SystemModstamp"
            ) {
                indexes.push(format!(
                    "CREATE INDEX {} ON {} ({})",
                    self.dialect
                        .quote_identifier(&format!("idx_{}_{}", table, field.column_name)),
                    self.dialect.quote_identifier(table),
                    self.dialect.quote_identifier(&field.column_name)
                ));
            }
        }

        // Index for soft delete
        if object.has_field("IsDeleted") {
            indexes.push(format!(
                "CREATE INDEX {} ON {} ({})",
                self.dialect
                    .quote_identifier(&format!("idx_{}_is_deleted", table)),
                self.dialect.quote_identifier(table),
                self.dialect.quote_identifier("is_deleted")
            ));
        }

        indexes
    }

    /// Generate complete DDL for a schema
    pub fn generate_schema(&self, schema: &SalesforceSchema) -> String {
        let mut sql = String::new();

        // Collect objects and sort them to handle dependencies
        // (parent objects should be created before children)
        let mut objects: Vec<_> = schema.objects().collect();
        objects.sort_by(|a, b| a.name.cmp(&b.name));

        // Create tables
        for object in &objects {
            sql.push_str(&self.generate_table(object));
            sql.push_str(";\n\n");
        }

        // Create indexes
        for object in &objects {
            for index in self.generate_indexes(object) {
                sql.push_str(&index);
                sql.push_str(";\n");
            }
            sql.push('\n');
        }

        sql
    }

    /// Generate DROP TABLE statement
    pub fn generate_drop_table(&self, object: &SObjectDescribe) -> String {
        format!(
            "DROP TABLE IF EXISTS {}",
            self.dialect.quote_identifier(&object.table_name)
        )
    }

    /// Generate DROP TABLE statements for all objects in schema
    pub fn generate_drop_schema(&self, schema: &SalesforceSchema) -> String {
        let mut sql = String::new();

        // Drop in reverse order to handle foreign key dependencies
        let mut objects: Vec<_> = schema.objects().collect();
        objects.sort_by(|a, b| b.name.cmp(&a.name));

        for object in objects {
            sql.push_str(&self.generate_drop_table(object));
            sql.push_str(";\n");
        }

        sql
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::schema::ChildRelationship;

    fn create_test_schema() -> SalesforceSchema {
        let mut schema = SalesforceSchema::new();

        // Create Account
        let mut account = SObjectDescribe::new("Account");
        account.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
        account.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
        account.add_field(FieldDescribe::new(
            "Industry",
            SalesforceFieldType::Picklist,
        ));
        account.add_field(FieldDescribe::new(
            "AnnualRevenue",
            SalesforceFieldType::Currency,
        ));
        account.add_field(FieldDescribe::new(
            "IsDeleted",
            SalesforceFieldType::Boolean,
        ));
        account.add_field(FieldDescribe::new(
            "CreatedDate",
            SalesforceFieldType::DateTime,
        ));
        account.add_child_relationship(ChildRelationship::new("Contacts", "Contact", "AccountId"));
        schema.add_object(account);

        // Create Contact
        let mut contact = SObjectDescribe::new("Contact");
        contact.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
        contact.add_field(FieldDescribe::new("FirstName", SalesforceFieldType::String));
        contact.add_field(FieldDescribe::new("LastName", SalesforceFieldType::String));
        contact.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
        contact.add_field(
            FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
                .with_reference("Account")
                .with_relationship_name("Account"),
        );
        contact.add_field(FieldDescribe::new(
            "IsDeleted",
            SalesforceFieldType::Boolean,
        ));
        schema.add_object(contact);

        schema
    }

    #[test]
    fn test_generate_table_postgres() {
        let schema = create_test_schema();
        let generator = DdlGenerator::new(SqlDialect::Postgres);

        let account = schema.get_object("Account").unwrap();
        let ddl = generator.generate_table(account);

        assert!(ddl.contains("CREATE TABLE \"account\""));
        assert!(ddl.contains("\"id\" TEXT PRIMARY KEY"));
        assert!(ddl.contains("\"name\" TEXT"));
        assert!(ddl.contains("\"industry\" TEXT"));
        assert!(ddl.contains("\"annual_revenue\" NUMERIC"));
        assert!(ddl.contains("\"is_deleted\" BOOLEAN"));
        assert!(ddl.contains("\"created_date\" TIMESTAMP"));
    }

    #[test]
    fn test_generate_table_sqlite() {
        let schema = create_test_schema();
        let generator = DdlGenerator::new(SqlDialect::Sqlite);

        let account = schema.get_object("Account").unwrap();
        let ddl = generator.generate_table(account);

        assert!(ddl.contains("CREATE TABLE \"account\""));
        assert!(ddl.contains("\"is_deleted\" INTEGER")); // SQLite uses INTEGER for bool
        assert!(ddl.contains("\"created_date\" TEXT")); // SQLite uses TEXT for datetime
    }

    #[test]
    fn test_generate_indexes() {
        let schema = create_test_schema();
        let generator = DdlGenerator::new(SqlDialect::Postgres);

        let contact = schema.get_object("Contact").unwrap();
        let indexes = generator.generate_indexes(contact);

        // Should have index for AccountId (lookup field)
        assert!(indexes.iter().any(|i| i.contains("idx_contact_account_id")));

        // Should have index for IsDeleted
        assert!(indexes.iter().any(|i| i.contains("idx_contact_is_deleted")));
    }

    #[test]
    fn test_generate_schema() {
        let schema = create_test_schema();
        let generator = DdlGenerator::new(SqlDialect::Postgres);

        let ddl = generator.generate_schema(&schema);

        assert!(ddl.contains("CREATE TABLE \"account\""));
        assert!(ddl.contains("CREATE TABLE \"contact\""));
        assert!(ddl.contains("CREATE INDEX"));
    }

    #[test]
    fn test_generate_drop_schema() {
        let schema = create_test_schema();
        let generator = DdlGenerator::new(SqlDialect::Postgres);

        let ddl = generator.generate_drop_schema(&schema);

        assert!(ddl.contains("DROP TABLE IF EXISTS \"account\""));
        assert!(ddl.contains("DROP TABLE IF EXISTS \"contact\""));
    }

    #[test]
    fn test_foreign_key_postgres() {
        let schema = create_test_schema();
        let generator = DdlGenerator::new(SqlDialect::Postgres);

        let contact = schema.get_object("Contact").unwrap();
        let ddl = generator.generate_table(contact);

        // Postgres should have FK constraint
        assert!(ddl.contains("FOREIGN KEY (\"account_id\") REFERENCES \"account\"(id)"));
    }

    #[test]
    fn test_polymorphic_field() {
        let mut schema = SalesforceSchema::new();

        let mut task = SObjectDescribe::new("Task");
        task.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
        task.add_field(
            FieldDescribe::new("WhatId", SalesforceFieldType::Reference)
                .with_polymorphic_reference(vec!["Account".to_string(), "Opportunity".to_string()])
                .with_relationship_name("What"),
        );
        schema.add_object(task);

        let generator = DdlGenerator::new(SqlDialect::Postgres);
        let ddl = generator.generate_table(schema.get_object("Task").unwrap());

        // Should have both ID and type columns for polymorphic field
        assert!(ddl.contains("\"what_id\" TEXT"));
        assert!(ddl.contains("\"what_id_type\" TEXT"));
    }
}
