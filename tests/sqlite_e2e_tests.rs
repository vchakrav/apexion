//! End-to-end tests that:
//! 1. Parse real Apex files
//! 2. Extract SOQL queries
//! 3. Generate DDL from schema
//! 4. Convert SOQL to SQL
//! 5. Execute SQL against SQLite

use apexrust::sql::{
    ChildRelationship, ConversionConfig, DdlGenerator, FieldDescribe, SObjectDescribe,
    SalesforceFieldType, SalesforceSchema, SoqlToSqlConverter, SqlDialect,
};
use apexrust::{parse, ClassMember, Expression, SoqlQuery, Statement, TypeDeclaration};
use rusqlite::{Connection, Result as SqliteResult};

/// Create a comprehensive Salesforce schema for testing
fn create_salesforce_schema() -> SalesforceSchema {
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
        "ShippingStreet",
        SalesforceFieldType::String,
    ));
    account.add_field(FieldDescribe::new(
        "ShippingCity",
        SalesforceFieldType::String,
    ));
    account.add_field(FieldDescribe::new(
        "ShippingState",
        SalesforceFieldType::String,
    ));
    account.add_field(FieldDescribe::new(
        "ShippingCountry",
        SalesforceFieldType::String,
    ));
    account.add_field(FieldDescribe::new(
        "AnnualRevenue",
        SalesforceFieldType::Currency,
    ));
    account.add_field(FieldDescribe::new(
        "NumberOfEmployees",
        SalesforceFieldType::Integer,
    ));
    account.add_field(FieldDescribe::new(
        "CreatedDate",
        SalesforceFieldType::DateTime,
    ));
    account.add_field(FieldDescribe::new(
        "IsDeleted",
        SalesforceFieldType::Boolean,
    ));
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
    contact.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    contact.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
    contact.add_field(FieldDescribe::new("Phone", SalesforceFieldType::Phone));
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

    // Junction__c (custom object for junction queries)
    let mut junction = SObjectDescribe::new("Junction__c");
    junction.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    junction.add_field(
        FieldDescribe::new("parent1__c", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("parent1__r"),
    );
    junction.add_field(
        FieldDescribe::new("Parent2__c", SalesforceFieldType::Lookup)
            .with_reference("Account")
            .with_relationship_name("Parent2__r"),
    );
    schema.add_object(junction);

    schema
}

/// Set up SQLite database with schema and sample data
fn setup_database(schema: &SalesforceSchema) -> SqliteResult<Connection> {
    let conn = Connection::open_in_memory()?;

    // Generate and execute DDL
    let generator = DdlGenerator::new(SqlDialect::Sqlite);
    let ddl = generator.generate_schema(schema);

    // Execute each statement separately
    for statement in ddl.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            conn.execute(trimmed, [])?;
        }
    }

    // Insert sample data
    insert_sample_data(&conn)?;

    Ok(conn)
}

/// Insert sample data for testing
fn insert_sample_data(conn: &Connection) -> SqliteResult<()> {
    // Accounts
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, shipping_street, shipping_city, shipping_state, shipping_country, annual_revenue, number_of_employees)
         VALUES ('001000000000001', 'Acme Corp', 'Technology', '123 Main St', 'San Francisco', 'CA', 'US', 1000000.0, 100)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, shipping_street, shipping_city, shipping_state, shipping_country, annual_revenue, number_of_employees)
         VALUES ('001000000000002', 'Global Industries', 'Manufacturing', '456 Oak Ave', 'London', 'Greater London', 'UK', 5000000.0, 500)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, shipping_street, shipping_city, shipping_state, shipping_country, annual_revenue, number_of_employees)
         VALUES ('001000000000003', 'Fast Food Inc', 'Fast Food - made whole', '789 Elm St', 'Indianapolis', 'IN', 'US', 250000.0, 50)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, shipping_state, shipping_country)
         VALUES ('001000000000004', 'Kansas Co', 'Retail', 'KS', 'US')",
        [],
    )?;

    // Contacts
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, account_id)
         VALUES ('003000000000001', 'John', 'Doe', 'John Doe', 'john@acme.com', '001000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, account_id)
         VALUES ('003000000000002', 'Jane', 'Smith', 'Jane Smith', 'jane@acme.com', '001000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, account_id)
         VALUES ('003000000000003', 'Bob', 'Wilson', 'Bob Wilson', 'bob@global.com', '001000000000002')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, account_id)
         VALUES ('003000000000004', 'Alice', 'Kansas', 'Alice Kansas', 'alice@kansas.com', '001000000000004')",
        [],
    )?;

    // Opportunities
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, amount, stage_name, close_date, account_id)
         VALUES ('006000000000001', 'Big Deal', 100000.0, 'Closed Won', '2024-01-15', '001000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, amount, stage_name, close_date, account_id)
         VALUES ('006000000000002', 'Medium Deal', 50000.0, 'Negotiation', '2024-03-01', '001000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, amount, stage_name, close_date, account_id)
         VALUES ('006000000000003', 'Small Deal', 10000.0, 'Prospecting', '2024-06-01', '001000000000002')",
        [],
    )?;

    // Junction records
    conn.execute(
        "INSERT INTO \"junction__c\" (id, parent1__c, parent2__c)
         VALUES ('a00000000000001', '001000000000001', '001000000000002')",
        [],
    )?;

    Ok(())
}

