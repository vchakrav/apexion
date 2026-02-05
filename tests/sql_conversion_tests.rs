//! Comprehensive tests for SOQL to SQL conversion

use apexrust::parse;
use apexrust::sql::{
    ChildRelationship, ConversionConfig, DdlGenerator, FieldDescribe, SObjectDescribe,
    SalesforceFieldType, SalesforceSchema, SoqlToSqlConverter, SqlDialect,
};
use apexrust::SoqlQuery;

/// Helper to extract SOQL from a test wrapper class
fn extract_soql(source: &str) -> SoqlQuery {
    let full_source = format!(
        "class Test {{ void test() {{ List<SObject> x = [{}]; }} }}",
        source
    );
    let cu = parse(&full_source).expect("Parse failed");
    if let apexrust::TypeDeclaration::Class(class) = &cu.declarations[0] {
        if let apexrust::ClassMember::Method(method) = &class.members[0] {
            if let Some(block) = &method.body {
                if let apexrust::Statement::LocalVariable(lv) = &block.statements[0] {
                    if let Some(apexrust::Expression::Soql(soql)) = &lv.declarators[0].initializer {
                        return (**soql).clone();
                    }
                }
            }
        }
    }
    panic!("Could not extract SOQL query");
}

/// Create a standard test schema with Account, Contact, and Opportunity
fn create_test_schema() -> SalesforceSchema {
    let mut schema = SalesforceSchema::new();

    // Account
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
        "NumberOfEmployees",
        SalesforceFieldType::Integer,
    ));
    account.add_field(FieldDescribe::new("Website", SalesforceFieldType::Url));
    account.add_field(FieldDescribe::new(
        "CreatedDate",
        SalesforceFieldType::DateTime,
    ));
    account.add_field(FieldDescribe::new(
        "IsDeleted",
        SalesforceFieldType::Boolean,
    ));
    account.add_field(
        FieldDescribe::new("OwnerId", SalesforceFieldType::Reference)
            .with_polymorphic_reference(vec!["User".to_string(), "Group".to_string()])
            .with_relationship_name("Owner"),
    );
    account.add_child_relationship(ChildRelationship::new("Contacts", "Contact", "AccountId"));
    account.add_child_relationship(ChildRelationship::new(
        "Opportunities",
        "Opportunity",
        "AccountId",
    ));
    schema.add_object(account);

    // Contact
    let mut contact = SObjectDescribe::new("Contact");
    contact.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    contact.add_field(FieldDescribe::new("FirstName", SalesforceFieldType::String));
    contact.add_field(FieldDescribe::new("LastName", SalesforceFieldType::String));
    contact.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
    contact.add_field(FieldDescribe::new("Phone", SalesforceFieldType::Phone));
    contact.add_field(FieldDescribe::new("Title", SalesforceFieldType::String));
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

    // Opportunity
    let mut opportunity = SObjectDescribe::new("Opportunity");
    opportunity.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    opportunity.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    opportunity.add_field(FieldDescribe::new("Amount", SalesforceFieldType::Currency));
    opportunity.add_field(FieldDescribe::new(
        "StageName",
        SalesforceFieldType::Picklist,
    ));
    opportunity.add_field(FieldDescribe::new("CloseDate", SalesforceFieldType::Date));
    opportunity.add_field(
        FieldDescribe::new("AccountId", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Account"),
    );
    schema.add_object(opportunity);

    // User (for Owner relationship)
    let mut user = SObjectDescribe::new("User");
    user.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    user.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    user.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
    schema.add_object(user);

    schema
}

// =============================================================================
// Basic SELECT tests
// =============================================================================

#[test]
fn test_simple_select_postgres() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id, Name FROM Account");

    let config = ConversionConfig {
        dialect: SqlDialect::Postgres,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("SELECT"));
    assert!(result.sql.to_lowercase().contains("id"));
    assert!(result.sql.to_lowercase().contains("name"));
    assert!(result.sql.contains("\"account\""));
}

#[test]
fn test_simple_select_sqlite() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id, Name FROM Account");

    let config = ConversionConfig {
        dialect: SqlDialect::Sqlite,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("SELECT"));
    assert!(result.sql.contains("\"account\""));
}

#[test]
fn test_select_all_field_types() {
    let schema = create_test_schema();
    let soql = extract_soql(
        "SELECT Id, Name, Industry, AnnualRevenue, NumberOfEmployees, Website FROM Account",
    );

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.to_lowercase().contains("industry"));
    assert!(result.sql.to_lowercase().contains("annual_revenue"));
    assert!(result.sql.to_lowercase().contains("number_of_employees"));
    assert!(result.sql.to_lowercase().contains("website"));
}

// =============================================================================
// WHERE clause tests
// =============================================================================

#[test]
fn test_where_equals() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE Name = 'Acme'");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("WHERE"));
    assert!(result.sql.contains("= 'Acme'"));
}

#[test]
fn test_where_not_equals() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE Name != 'Acme'");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("!= 'Acme'"));
}

