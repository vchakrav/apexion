//! SOQL to SQL conversion module
//!
//! This module provides functionality to convert parsed SOQL queries to SQL
//! compatible with both SQLite and PostgreSQL, along with DDL generation
//! for modeling Salesforce org schema in relational databases.
//!
//! # Overview
//!
//! The conversion process involves:
//! 1. Defining a Salesforce schema (objects, fields, relationships)
//! 2. Parsing SOQL queries using the main parser
//! 3. Converting the SOQL AST to SQL using dialect-specific rules
//!
//! # Example
//!
//! ```rust
//! use apexrust::parse;
//! use apexrust::sql::{
//!     SalesforceSchema, SObjectDescribe, FieldDescribe, SalesforceFieldType,
//!     ChildRelationship, SoqlToSqlConverter, ConversionConfig, SqlDialect,
//!     DdlGenerator,
//! };
//!
//! // Create a schema
//! let mut schema = SalesforceSchema::new();
//!
//! let mut account = SObjectDescribe::new("Account");
//! account.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id));
//! account.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
//! account.add_child_relationship(ChildRelationship::new("Contacts", "Contact", "AccountId"));
//! schema.add_object(account);
//!
//! let mut contact = SObjectDescribe::new("Contact");
//! contact.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id));
//! contact.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
//! contact.add_field(
//!     FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
//!         .with_reference("Account")
//!         .with_relationship_name("Account")
//! );
//! schema.add_object(contact);
//!
//! // Generate DDL
//! let ddl_gen = DdlGenerator::new(SqlDialect::Postgres);
//! let ddl = ddl_gen.generate_schema(&schema);
//!
//! // Parse and convert SOQL (would extract from Apex source in practice)
//! // let soql_query = ...;
//! // let config = ConversionConfig::default();
//! // let mut converter = SoqlToSqlConverter::new(&schema, config);
//! // let result = converter.convert(&soql_query)?;
//! // println!("SQL: {}", result.sql);
//! ```
//!
//! # Features
//!
//! ## Supported SOQL Features
//!
//! - Basic SELECT with field lists
//! - WHERE clause with operators (=, !=, <, >, <=, >=, LIKE, IN, NOT IN)
//! - INCLUDES/EXCLUDES for multi-picklist fields
//! - ORDER BY with ASC/DESC and NULLS FIRST/LAST
//! - LIMIT and OFFSET
//! - GROUP BY and HAVING
//! - Aggregate functions (COUNT, SUM, AVG, MIN, MAX)
//! - Bind variables (:variableName) converted to parameterized queries
//! - Date literals (TODAY, LAST_N_DAYS, THIS_MONTH, etc.)
//! - Parent relationship queries (Account.Name)
//! - Child relationship subqueries (SELECT ... FROM Contacts)
//! - TYPEOF for polymorphic fields
//! - FOR UPDATE (PostgreSQL only)
//!
//! ## SQL Dialects
//!
//! - **PostgreSQL**: Full support including FOR UPDATE, TIMESTAMP, BOOLEAN
//! - **SQLite**: Compatible output using INTEGER for booleans, TEXT for dates
//!
//! ## Schema Modeling
//!
//! The schema model supports:
//! - All Salesforce field types
//! - Lookup and Master-Detail relationships
//! - Polymorphic fields (with type discriminator columns)
//! - Child relationships for subqueries
//! - Standard system fields (CreatedDate, LastModifiedDate, etc.)

pub mod converter;
pub mod date_literals;
pub mod ddl;
pub mod dialect;
pub mod error;
pub mod schema;
pub mod standard_objects;

// Re-export main types
pub use converter::{
    convert_soql, convert_soql_simple, BindVariableMode, ConversionConfig, SecurityMode,
    SoqlToSqlConverter, SqlConversion, SqlParameter,
};
pub use ddl::DdlGenerator;
pub use dialect::{DateUnit, PostgresDialect, SqlDialect, SqlDialectImpl, SqliteDialect};
pub use error::{ConversionError, ConversionResult, ConversionWarning};
pub use schema::{
    ChildRelationship, FieldDescribe, SObjectDescribe, SalesforceFieldType, SalesforceSchema,
    SchemaBuilder,
};
pub use standard_objects::create_sales_cloud_schema;