/// Extract all SOQL queries from an Apex source file
fn extract_soql_queries(source: &str) -> Vec<(String, SoqlQuery)> {
    let mut queries = Vec::new();

    let cu = match parse(source) {
        Ok(cu) => cu,
        Err(_) => return queries,
    };

    for decl in &cu.declarations {
        if let TypeDeclaration::Class(class) = decl {
            for member in &class.members {
                if let ClassMember::Method(method) = member {
                    if let Some(ref body) = method.body {
                        extract_soql_from_statements(&body.statements, &method.name, &mut queries);
                    }
                }
            }
        }
    }

    queries
}

fn extract_soql_from_statements(
    statements: &[Statement],
    method_name: &str,
    queries: &mut Vec<(String, SoqlQuery)>,
) {
    for stmt in statements {
        match stmt {
            Statement::LocalVariable(lv) => {
                for decl in &lv.declarators {
                    if let Some(ref init) = decl.initializer {
                        extract_soql_from_expression(init, method_name, queries);
                    }
                }
            }
            Statement::Return(ret) => {
                if let Some(ref expr) = ret.value {
                    extract_soql_from_expression(expr, method_name, queries);
                }
            }
            Statement::ForEach(fe) => {
                extract_soql_from_expression(&fe.iterable, method_name, queries);
                if let Statement::Block(block) = fe.body.as_ref() {
                    extract_soql_from_statements(&block.statements, method_name, queries);
                }
            }
            Statement::For(f) => {
                if let Statement::Block(block) = f.body.as_ref() {
                    extract_soql_from_statements(&block.statements, method_name, queries);
                }
            }
            Statement::If(if_stmt) => {
                if let Statement::Block(block) = if_stmt.then_branch.as_ref() {
                    extract_soql_from_statements(&block.statements, method_name, queries);
                }
                if let Some(ref else_branch) = if_stmt.else_branch {
                    if let Statement::Block(block) = else_branch.as_ref() {
                        extract_soql_from_statements(&block.statements, method_name, queries);
                    }
                }
            }
            Statement::Block(block) => {
                extract_soql_from_statements(&block.statements, method_name, queries);
            }
            Statement::Expression(expr_stmt) => {
                extract_soql_from_expression(&expr_stmt.expression, method_name, queries);
            }
            _ => {}
        }
    }
}

fn extract_soql_from_expression(
    expr: &Expression,
    method_name: &str,
    queries: &mut Vec<(String, SoqlQuery)>,
) {
    match expr {
        Expression::Soql(soql) => {
            queries.push((method_name.to_string(), (**soql).clone()));
        }
        Expression::Assignment(assign) => {
            extract_soql_from_expression(&assign.value, method_name, queries);
        }
        Expression::MethodCall(call) => {
            for arg in &call.arguments {
                extract_soql_from_expression(arg, method_name, queries);
            }
        }
        Expression::Cast(cast) => {
            extract_soql_from_expression(&cast.expression, method_name, queries);
        }
        Expression::ArrayAccess(aa) => {
            extract_soql_from_expression(&aa.array, method_name, queries);
        }
        Expression::FieldAccess(fa) => {
            extract_soql_from_expression(&fa.object, method_name, queries);
        }
        _ => {}
    }
}

/// Convert SOQL to SQL and execute against SQLite
fn convert_and_execute(
    conn: &Connection,
    schema: &SalesforceSchema,
    soql: &SoqlQuery,
    method_name: &str,
) -> Result<usize, String> {
    let config = ConversionConfig {
        dialect: SqlDialect::Sqlite,
        ..Default::default()
    };

    let mut converter = SoqlToSqlConverter::new(schema, config);
    let result = converter
        .convert(soql)
        .map_err(|e| format!("Conversion error: {}", e))?;

    // Prepare the query (without bind parameters for this test)
    let sql = &result.sql;

    // Execute the query
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("SQL prepare error for {}: {}\nSQL: {}", method_name, e, sql))?;

    let column_count = stmt.column_count();
    let rows: Vec<_> = stmt
        .query_map([], |_row| Ok(()))
        .map_err(|e| format!("SQL execute error for {}: {}\nSQL: {}", method_name, e, sql))?
        .collect();

    let row_count = rows.len();

    println!(
        "  {} - {} rows, {} columns\n    SQL: {}",
        method_name,
        row_count,
        column_count,
        sql.replace('\n', " ")
    );

    Ok(row_count)
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn test_e2e_simple_select() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Account> getAccounts() {
            return [SELECT Id, Name FROM Account];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    assert_eq!(queries.len(), 1);

    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 4); // We inserted 4 accounts
}