#[test]
fn test_where_comparison_operators() {
    let schema = create_test_schema();
    let soql = extract_soql(
        "SELECT Id FROM Account WHERE AnnualRevenue > 1000000 AND NumberOfEmployees >= 100",
    );

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("> 1000000"));
    assert!(result.sql.contains(">= 100"));
    assert!(result.sql.contains("AND"));
}

#[test]
fn test_where_like() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE Name LIKE 'Acme%'");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("LIKE"));
    assert!(result.sql.contains("'Acme%'"));
}

#[test]
fn test_where_in_list() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE Industry IN ('Technology', 'Finance')");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("IN"));
    assert!(result.sql.contains("'Technology'"));
    assert!(result.sql.contains("'Finance'"));
}

#[test]
fn test_where_not_in() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE Industry NOT IN ('Agriculture')");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("NOT IN"));
}

#[test]
fn test_where_and_or() {
    let schema = create_test_schema();
    let soql = extract_soql(
        "SELECT Id FROM Account WHERE (Name = 'Acme' OR Name = 'Test') AND Industry = 'Tech'",
    );

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("AND"));
    assert!(result.sql.contains("OR"));
}

// =============================================================================
// Bind variable tests
// =============================================================================

#[test]
fn test_bind_variable_postgres() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE Name = :accountName");

    let config = ConversionConfig {
        dialect: SqlDialect::Postgres,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("$1"));
    assert_eq!(result.parameters.len(), 1);
    assert_eq!(result.parameters[0].original_name, "accountName");
    assert_eq!(result.parameters[0].placeholder, "$1");
}

#[test]
fn test_bind_variable_sqlite() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE Name = :accountName");

    let config = ConversionConfig {
        dialect: SqlDialect::Sqlite,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("?1"));
    assert_eq!(result.parameters[0].placeholder, "?1");
}

#[test]
fn test_multiple_bind_variables() {
    let schema = create_test_schema();
    let soql = extract_soql(
        "SELECT Id FROM Account WHERE Name = :name AND Industry = :industry LIMIT :maxRecords",
    );

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert_eq!(result.parameters.len(), 3);
    assert!(result.sql.contains("$1"));
    assert!(result.sql.contains("$2"));
    assert!(result.sql.contains("$3"));
}

// =============================================================================
// ORDER BY tests
// =============================================================================

#[test]
fn test_order_by_asc() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id, Name FROM Account ORDER BY Name");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("ORDER BY"));
}

#[test]
fn test_order_by_desc() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id, Name FROM Account ORDER BY Name DESC");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("DESC"));
}

#[test]
fn test_order_by_nulls() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account ORDER BY Name ASC NULLS LAST");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("NULLS LAST"));
}

#[test]
fn test_order_by_multiple() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account ORDER BY Industry, Name DESC");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    let order_by_pos = result.sql.find("ORDER BY").unwrap();
    let order_clause = &result.sql[order_by_pos..];
    assert!(order_clause.contains("industry"));
    assert!(order_clause.contains("DESC"));
}

// =============================================================================
// LIMIT and OFFSET tests
// =============================================================================

#[test]
fn test_limit() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account LIMIT 10");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("LIMIT 10"));
}

#[test]
fn test_offset() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account LIMIT 10 OFFSET 20");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("LIMIT 10"));
    assert!(result.sql.contains("OFFSET 20"));
}

// =============================================================================
// Aggregate function tests
// =============================================================================

#[test]
fn test_count_star() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT COUNT() FROM Account");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("COUNT("));
}

#[test]
fn test_count_field() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT COUNT(Id) FROM Account");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("COUNT("));
}

#[test]
fn test_aggregate_with_alias() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT COUNT(Id) total FROM Account");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("AS \"total\""));
}

#[test]
fn test_sum_avg_min_max() {
    let schema = create_test_schema();
    let soql = extract_soql(
        "SELECT SUM(AnnualRevenue) s, AVG(NumberOfEmployees) a FROM Account GROUP BY Industry",
    );

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("SUM("));
    assert!(result.sql.contains("AVG("));
}

// =============================================================================
// GROUP BY and HAVING tests
// =============================================================================

#[test]
fn test_group_by() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Industry, COUNT(Id) FROM Account GROUP BY Industry");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("GROUP BY"));
}

#[test]
fn test_having() {
    let schema = create_test_schema();
    // Use a simpler HAVING clause that the parser supports
    let soql = extract_soql(
        "SELECT Industry, COUNT(Id) cnt FROM Account GROUP BY Industry HAVING cnt > 5",
    );

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("HAVING"));
}

// =============================================================================
// Relationship query tests
// =============================================================================

#[test]
fn test_parent_relationship() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id, Account.Name FROM Contact");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("LEFT JOIN"));
    assert!(result.sql.contains("\"account\""));
}

#[test]
fn test_child_subquery() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id, Name, (SELECT Id, Email FROM Contacts) FROM Account");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    // Should use JSON aggregation for subquery
    assert!(result.sql.contains("json_agg") || result.sql.contains("json_group_array"));
}

