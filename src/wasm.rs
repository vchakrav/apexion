//! WebAssembly bindings for ApexRust
//!
//! This module provides JavaScript-friendly APIs for parsing Apex code
//! and converting SOQL to SQL.
//!
//! # Usage from JavaScript
//!
//! ```javascript
//! import init, { parseApex, convertSoqlToSql, WasmSchema } from 'apexrust';
//!
//! await init();
//!
//! // Parse Apex code
//! const result = parseApex(`
//!   public class MyClass {
//!     public void myMethod() {
//!       List<Account> accounts = [SELECT Id, Name FROM Account];
//!     }
//!   }
//! `);
//!
//! if (result.success) {
//!   console.log(result.ast);
//!   console.log(result.soqlQueries); // Extracted SOQL queries
//! } else {
//!   console.error(result.error);
//! }
//!
//! // Convert SOQL to SQL
//! const schema = new WasmSchema();
//! schema.loadSalesCloud(); // Load standard objects
//!
//! const sql = convertSoqlToSql('SELECT Id, Name FROM Account', schema, 'sqlite');
//! console.log(sql.sql);
//! ```

use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Helper to serialize values as plain JS objects (not Maps)
fn to_js_value<T: Serialize>(value: &T) -> JsValue {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value.serialize(&serializer).unwrap_or(JsValue::NULL)
}