#[test]
fn test_e2e_select_with_where() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Account> getUKAccounts() {
            return [SELECT Name FROM Account WHERE ShippingCountry = 'UK'];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 1); // Only Global Industries is in UK
}

#[test]
fn test_e2e_select_with_limit() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Account> getLimitedAccounts() {
            return [SELECT Name FROM Account ORDER BY Name LIMIT 2];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 2);
}

#[test]
fn test_e2e_select_with_offset() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Account> getAccountsWithOffset() {
            return [SELECT Id FROM Account ORDER BY Name LIMIT 2 OFFSET 1];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 2);
}

#[test]
fn test_e2e_parent_relationship() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Contact> getContactsWithAccount() {
            return [SELECT Id, Name, Account.Name FROM Contact];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 4); // 4 contacts
}

#[test]
fn test_e2e_parent_relationship_in_where() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Contact> getKansasContacts() {
            return [SELECT Id, Name, Account.Name FROM Contact WHERE Account.ShippingState = 'KS'];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 1); // Only Alice Kansas
}

#[test]
fn test_e2e_child_subquery() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Account> getAccountsWithContacts() {
            return [SELECT Name, (SELECT Name FROM Contacts) FROM Account];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 4); // 4 accounts (with their contacts as JSON)
}

#[test]
fn test_e2e_aggregate_sum() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        AggregateResult[] getOpportunitySum() {
            return [SELECT SUM(Amount) total FROM Opportunity];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 1); // Single aggregate result
}

#[test]
fn test_e2e_aggregate_count() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        Integer getAccountCount() {
            return [SELECT COUNT() FROM Account];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 1); // Single count result
}

#[test]
fn test_e2e_group_by() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        AggregateResult[] getAccountsByCountry() {
            return [SELECT ShippingCountry, COUNT(Id) cnt FROM Account GROUP BY ShippingCountry];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 2); // US and UK
}

#[test]
fn test_e2e_complex_where() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Account> getFilteredAccounts() {
            return [
                SELECT Name
                FROM Account
                WHERE ShippingCountry = 'US' AND ShippingState = 'IN'
            ];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 1); // Fast Food Inc in Indianapolis, IN
}

#[test]
fn test_e2e_junction_query() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = r#"
    class Test {
        List<Junction__c> getJunctions() {
            return [SELECT Id, parent1__r.Name, Parent2__r.Name FROM Junction__c];
        }
    }
    "#;

    let queries = extract_soql_queries(source);
    let (method_name, soql) = &queries[0];
    let row_count =
        convert_and_execute(&conn, &schema, soql, method_name).expect("Failed to execute query");

    assert_eq!(row_count, 1); // One junction record
}

#[test]
fn test_e2e_soql_recipes_file() {
    let schema = create_salesforce_schema();
    let conn = setup_database(&schema).expect("Failed to set up database");

    let source = include_str!("apex_files/SOQLRecipes.cls");
    let queries = extract_soql_queries(source);

    println!(
        "\nExtracted {} SOQL queries from SOQLRecipes.cls:",
        queries.len()
    );

    let mut success_count = 0;
    let mut fail_count = 0;

    for (method_name, soql) in &queries {
        match convert_and_execute(&conn, &schema, soql, method_name) {
            Ok(_) => success_count += 1,
            Err(e) => {
                println!("  FAILED: {} - {}", method_name, e);
                fail_count += 1;
            }
        }
    }

    println!(
        "\nResults: {} succeeded, {} failed out of {} total",
        success_count,
        fail_count,
        queries.len()
    );

    // We expect most queries to work, but some may fail due to:
    // - Bind variables (which need runtime values)
    // - Custom objects not in schema
    // - Complex features not yet supported
    assert!(
        success_count > 0,
        "At least some queries should execute successfully"
    );
}

#[test]
fn test_ddl_creates_valid_schema() {
    let schema = create_salesforce_schema();
    let generator = DdlGenerator::new(SqlDialect::Sqlite);
    let ddl = generator.generate_schema(&schema);

    println!("Generated DDL:\n{}", ddl);

    // Verify we can create the database
    let conn = Connection::open_in_memory().expect("Failed to open in-memory database");

    for statement in ddl.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            conn.execute(trimmed, [])
                .expect(&format!("Failed to execute DDL: {}", trimmed));
        }
    }

    // Verify tables exist
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    println!("Created tables: {:?}", tables);

    assert!(tables.contains(&"account".to_string()));
    assert!(tables.contains(&"contact".to_string()));
    assert!(tables.contains(&"opportunity".to_string()));
    assert!(tables.contains(&"junction__c".to_string()));
}