// =============================================================================
// FOR clause tests
// =============================================================================

#[test]
fn test_for_update_postgres() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account FOR UPDATE");

    let config = ConversionConfig {
        dialect: SqlDialect::Postgres,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("FOR UPDATE"));
}

#[test]
fn test_for_update_sqlite_warning() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account FOR UPDATE");

    let config = ConversionConfig {
        dialect: SqlDialect::Sqlite,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    // SQLite doesn't support FOR UPDATE
    assert!(!result.sql.contains("FOR UPDATE"));
    assert!(!result.warnings.is_empty());
}

// =============================================================================
// Date literal tests
// =============================================================================

#[test]
fn test_date_literal_today_postgres() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE CreatedDate = TODAY");

    let config = ConversionConfig {
        dialect: SqlDialect::Postgres,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("CURRENT_DATE"));
}

#[test]
fn test_date_literal_today_sqlite() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE CreatedDate = TODAY");

    let config = ConversionConfig {
        dialect: SqlDialect::Sqlite,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("date('now')"));
}

#[test]
fn test_date_literal_last_n_days() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WHERE CreatedDate = LAST_N_DAYS:30");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    // Should expand to a range comparison
    assert!(result.sql.contains(">="));
    assert!(result.sql.contains("<"));
}

// =============================================================================
// DDL generation tests
// =============================================================================

#[test]
fn test_ddl_create_table_postgres() {
    let schema = create_test_schema();
    let generator = DdlGenerator::new(SqlDialect::Postgres);

    let account = schema.get_object("Account").unwrap();
    let ddl = generator.generate_table(account);

    assert!(ddl.contains("CREATE TABLE \"account\""));
    assert!(ddl.contains("\"id\" TEXT PRIMARY KEY"));
    assert!(ddl.contains("\"name\" TEXT"));
    assert!(ddl.contains("\"annual_revenue\" NUMERIC"));
    assert!(ddl.contains("\"is_deleted\" BOOLEAN"));
}

#[test]
fn test_ddl_create_table_sqlite() {
    let schema = create_test_schema();
    let generator = DdlGenerator::new(SqlDialect::Sqlite);

    let account = schema.get_object("Account").unwrap();
    let ddl = generator.generate_table(account);

    assert!(ddl.contains("CREATE TABLE \"account\""));
    // SQLite uses INTEGER for boolean
    assert!(ddl.contains("\"is_deleted\" INTEGER"));
}

#[test]
fn test_ddl_polymorphic_field() {
    let schema = create_test_schema();
    let generator = DdlGenerator::new(SqlDialect::Postgres);

    let account = schema.get_object("Account").unwrap();
    let ddl = generator.generate_table(account);

    // Polymorphic OwnerId should have both ID and type columns
    assert!(ddl.contains("\"owner_id\" TEXT"));
    assert!(ddl.contains("\"owner_id_type\" TEXT"));
}

#[test]
fn test_ddl_foreign_key() {
    let schema = create_test_schema();
    let generator = DdlGenerator::new(SqlDialect::Postgres);

    let contact = schema.get_object("Contact").unwrap();
    let ddl = generator.generate_table(contact);

    assert!(ddl.contains("FOREIGN KEY"));
    assert!(ddl.contains("REFERENCES \"account\"(id)"));
}

#[test]
fn test_ddl_indexes() {
    let schema = create_test_schema();
    let generator = DdlGenerator::new(SqlDialect::Postgres);

    let contact = schema.get_object("Contact").unwrap();
    let indexes = generator.generate_indexes(contact);

    // Should have index for AccountId (lookup field)
    assert!(indexes.iter().any(|i| i.contains("account_id")));
}

#[test]
fn test_ddl_full_schema() {
    let schema = create_test_schema();
    let generator = DdlGenerator::new(SqlDialect::Postgres);

    let ddl = generator.generate_schema(&schema);

    assert!(ddl.contains("CREATE TABLE \"account\""));
    assert!(ddl.contains("CREATE TABLE \"contact\""));
    assert!(ddl.contains("CREATE TABLE \"opportunity\""));
    assert!(ddl.contains("CREATE TABLE \"user\""));
    assert!(ddl.contains("CREATE INDEX"));
}

// =============================================================================
// Configuration tests
// =============================================================================

#[test]
fn test_filter_deleted() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account");

    let config = ConversionConfig {
        filter_deleted: true,
        ..Default::default()
    };
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    assert!(result.sql.contains("is_deleted"));
    assert!(result.sql.contains("FALSE") || result.sql.contains("false"));
}

#[test]
fn test_security_mode_warning() {
    let schema = create_test_schema();
    let soql = extract_soql("SELECT Id FROM Account WITH SECURITY_ENFORCED");

    let config = ConversionConfig::default();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter.convert(&soql).unwrap();

    // Security clause should be removed but recorded
    assert!(result.security_mode.is_some());
    assert!(!result.warnings.is_empty());
}