use crate::ast::{
    Block, ClassDeclaration, ClassMember, Expression, ForInit, InterfaceDeclaration,
    MethodDeclaration, PropertyDeclaration, SoqlQuery, Statement, TypeDeclaration,
};
use crate::parser;
use crate::sql::converter::{ConversionConfig, SoqlToSqlConverter};
use crate::sql::dialect::SqlDialect;
use crate::sql::schema::{
    ChildRelationship, FieldDescribe, SObjectDescribe, SalesforceFieldType, SalesforceSchema,
};
use crate::transpile::{TranspileOptions, Transpiler};

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Parse Apex source code and return JSON result
///
/// Returns a JSON object with:
/// - `success`: boolean
/// - `ast`: the parsed AST (if successful)
/// - `soqlQueries`: array of SOQL queries found in the code
/// - `error`: error message (if failed)
#[wasm_bindgen(js_name = parseApex)]
pub fn parse_apex(source: &str) -> JsValue {
    match parser::parse(source) {
        Ok(compilation_unit) => {
            // Extract SOQL queries from the AST
            let mut soql_queries = Vec::new();
            for decl in &compilation_unit.declarations {
                extract_soql_from_type_declaration(decl, &mut soql_queries);
            }

            to_js_value(&serde_json::json!({
                "success": true,
                "ast": format!("{:#?}", compilation_unit),
                "soqlQueries": soql_queries,
            }))
        }
        Err(e) => to_js_value(&serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

/// Parse a single SOQL query and return JSON result
#[wasm_bindgen(js_name = parseSoql)]
pub fn parse_soql(source: &str) -> JsValue {
    // Wrap the SOQL in a minimal Apex context to parse it
    let apex_wrapper = format!(
        "public class Q {{ public void q() {{ List<SObject> r = [{}]; }} }}",
        source
    );

    match parser::parse(&apex_wrapper) {
        Ok(unit) => {
            // Extract the SOQL query from the parsed AST
            let mut queries = Vec::new();
            for decl in &unit.declarations {
                extract_soql_from_type_declaration(decl, &mut queries);
            }

            if queries.is_empty() {
                return to_js_value(&serde_json::json!({
                    "success": false,
                    "error": "No SOQL query found",
                }));
            }

            to_js_value(&serde_json::json!({
                "success": true,
                "query": queries[0],
            }))
        }
        Err(e) => to_js_value(&serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    }
}

/// Salesforce schema for WASM
///
/// This is a JavaScript-friendly wrapper around the Rust schema types.
#[wasm_bindgen]
pub struct WasmSchema {
    inner: SalesforceSchema,
}

#[wasm_bindgen]
impl WasmSchema {
    /// Create a new empty schema
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSchema {
        WasmSchema {
            inner: SalesforceSchema::new(),
        }
    }

    /// Add an object to the schema from JSON
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "name": "Account",
    ///   "fields": [
    ///     { "name": "Id", "type": "Id" },
    ///     { "name": "Name", "type": "String" },
    ///     { "name": "OwnerId", "type": "Lookup", "referenceTo": "User", "relationshipName": "Owner" }
    ///   ],
    ///   "childRelationships": [
    ///     { "name": "Contacts", "childObject": "Contact", "field": "AccountId" }
    ///   ]
    /// }
    /// ```
    #[wasm_bindgen(js_name = addObject)]
    pub fn add_object(&mut self, object_json: JsValue) -> Result<(), JsValue> {
        let obj: serde_json::Value = serde_wasm_bindgen::from_value(object_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

        let name = obj["name"]
            .as_str()
            .ok_or_else(|| JsValue::from_str("Missing 'name' field"))?;

        let mut sobject = SObjectDescribe::new(name);

        // Parse fields
        if let Some(fields) = obj["fields"].as_array() {
            for field_json in fields {
                let field = parse_field_json(field_json)
                    .map_err(|e| JsValue::from_str(&format!("Invalid field: {}", e)))?;
                sobject.add_field(field);
            }
        }

        // Parse child relationships
        if let Some(relationships) = obj["childRelationships"].as_array() {
            for rel_json in relationships {
                let rel_name = rel_json["name"]
                    .as_str()
                    .ok_or_else(|| JsValue::from_str("Missing relationship 'name'"))?;
                let child_object = rel_json["childObject"]
                    .as_str()
                    .ok_or_else(|| JsValue::from_str("Missing 'childObject'"))?;
                let field = rel_json["field"]
                    .as_str()
                    .ok_or_else(|| JsValue::from_str("Missing 'field'"))?;

                sobject.add_child_relationship(ChildRelationship::new(
                    rel_name,
                    child_object,
                    field,
                ));
            }
        }

        self.inner.add_object(sobject);
        Ok(())
    }

    /// Load a complete schema from JSON
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "objects": [
    ///     { "name": "Account", "fields": [...] },
    ///     { "name": "Contact", "fields": [...] }
    ///   ]
    /// }
    /// ```
    #[wasm_bindgen(js_name = loadFromJson)]
    pub fn load_from_json(&mut self, schema_json: JsValue) -> Result<(), JsValue> {
        let schema: serde_json::Value = serde_wasm_bindgen::from_value(schema_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

        if let Some(objects) = schema["objects"].as_array() {
            for obj in objects {
                self.add_object(serde_wasm_bindgen::to_value(obj)?)?;
            }
        }

        Ok(())
    }

    /// Get the standard Sales Cloud schema
    #[wasm_bindgen(js_name = loadSalesCloud)]
    pub fn load_sales_cloud(&mut self) {
        self.inner = crate::sql::standard_objects::create_sales_cloud_schema();
    }

    /// Check if an object exists in the schema
    #[wasm_bindgen(js_name = hasObject)]
    pub fn has_object(&self, name: &str) -> bool {
        self.inner.has_object(name)
    }

    /// Get list of object names
    #[wasm_bindgen(js_name = getObjectNames)]
    pub fn get_object_names(&self) -> JsValue {
        let names: Vec<&str> = self.inner.objects().map(|o| o.name.as_str()).collect();
        to_js_value(&names)
    }
}

impl Default for WasmSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert SOQL to SQL
///
/// # Arguments
/// * `soql` - The SOQL query string
/// * `schema` - The Salesforce schema
/// * `dialect` - Either "sqlite" or "postgres"
///
/// # Returns
/// JSON object with:
/// - `success`: boolean
/// - `sql`: the converted SQL (if successful)
/// - `parameters`: array of parameter info objects
/// - `warnings`: array of warning messages
/// - `error`: error message (if failed)
#[wasm_bindgen(js_name = convertSoqlToSql)]
pub fn convert_soql_to_sql(soql: &str, schema: &WasmSchema, dialect: &str) -> JsValue {
    let sql_dialect = match dialect.to_lowercase().as_str() {
        "postgres" | "postgresql" => SqlDialect::Postgres,
        "sqlite" | "sqlite3" => SqlDialect::Sqlite,
        _ => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("Unknown dialect '{}'. Use 'sqlite' or 'postgres'.", dialect),
            });
            return to_js_value(&result);
        }
    };

    // Parse the SOQL by wrapping in Apex
    let apex_wrapper = format!(
        "public class Q {{ public void q() {{ List<SObject> r = [{}]; }} }}",
        soql
    );

    let query = match parser::parse(&apex_wrapper) {
        Ok(unit) => {
            // Extract the SOQL query
            let mut queries: Vec<&SoqlQuery> = Vec::new();
            for decl in &unit.declarations {
                extract_soql_refs_from_type_declaration(decl, &mut queries);
            }

            if queries.is_empty() {
                let result = serde_json::json!({
                    "success": false,
                    "error": "No SOQL query found in input",
                });
                return to_js_value(&result);
            }

            queries[0].clone()
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("SOQL parse error: {}", e),
            });
            return to_js_value(&result);
        }
    };

    // Convert to SQL
    let config = ConversionConfig {
        dialect: sql_dialect,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema.inner, config);

    match converter.convert(&query) {
        Ok(result) => {
            let warnings: Vec<String> = result.warnings.iter().map(|w| w.to_string()).collect();
            let params: Vec<serde_json::Value> = result
                .parameters
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "placeholder": p.placeholder,
                        "originalName": p.original_name,
                    })
                })
                .collect();

            let json_result = serde_json::json!({
                "success": true,
                "sql": result.sql,
                "parameters": params,
                "warnings": warnings,
            });
            to_js_value(&json_result)
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("Conversion error: {}", e),
            });
            to_js_value(&result)
        }
    }
}

/// Generate DDL for a schema
///
/// # Arguments
/// * `schema` - The Salesforce schema
/// * `dialect` - Either "sqlite" or "postgres"
///
/// # Returns
/// JSON object with:
/// - `success`: boolean
/// - `ddl`: the DDL statements (if successful)
/// - `error`: error message (if failed)
#[wasm_bindgen(js_name = generateDdl)]
pub fn generate_ddl(schema: &WasmSchema, dialect: &str) -> JsValue {
    let sql_dialect = match dialect.to_lowercase().as_str() {
        "postgres" | "postgresql" => SqlDialect::Postgres,
        "sqlite" | "sqlite3" => SqlDialect::Sqlite,
        _ => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("Unknown dialect '{}'. Use 'sqlite' or 'postgres'.", dialect),
            });
            return to_js_value(&result);
        }
    };

    let generator = crate::sql::ddl::DdlGenerator::new(sql_dialect);
    let ddl = generator.generate_schema(&schema.inner);

    let result = serde_json::json!({
        "success": true,
        "ddl": ddl,
    });
    to_js_value(&result)
}

// ============================================================================
// Apex to TypeScript transpilation
// ============================================================================

/// Transpile Apex code to TypeScript
///
/// # Arguments
/// * `source` - The Apex source code
/// * `options` - Optional transpilation options (JSON object)
///
/// Options:
/// - `typescript`: boolean - Generate TypeScript (with types) or plain JavaScript (default: true)
/// - `asyncDatabase`: boolean - Generate async/await for SOQL/DML (default: true)
/// - `includeImports`: boolean - Include runtime import statement (default: true)
/// - `indent`: string - Indentation string (default: "  ")
///
/// # Returns
/// JSON object with:
/// - `success`: boolean
/// - `typescript`: the generated TypeScript/JavaScript code (if successful)
/// - `runtimeInterface`: TypeScript interface definition for the runtime
/// - `error`: error message (if failed)
#[wasm_bindgen(js_name = transpileApex)]
pub fn transpile_apex(source: &str, options: JsValue) -> JsValue {
    // Parse options if provided
    let transpile_options = if options.is_undefined() || options.is_null() {
        TranspileOptions::default()
    } else {
        match serde_wasm_bindgen::from_value::<serde_json::Value>(options) {
            Ok(opts_json) => {
                let mut opts = TranspileOptions::default();
                if let Some(ts) = opts_json.get("typescript").and_then(|v| v.as_bool()) {
                    opts.typescript = ts;
                }
                if let Some(async_db) = opts_json.get("asyncDatabase").and_then(|v| v.as_bool()) {
                    opts.async_database = async_db;
                }
                if let Some(imports) = opts_json.get("includeImports").and_then(|v| v.as_bool()) {
                    opts.include_imports = imports;
                }
                if let Some(indent) = opts_json.get("indent").and_then(|v| v.as_str()) {
                    opts.indent = indent.to_string();
                }
                opts
            }
            Err(_) => TranspileOptions::default(),
        }
    };

    // Parse the Apex code
    let compilation_unit = match parser::parse(source) {
        Ok(unit) => unit,
        Err(e) => {
            return to_js_value(&serde_json::json!({
                "success": false,
                "error": format!("Parse error: {}", e),
            }));
        }
    };

    // Transpile to TypeScript
    let mut transpiler = Transpiler::with_options(transpile_options);
    match transpiler.transpile(&compilation_unit) {
        Ok(typescript) => to_js_value(&serde_json::json!({
            "success": true,
            "typescript": typescript,
            "runtimeInterface": crate::transpile::context::RUNTIME_INTERFACE,
        })),
        Err(e) => to_js_value(&serde_json::json!({
            "success": false,
            "error": format!("Transpilation error: {}", e),
        })),
    }
}

/// Get the ApexRuntime TypeScript interface definition
///
/// This returns the TypeScript interface that the runtime must implement
/// to execute transpiled Apex code.
#[wasm_bindgen(js_name = getRuntimeInterface)]
pub fn get_runtime_interface() -> String {
    crate::transpile::context::RUNTIME_INTERFACE.to_string()
}

// ============================================================================
// Helper functions for SOQL extraction
// ============================================================================

fn parse_field_json(json: &serde_json::Value) -> Result<FieldDescribe, String> {
    let name = json["name"]
        .as_str()
        .ok_or_else(|| "Missing field 'name'".to_string())?;

    let field_type_str = json["type"]
        .as_str()
        .ok_or_else(|| "Missing field 'type'".to_string())?;

    let field_type = parse_field_type(field_type_str)?;
    let mut field = FieldDescribe::new(name, field_type);

    // Optional: referenceTo for lookups (can be string or array)
    if let Some(ref_to) = json["referenceTo"].as_str() {
        field = field.with_reference(ref_to);
    } else if let Some(refs) = json["referenceTo"].as_array() {
        let ref_names: Vec<String> = refs
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        if ref_names.len() == 1 {
            field = field.with_reference(&ref_names[0]);
        } else if !ref_names.is_empty() {
            field = field.with_polymorphic_reference(ref_names);
        }
    }

    // Optional: relationshipName
    if let Some(rel_name) = json["relationshipName"].as_str() {
        field = field.with_relationship_name(rel_name);
    }

    // Optional: nillable
    if let Some(nillable) = json["nillable"].as_bool() {
        field = field.with_nillable(nillable);
    }

    Ok(field)
}

fn parse_field_type(s: &str) -> Result<SalesforceFieldType, String> {
    match s.to_lowercase().as_str() {
        "id" => Ok(SalesforceFieldType::Id),
        "string" | "text" => Ok(SalesforceFieldType::String),
        "textarea" => Ok(SalesforceFieldType::TextArea),
        "longtextarea" => Ok(SalesforceFieldType::LongTextArea),
        "richtextarea" => Ok(SalesforceFieldType::RichTextArea),
        "boolean" | "checkbox" => Ok(SalesforceFieldType::Boolean),
        "integer" | "int" => Ok(SalesforceFieldType::Integer),
        "double" | "number" => Ok(SalesforceFieldType::Double),
        "currency" => Ok(SalesforceFieldType::Currency),
        "percent" => Ok(SalesforceFieldType::Percent),
        "date" => Ok(SalesforceFieldType::Date),
        "datetime" => Ok(SalesforceFieldType::DateTime),
        "time" => Ok(SalesforceFieldType::Time),
        "phone" => Ok(SalesforceFieldType::Phone),
        "email" => Ok(SalesforceFieldType::Email),
        "url" => Ok(SalesforceFieldType::Url),
        "picklist" => Ok(SalesforceFieldType::Picklist),
        "multipicklist" => Ok(SalesforceFieldType::MultiPicklist),
        "lookup" => Ok(SalesforceFieldType::Lookup),
        "masterdetail" => Ok(SalesforceFieldType::MasterDetail),
        "reference" => Ok(SalesforceFieldType::Reference),
        "address" => Ok(SalesforceFieldType::Address),
        "location" | "geolocation" => Ok(SalesforceFieldType::Location),
        "autonumber" | "auto" => Ok(SalesforceFieldType::Auto),
        _ => Err(format!("Unknown field type: {}", s)),
    }
}

/// Extract SOQL queries as debug strings from type declarations
fn extract_soql_from_type_declaration(decl: &TypeDeclaration, queries: &mut Vec<String>) {
    match decl {
        TypeDeclaration::Class(class) => {
            extract_soql_from_class(class, queries);
        }
        TypeDeclaration::Interface(iface) => {
            extract_soql_from_interface(iface, queries);
        }
        TypeDeclaration::Trigger(trigger) => {
            extract_soql_from_block(&trigger.body, queries);
        }
        TypeDeclaration::Enum(_) => {}
    }
}

fn extract_soql_from_class(class: &ClassDeclaration, queries: &mut Vec<String>) {
    for member in &class.members {
        match member {
            ClassMember::Method(method) => {
                extract_soql_from_method(method, queries);
            }
            ClassMember::Constructor(ctor) => {
                extract_soql_from_block(&ctor.body, queries);
            }
            ClassMember::Property(prop) => {
                extract_soql_from_property(prop, queries);
            }
            ClassMember::StaticBlock(block) => {
                extract_soql_from_block(block, queries);
            }
            ClassMember::InnerClass(inner) => {
                extract_soql_from_class(inner, queries);
            }
            ClassMember::InnerInterface(inner) => {
                extract_soql_from_interface(inner, queries);
            }
            ClassMember::Field(field) => {
                for declarator in &field.declarators {
                    if let Some(ref init) = declarator.initializer {
                        extract_soql_from_expression(init, queries);
                    }
                }
            }
            ClassMember::InnerEnum(_) => {}
        }
    }
}

fn extract_soql_from_interface(_iface: &InterfaceDeclaration, _queries: &mut Vec<String>) {
    // Interfaces don't have method bodies with executable code
}

fn extract_soql_from_method(method: &MethodDeclaration, queries: &mut Vec<String>) {
    if let Some(ref body) = method.body {
        extract_soql_from_block(body, queries);
    }
}

fn extract_soql_from_property(prop: &PropertyDeclaration, queries: &mut Vec<String>) {
    if let Some(ref getter) = prop.getter {
        if let Some(ref body) = getter.body {
            extract_soql_from_block(body, queries);
        }
    }
    if let Some(ref setter) = prop.setter {
        if let Some(ref body) = setter.body {
            extract_soql_from_block(body, queries);
        }
    }
}

fn extract_soql_from_block(block: &Block, queries: &mut Vec<String>) {
    for stmt in &block.statements {
        extract_soql_from_statement(stmt, queries);
    }
}

fn extract_soql_from_statement(stmt: &Statement, queries: &mut Vec<String>) {
    match stmt {
        Statement::Expression(expr_stmt) => {
            extract_soql_from_expression(&expr_stmt.expression, queries);
        }
        Statement::LocalVariable(var_decl) => {
            for declarator in &var_decl.declarators {
                if let Some(ref init) = declarator.initializer {
                    extract_soql_from_expression(init, queries);
                }
            }
        }
        Statement::If(if_stmt) => {
            extract_soql_from_expression(&if_stmt.condition, queries);
            extract_soql_from_statement(&if_stmt.then_branch, queries);
            if let Some(ref else_branch) = if_stmt.else_branch {
                extract_soql_from_statement(else_branch, queries);
            }
        }
        Statement::For(for_stmt) => {
            if let Some(ref init) = for_stmt.init {
                extract_soql_from_for_init(init, queries);
            }
            if let Some(ref condition) = for_stmt.condition {
                extract_soql_from_expression(condition, queries);
            }
            for update in &for_stmt.update {
                extract_soql_from_expression(update, queries);
            }
            extract_soql_from_statement(&for_stmt.body, queries);
        }
        Statement::ForEach(foreach_stmt) => {
            extract_soql_from_expression(&foreach_stmt.iterable, queries);
            extract_soql_from_statement(&foreach_stmt.body, queries);
        }
        Statement::While(while_stmt) => {
            extract_soql_from_expression(&while_stmt.condition, queries);
            extract_soql_from_statement(&while_stmt.body, queries);
        }
        Statement::DoWhile(do_stmt) => {
            extract_soql_from_statement(&do_stmt.body, queries);
            extract_soql_from_expression(&do_stmt.condition, queries);
        }
        Statement::Return(ret) => {
            if let Some(ref expr) = ret.value {
                extract_soql_from_expression(expr, queries);
            }
        }
        Statement::Try(try_stmt) => {
            extract_soql_from_block(&try_stmt.try_block, queries);
            for catch in &try_stmt.catch_clauses {
                extract_soql_from_block(&catch.block, queries);
            }
            if let Some(ref finally) = try_stmt.finally_block {
                extract_soql_from_block(finally, queries);
            }
        }
        Statement::Block(block) => {
            extract_soql_from_block(block, queries);
        }
        Statement::Switch(switch) => {
            extract_soql_from_expression(&switch.expression, queries);
            for when_clause in &switch.when_clauses {
                extract_soql_from_block(&when_clause.block, queries);
            }
        }
        Statement::Throw(throw) => {
            extract_soql_from_expression(&throw.exception, queries);
        }
        Statement::Dml(dml) => {
            extract_soql_from_expression(&dml.expression, queries);
        }
        Statement::Break(_) | Statement::Continue(_) | Statement::Empty(_) => {}
    }
}

fn extract_soql_from_for_init(init: &ForInit, queries: &mut Vec<String>) {
    match init {
        ForInit::Variables(var_decl) => {
            for declarator in &var_decl.declarators {
                if let Some(ref expr) = declarator.initializer {
                    extract_soql_from_expression(expr, queries);
                }
            }
        }
        ForInit::Expressions(exprs) => {
            for expr in exprs {
                extract_soql_from_expression(expr, queries);
            }
        }
    }
}

fn extract_soql_from_expression(expr: &Expression, queries: &mut Vec<String>) {
    match expr {
        Expression::Soql(query) => {
            queries.push(format!("{:?}", query));
        }
        Expression::Binary(binary) => {
            extract_soql_from_expression(&binary.left, queries);
            extract_soql_from_expression(&binary.right, queries);
        }
        Expression::Unary(unary) => {
            extract_soql_from_expression(&unary.operand, queries);
        }
        Expression::MethodCall(call) => {
            if let Some(ref object) = call.object {
                extract_soql_from_expression(object, queries);
            }
            for arg in &call.arguments {
                extract_soql_from_expression(arg, queries);
            }
        }
        Expression::FieldAccess(access) => {
            extract_soql_from_expression(&access.object, queries);
        }
        Expression::ArrayAccess(arr) => {
            extract_soql_from_expression(&arr.array, queries);
            extract_soql_from_expression(&arr.index, queries);
        }
        Expression::Ternary(cond) => {
            extract_soql_from_expression(&cond.condition, queries);
            extract_soql_from_expression(&cond.then_expr, queries);
            extract_soql_from_expression(&cond.else_expr, queries);
        }
        Expression::Assignment(assign) => {
            extract_soql_from_expression(&assign.target, queries);
            extract_soql_from_expression(&assign.value, queries);
        }
        Expression::New(new_obj) => {
            for arg in &new_obj.arguments {
                extract_soql_from_expression(arg, queries);
            }
        }
        Expression::NewArray(new_arr) => {
            if let Some(ref size) = new_arr.size {
                extract_soql_from_expression(size, queries);
            }
            if let Some(ref init) = new_arr.initializer {
                for item in init {
                    extract_soql_from_expression(item, queries);
                }
            }
        }
        Expression::NewMap(new_map) => {
            if let Some(ref init) = new_map.initializer {
                for (k, v) in init {
                    extract_soql_from_expression(k, queries);
                    extract_soql_from_expression(v, queries);
                }
            }
        }
        Expression::Cast(cast) => {
            extract_soql_from_expression(&cast.expression, queries);
        }
        Expression::Instanceof(inst) => {
            extract_soql_from_expression(&inst.expression, queries);
        }
        Expression::Parenthesized(inner, _) => {
            extract_soql_from_expression(inner, queries);
        }
        Expression::SafeNavigation(nav) => {
            extract_soql_from_expression(&nav.object, queries);
        }
        Expression::NullCoalesce(nc) => {
            extract_soql_from_expression(&nc.left, queries);
            extract_soql_from_expression(&nc.right, queries);
        }
        Expression::ListLiteral(items, _) | Expression::SetLiteral(items, _) => {
            for item in items {
                extract_soql_from_expression(item, queries);
            }
        }
        Expression::MapLiteral(pairs, _) => {
            for (k, v) in pairs {
                extract_soql_from_expression(k, queries);
                extract_soql_from_expression(v, queries);
            }
        }
        Expression::PostIncrement(e, _)
        | Expression::PostDecrement(e, _)
        | Expression::PreIncrement(e, _)
        | Expression::PreDecrement(e, _) => {
            extract_soql_from_expression(e, queries);
        }
        // Literals and simple expressions
        Expression::Null(_)
        | Expression::Boolean(_, _)
        | Expression::Integer(_, _)
        | Expression::Long(_, _)
        | Expression::Double(_, _)
        | Expression::String(_, _)
        | Expression::Identifier(_, _)
        | Expression::This(_)
        | Expression::Super(_)
        | Expression::BindVariable(_, _)
        | Expression::Sosl(_) => {}
    }
}

// ============================================================================
// Helper to extract SOQL query references (not strings)
// ============================================================================

fn extract_soql_refs_from_type_declaration<'a>(
    decl: &'a TypeDeclaration,
    queries: &mut Vec<&'a SoqlQuery>,
) {
    match decl {
        TypeDeclaration::Class(class) => {
            extract_soql_refs_from_class(class, queries);
        }
        TypeDeclaration::Interface(_) => {}
        TypeDeclaration::Trigger(trigger) => {
            extract_soql_refs_from_block(&trigger.body, queries);
        }
        TypeDeclaration::Enum(_) => {}
    }
}

fn extract_soql_refs_from_class<'a>(class: &'a ClassDeclaration, queries: &mut Vec<&'a SoqlQuery>) {
    for member in &class.members {
        match member {
            ClassMember::Method(method) => {
                if let Some(ref body) = method.body {
                    extract_soql_refs_from_block(body, queries);
                }
            }
            ClassMember::Constructor(ctor) => {
                extract_soql_refs_from_block(&ctor.body, queries);
            }
            ClassMember::Property(prop) => {
                if let Some(ref getter) = prop.getter {
                    if let Some(ref body) = getter.body {
                        extract_soql_refs_from_block(body, queries);
                    }
                }
                if let Some(ref setter) = prop.setter {
                    if let Some(ref body) = setter.body {
                        extract_soql_refs_from_block(body, queries);
                    }
                }
            }
            ClassMember::StaticBlock(block) => {
                extract_soql_refs_from_block(block, queries);
            }
            ClassMember::InnerClass(inner) => {
                extract_soql_refs_from_class(inner, queries);
            }
            ClassMember::Field(field) => {
                for declarator in &field.declarators {
                    if let Some(ref init) = declarator.initializer {
                        extract_soql_refs_from_expression(init, queries);
                    }
                }
            }
            _ => {}
        }
    }
}

fn extract_soql_refs_from_block<'a>(block: &'a Block, queries: &mut Vec<&'a SoqlQuery>) {
    for stmt in &block.statements {
        extract_soql_refs_from_statement(stmt, queries);
    }
}

fn extract_soql_refs_from_statement<'a>(stmt: &'a Statement, queries: &mut Vec<&'a SoqlQuery>) {
    match stmt {
        Statement::Expression(expr_stmt) => {
            extract_soql_refs_from_expression(&expr_stmt.expression, queries);
        }
        Statement::LocalVariable(var_decl) => {
            for declarator in &var_decl.declarators {
                if let Some(ref init) = declarator.initializer {
                    extract_soql_refs_from_expression(init, queries);
                }
            }
        }
        Statement::If(if_stmt) => {
            extract_soql_refs_from_expression(&if_stmt.condition, queries);
            extract_soql_refs_from_statement(&if_stmt.then_branch, queries);
            if let Some(ref else_branch) = if_stmt.else_branch {
                extract_soql_refs_from_statement(else_branch, queries);
            }
        }
        Statement::For(for_stmt) => {
            if let Some(ref init) = for_stmt.init {
                extract_soql_refs_from_for_init(init, queries);
            }
            if let Some(ref condition) = for_stmt.condition {
                extract_soql_refs_from_expression(condition, queries);
            }
            for update in &for_stmt.update {
                extract_soql_refs_from_expression(update, queries);
            }
            extract_soql_refs_from_statement(&for_stmt.body, queries);
        }
        Statement::ForEach(foreach_stmt) => {
            extract_soql_refs_from_expression(&foreach_stmt.iterable, queries);
            extract_soql_refs_from_statement(&foreach_stmt.body, queries);
        }
        Statement::While(while_stmt) => {
            extract_soql_refs_from_expression(&while_stmt.condition, queries);
            extract_soql_refs_from_statement(&while_stmt.body, queries);
        }
        Statement::DoWhile(do_stmt) => {
            extract_soql_refs_from_statement(&do_stmt.body, queries);
            extract_soql_refs_from_expression(&do_stmt.condition, queries);
        }
        Statement::Return(ret) => {
            if let Some(ref expr) = ret.value {
                extract_soql_refs_from_expression(expr, queries);
            }
        }
        Statement::Try(try_stmt) => {
            extract_soql_refs_from_block(&try_stmt.try_block, queries);
            for catch in &try_stmt.catch_clauses {
                extract_soql_refs_from_block(&catch.block, queries);
            }
            if let Some(ref finally) = try_stmt.finally_block {
                extract_soql_refs_from_block(finally, queries);
            }
        }
        Statement::Block(block) => {
            extract_soql_refs_from_block(block, queries);
        }
        Statement::Switch(switch) => {
            extract_soql_refs_from_expression(&switch.expression, queries);
            for when_clause in &switch.when_clauses {
                extract_soql_refs_from_block(&when_clause.block, queries);
            }
        }
        Statement::Throw(throw) => {
            extract_soql_refs_from_expression(&throw.exception, queries);
        }
        Statement::Dml(dml) => {
            extract_soql_refs_from_expression(&dml.expression, queries);
        }
        Statement::Break(_) | Statement::Continue(_) | Statement::Empty(_) => {}
    }
}

fn extract_soql_refs_from_for_init<'a>(init: &'a ForInit, queries: &mut Vec<&'a SoqlQuery>) {
    match init {
        ForInit::Variables(var_decl) => {
            for declarator in &var_decl.declarators {
                if let Some(ref expr) = declarator.initializer {
                    extract_soql_refs_from_expression(expr, queries);
                }
            }
        }
        ForInit::Expressions(exprs) => {
            for expr in exprs {
                extract_soql_refs_from_expression(expr, queries);
            }
        }
    }
}

fn extract_soql_refs_from_expression<'a>(expr: &'a Expression, queries: &mut Vec<&'a SoqlQuery>) {
    match expr {
        Expression::Soql(query) => {
            queries.push(query);
        }
        Expression::Binary(binary) => {
            extract_soql_refs_from_expression(&binary.left, queries);
            extract_soql_refs_from_expression(&binary.right, queries);
        }
        Expression::Unary(unary) => {
            extract_soql_refs_from_expression(&unary.operand, queries);
        }
        Expression::MethodCall(call) => {
            if let Some(ref object) = call.object {
                extract_soql_refs_from_expression(object, queries);
            }
            for arg in &call.arguments {
                extract_soql_refs_from_expression(arg, queries);
            }
        }
        Expression::FieldAccess(access) => {
            extract_soql_refs_from_expression(&access.object, queries);
        }
        Expression::ArrayAccess(arr) => {
            extract_soql_refs_from_expression(&arr.array, queries);
            extract_soql_refs_from_expression(&arr.index, queries);
        }
        Expression::Ternary(cond) => {
            extract_soql_refs_from_expression(&cond.condition, queries);
            extract_soql_refs_from_expression(&cond.then_expr, queries);
            extract_soql_refs_from_expression(&cond.else_expr, queries);
        }
        Expression::Assignment(assign) => {
            extract_soql_refs_from_expression(&assign.target, queries);
            extract_soql_refs_from_expression(&assign.value, queries);
        }
        Expression::New(new_obj) => {
            for arg in &new_obj.arguments {
                extract_soql_refs_from_expression(arg, queries);
            }
        }
        Expression::NewArray(new_arr) => {
            if let Some(ref size) = new_arr.size {
                extract_soql_refs_from_expression(size, queries);
            }
            if let Some(ref init) = new_arr.initializer {
                for item in init {
                    extract_soql_refs_from_expression(item, queries);
                }
            }
        }
        Expression::NewMap(new_map) => {
            if let Some(ref init) = new_map.initializer {
                for (k, v) in init {
                    extract_soql_refs_from_expression(k, queries);
                    extract_soql_refs_from_expression(v, queries);
                }
            }
        }
        Expression::Cast(cast) => {
            extract_soql_refs_from_expression(&cast.expression, queries);
        }
        Expression::Instanceof(inst) => {
            extract_soql_refs_from_expression(&inst.expression, queries);
        }
        Expression::Parenthesized(inner, _) => {
            extract_soql_refs_from_expression(inner, queries);
        }
        Expression::SafeNavigation(nav) => {
            extract_soql_refs_from_expression(&nav.object, queries);
        }
        Expression::NullCoalesce(nc) => {
            extract_soql_refs_from_expression(&nc.left, queries);
            extract_soql_refs_from_expression(&nc.right, queries);
        }
        Expression::ListLiteral(items, _) | Expression::SetLiteral(items, _) => {
            for item in items {
                extract_soql_refs_from_expression(item, queries);
            }
        }
        Expression::MapLiteral(pairs, _) => {
            for (k, v) in pairs {
                extract_soql_refs_from_expression(k, queries);
                extract_soql_refs_from_expression(v, queries);
            }
        }
        Expression::PostIncrement(e, _)
        | Expression::PostDecrement(e, _)
        | Expression::PreIncrement(e, _)
        | Expression::PreDecrement(e, _) => {
            extract_soql_refs_from_expression(e, queries);
        }
        _ => {}
    }
}
