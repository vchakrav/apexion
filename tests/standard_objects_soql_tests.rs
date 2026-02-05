//! Comprehensive SOQL tests against standard Salesforce objects
//!
//! These tests verify SOQL to SQL conversion using a realistic Sales Cloud schema
//! with Account, Contact, Opportunity, Lead, Case, Task, Event, Campaign, etc.

use apexrust::sql::{
    create_sales_cloud_schema, ConversionConfig, DdlGenerator, SoqlToSqlConverter, SqlDialect,
};
use apexrust::{parse, ClassMember, Expression, SoqlQuery, Statement, TypeDeclaration};
use rusqlite::{Connection, Result as SqliteResult};

/// Helper to extract SOQL from a test wrapper class
fn extract_soql(soql_source: &str) -> SoqlQuery {
    let full_source = format!(
        "class Test {{ void test() {{ List<SObject> x = [{}]; }} }}",
        soql_source
    );
    let cu = parse(&full_source).expect("Parse failed");
    if let TypeDeclaration::Class(class) = &cu.declarations[0] {
        if let ClassMember::Method(method) = &class.members[0] {
            if let Some(block) = &method.body {
                if let Statement::LocalVariable(lv) = &block.statements[0] {
                    if let Some(Expression::Soql(soql)) = &lv.declarators[0].initializer {
                        return (**soql).clone();
                    }
                }
            }
        }
    }
    panic!("Could not extract SOQL query");
}

/// Set up SQLite database with the standard Sales Cloud schema
fn setup_sales_cloud_db() -> SqliteResult<Connection> {
    let schema = create_sales_cloud_schema();
    let conn = Connection::open_in_memory()?;

    // Generate and execute DDL
    let generator = DdlGenerator::new(SqlDialect::Sqlite);
    let ddl = generator.generate_schema(&schema);

    for statement in ddl.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            conn.execute(trimmed, [])?;
        }
    }

    // Insert sample data
    insert_sample_sales_data(&conn)?;

    Ok(conn)
}

/// Insert realistic sample sales data
fn insert_sample_sales_data(conn: &Connection) -> SqliteResult<()> {
    // Users
    conn.execute(
        "INSERT INTO \"user\" (id, username, first_name, last_name, name, email, is_active, title)
         VALUES ('005000000000001', 'jsmith@test.com', 'John', 'Smith', 'John Smith', 'jsmith@test.com', 1, 'Sales Rep')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"user\" (id, username, first_name, last_name, name, email, is_active, title)
         VALUES ('005000000000002', 'mjones@test.com', 'Mary', 'Jones', 'Mary Jones', 'mjones@test.com', 1, 'Sales Manager')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"user\" (id, username, first_name, last_name, name, email, is_active, title)
         VALUES ('005000000000003', 'bwilson@test.com', 'Bob', 'Wilson', 'Bob Wilson', 'bwilson@test.com', 1, 'Account Executive')",
        [],
    )?;

    // Accounts
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, type, annual_revenue, number_of_employees, rating, billing_city, billing_state, billing_country, phone, website, owner_id)
         VALUES ('001000000000001', 'Acme Corporation', 'Technology', 'Customer', 5000000.00, 500, 'Hot', 'San Francisco', 'CA', 'USA', '415-555-1234', 'www.acme.com', '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, type, annual_revenue, number_of_employees, rating, billing_city, billing_state, billing_country, phone, website, owner_id)
         VALUES ('001000000000002', 'Global Tech Inc', 'Technology', 'Customer', 10000000.00, 1000, 'Hot', 'New York', 'NY', 'USA', '212-555-5678', 'www.globaltech.com', '005000000000002')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, type, annual_revenue, number_of_employees, rating, billing_city, billing_state, billing_country, phone, owner_id)
         VALUES ('001000000000003', 'Small Biz LLC', 'Retail', 'Prospect', 500000.00, 25, 'Warm', 'Chicago', 'IL', 'USA', '312-555-9012', '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, type, annual_revenue, number_of_employees, rating, billing_city, billing_state, billing_country, owner_id, parent_id)
         VALUES ('001000000000004', 'Acme West Division', 'Technology', 'Customer', 1000000.00, 100, 'Warm', 'Los Angeles', 'CA', 'USA', '005000000000001', '001000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"account\" (id, name, industry, type, annual_revenue, number_of_employees, rating, billing_city, billing_state, billing_country, owner_id)
         VALUES ('001000000000005', 'Healthcare Systems', 'Healthcare', 'Customer', 25000000.00, 2500, 'Hot', 'Boston', 'MA', 'USA', '005000000000003')",
        [],
    )?;

    // Contacts
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, phone, title, department, account_id, mailing_city, mailing_state, owner_id)
         VALUES ('003000000000001', 'Alice', 'Johnson', 'Alice Johnson', 'alice@acme.com', '415-555-1111', 'VP Sales', 'Sales', '001000000000001', 'San Francisco', 'CA', '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, phone, title, department, account_id, mailing_city, mailing_state, owner_id)
         VALUES ('003000000000002', 'Bob', 'Williams', 'Bob Williams', 'bob@acme.com', '415-555-2222', 'CTO', 'Engineering', '001000000000001', 'San Francisco', 'CA', '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, phone, title, department, account_id, mailing_city, mailing_state, owner_id)
         VALUES ('003000000000003', 'Carol', 'Davis', 'Carol Davis', 'carol@globaltech.com', '212-555-3333', 'CEO', 'Executive', '001000000000002', 'New York', 'NY', '005000000000002')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, phone, title, department, account_id, mailing_city, mailing_state, owner_id)
         VALUES ('003000000000004', 'David', 'Miller', 'David Miller', 'david@globaltech.com', '212-555-4444', 'CFO', 'Finance', '001000000000002', 'New York', 'NY', '005000000000002')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"contact\" (id, first_name, last_name, name, email, phone, title, department, account_id, mailing_city, mailing_state, owner_id)
         VALUES ('003000000000005', 'Eve', 'Brown', 'Eve Brown', 'eve@smallbiz.com', '312-555-5555', 'Owner', 'Executive', '001000000000003', 'Chicago', 'IL', '005000000000001')",
        [],
    )?;

    // Leads
    conn.execute(
        "INSERT INTO \"lead\" (id, first_name, last_name, name, email, phone, company, title, status, industry, rating, city, state, country, owner_id, is_converted)
         VALUES ('00Q000000000001', 'Frank', 'Taylor', 'Frank Taylor', 'frank@prospect.com', '555-111-1111', 'Prospect Corp', 'Director', 'New', 'Finance', 'Hot', 'Miami', 'FL', 'USA', '005000000000001', 0)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"lead\" (id, first_name, last_name, name, email, phone, company, title, status, industry, rating, city, state, country, owner_id, is_converted)
         VALUES ('00Q000000000002', 'Grace', 'Lee', 'Grace Lee', 'grace@newclient.com', '555-222-2222', 'New Client Inc', 'VP', 'Working', 'Technology', 'Warm', 'Seattle', 'WA', 'USA', '005000000000002', 0)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"lead\" (id, first_name, last_name, name, email, phone, company, title, status, industry, rating, city, state, country, owner_id, is_converted, converted_account_id, converted_contact_id)
         VALUES ('00Q000000000003', 'Henry', 'Clark', 'Henry Clark', 'henry@converted.com', '555-333-3333', 'Converted LLC', 'Manager', 'Converted', 'Retail', 'Hot', 'Denver', 'CO', 'USA', '005000000000003', 1, '001000000000003', '003000000000005')",
        [],
    )?;

    // Opportunities
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, account_id, stage_name, amount, probability, close_date, type, lead_source, is_closed, is_won, fiscal_year, fiscal_quarter, owner_id)
         VALUES ('006000000000001', 'Acme Enterprise Deal', '001000000000001', 'Closed Won', 250000.00, 100, '2024-01-15', 'New Business', 'Web', 1, 1, 2024, 1, '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, account_id, stage_name, amount, probability, close_date, type, lead_source, is_closed, is_won, fiscal_year, fiscal_quarter, owner_id)
         VALUES ('006000000000002', 'Global Tech Expansion', '001000000000002', 'Negotiation', 500000.00, 75, '2024-03-31', 'Existing Business', 'Referral', 0, 0, 2024, 1, '005000000000002')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, account_id, stage_name, amount, probability, close_date, type, lead_source, is_closed, is_won, fiscal_year, fiscal_quarter, owner_id)
         VALUES ('006000000000003', 'Small Biz Starter', '001000000000003', 'Qualification', 25000.00, 25, '2024-06-30', 'New Business', 'Trade Show', 0, 0, 2024, 2, '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, account_id, stage_name, amount, probability, close_date, type, lead_source, is_closed, is_won, fiscal_year, fiscal_quarter, owner_id)
         VALUES ('006000000000004', 'Acme Renewal', '001000000000001', 'Proposal', 150000.00, 60, '2024-04-15', 'Existing Business', 'Customer', 0, 0, 2024, 2, '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, account_id, stage_name, amount, probability, close_date, type, lead_source, is_closed, is_won, fiscal_year, fiscal_quarter, owner_id)
         VALUES ('006000000000005', 'Healthcare Big Deal', '001000000000005', 'Negotiation', 1000000.00, 80, '2024-02-28', 'New Business', 'Partner', 0, 0, 2024, 1, '005000000000003')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity\" (id, name, account_id, stage_name, amount, probability, close_date, type, lead_source, is_closed, is_won, fiscal_year, fiscal_quarter, owner_id)
         VALUES ('006000000000006', 'Lost Deal', '001000000000003', 'Closed Lost', 50000.00, 0, '2024-01-31', 'New Business', 'Web', 1, 0, 2024, 1, '005000000000001')",
        [],
    )?;

    // Opportunity Contact Roles
    conn.execute(
        "INSERT INTO \"opportunity_contact_role\" (id, opportunity_id, contact_id, role, is_primary)
         VALUES ('00K000000000001', '006000000000001', '003000000000001', 'Decision Maker', 1)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity_contact_role\" (id, opportunity_id, contact_id, role, is_primary)
         VALUES ('00K000000000002', '006000000000001', '003000000000002', 'Technical Buyer', 0)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"opportunity_contact_role\" (id, opportunity_id, contact_id, role, is_primary)
         VALUES ('00K000000000003', '006000000000002', '003000000000003', 'Executive Sponsor', 1)",
        [],
    )?;

    // Cases
    conn.execute(
        "INSERT INTO \"case\" (id, case_number, subject, status, priority, origin, type, account_id, contact_id, is_closed, is_escalated, owner_id)
         VALUES ('500000000000001', '00001001', 'Product not working', 'New', 'High', 'Email', 'Problem', '001000000000001', '003000000000001', 0, 0, '005000000000001')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"case\" (id, case_number, subject, status, priority, origin, type, account_id, contact_id, is_closed, is_escalated, owner_id)
         VALUES ('500000000000002', '00001002', 'Feature request', 'Working', 'Medium', 'Web', 'Feature Request', '001000000000002', '003000000000003', 0, 0, '005000000000002')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"case\" (id, case_number, subject, status, priority, origin, type, account_id, contact_id, is_closed, is_escalated, closed_date, owner_id)
         VALUES ('500000000000003', '00001003', 'Billing question', 'Closed', 'Low', 'Phone', 'Question', '001000000000001', '003000000000002', 1, 0, '2024-01-20', '005000000000001')",
        [],
    )?;

    // Tasks
    conn.execute(
        "INSERT INTO \"task\" (id, subject, status, priority, activity_date, what_id, who_id, owner_id, is_closed)
         VALUES ('00T000000000001', 'Follow up call', 'Not Started', 'Normal', '2024-02-15', '001000000000001', '003000000000001', '005000000000001', 0)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"task\" (id, subject, status, priority, activity_date, what_id, who_id, owner_id, is_closed)
         VALUES ('00T000000000002', 'Send proposal', 'Completed', 'High', '2024-01-10', '006000000000002', '003000000000003', '005000000000002', 1)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"task\" (id, subject, status, priority, activity_date, what_id, who_id, owner_id, is_closed)
         VALUES ('00T000000000003', 'Demo scheduled', 'In Progress', 'Normal', '2024-02-20', '006000000000003', '003000000000005', '005000000000001', 0)",
        [],
    )?;

    // Events
    conn.execute(
        "INSERT INTO \"event\" (id, subject, start_date_time, end_date_time, duration_in_minutes, location, what_id, who_id, owner_id, is_all_day_event)
         VALUES ('00U000000000001', 'Quarterly Business Review', '2024-02-15 10:00:00', '2024-02-15 12:00:00', 120, 'Acme HQ', '001000000000001', '003000000000001', '005000000000001', 0)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"event\" (id, subject, start_date_time, end_date_time, duration_in_minutes, location, what_id, who_id, owner_id, is_all_day_event)
         VALUES ('00U000000000002', 'Contract Signing', '2024-03-01 14:00:00', '2024-03-01 15:00:00', 60, 'Global Tech Office', '006000000000002', '003000000000003', '005000000000002', 0)",
        [],
    )?;

    // Campaigns
    conn.execute(
        "INSERT INTO \"campaign\" (id, name, type, status, start_date, end_date, budgeted_cost, actual_cost, expected_revenue, is_active, owner_id, number_of_leads, number_of_contacts, number_of_opportunities)
         VALUES ('701000000000001', 'Spring Product Launch', 'Product Launch', 'In Progress', '2024-03-01', '2024-05-31', 50000.00, 25000.00, 500000.00, 1, '005000000000002', 10, 5, 3)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"campaign\" (id, name, type, status, start_date, end_date, budgeted_cost, actual_cost, is_active, owner_id, number_of_leads, number_of_contacts)
         VALUES ('701000000000002', 'Trade Show 2024', 'Trade Show', 'Planned', '2024-06-15', '2024-06-17', 100000.00, 0.00, 1, '005000000000001', 0, 0)",
        [],
    )?;

    // Campaign Members
    conn.execute(
        "INSERT INTO \"campaign_member\" (id, campaign_id, contact_id, status, has_responded)
         VALUES ('00v000000000001', '701000000000001', '003000000000001', 'Sent', 1)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"campaign_member\" (id, campaign_id, lead_id, status, has_responded)
         VALUES ('00v000000000002', '701000000000001', '00Q000000000001', 'Sent', 0)",
        [],
    )?;

    // Products
    conn.execute(
        "INSERT INTO \"product2\" (id, name, product_code, family, is_active, description)
         VALUES ('01t000000000001', 'Enterprise License', 'ENT-001', 'Software', 1, 'Full enterprise software license')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"product2\" (id, name, product_code, family, is_active, description)
         VALUES ('01t000000000002', 'Professional Services', 'PS-001', 'Services', 1, 'Implementation and consulting services')",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"product2\" (id, name, product_code, family, is_active, description)
         VALUES ('01t000000000003', 'Support Package', 'SUP-001', 'Support', 1, 'Annual support and maintenance')",
        [],
    )?;

    // Pricebooks
    conn.execute(
        "INSERT INTO \"pricebook2\" (id, name, is_active, is_standard)
         VALUES ('01s000000000001', 'Standard Price Book', 1, 1)",
        [],
    )?;

    // Pricebook Entries
    conn.execute(
        "INSERT INTO \"pricebook_entry\" (id, pricebook2_id, product2_id, unit_price, is_active)
         VALUES ('01u000000000001', '01s000000000001', '01t000000000001', 10000.00, 1)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"pricebook_entry\" (id, pricebook2_id, product2_id, unit_price, is_active)
         VALUES ('01u000000000002', '01s000000000001', '01t000000000002', 200.00, 1)",
        [],
    )?;
    conn.execute(
        "INSERT INTO \"pricebook_entry\" (id, pricebook2_id, product2_id, unit_price, is_active)
         VALUES ('01u000000000003', '01s000000000001', '01t000000000003', 5000.00, 1)",
        [],
    )?;

    Ok(())
}

/// Convert SOQL to SQL and execute against SQLite
fn execute_soql(conn: &Connection, soql_source: &str) -> Result<(usize, String), String> {
    let schema = create_sales_cloud_schema();
    let soql = extract_soql(soql_source);

    let config = ConversionConfig {
        dialect: SqlDialect::Sqlite,
        ..Default::default()
    };

    let mut converter = SoqlToSqlConverter::new(&schema, config);
    let result = converter
        .convert(&soql)
        .map_err(|e| format!("Conversion error: {}", e))?;

    let sql = &result.sql;

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("SQL prepare error: {}\nSQL: {}", e, sql))?;

    let rows: Vec<_> = stmt
        .query_map([], |_row| Ok(()))
        .map_err(|e| format!("SQL execute error: {}\nSQL: {}", e, sql))?
        .collect();

    Ok((rows.len(), sql.clone()))
}

// =============================================================================
// Account Queries
// =============================================================================

#[test]
fn test_account_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(&conn, "SELECT Id, Name FROM Account").unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_account_with_all_fields() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Industry, Type, AnnualRevenue, NumberOfEmployees, Rating, Phone, Website, BillingCity, BillingState, BillingCountry FROM Account",
    ).unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_account_filter_by_industry() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name FROM Account WHERE Industry = 'Technology'",
    )
    .unwrap();
    assert_eq!(count, 3); // Acme, Global Tech, Acme West
}

#[test]
fn test_account_filter_by_revenue() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, AnnualRevenue FROM Account WHERE AnnualRevenue > 1000000",
    )
    .unwrap();
    assert_eq!(count, 3); // Acme, Global Tech, Healthcare
}

#[test]
fn test_account_filter_by_rating() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) =
        execute_soql(&conn, "SELECT Id, Name FROM Account WHERE Rating = 'Hot'").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_account_with_child_contacts() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, (SELECT Id, Name, Email FROM Contacts) FROM Account",
    )
    .unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_account_with_child_opportunities() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, (SELECT Id, Name, Amount, StageName FROM Opportunities) FROM Account",
    )
    .unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_account_parent_child_hierarchy() {
    let conn = setup_sales_cloud_db().unwrap();
    // Use explicit check for non-null parent
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Parent.Name FROM Account WHERE Parent.Name != ''",
    )
    .unwrap();
    assert_eq!(count, 1); // Acme West Division
}

#[test]
fn test_account_aggregate_by_industry() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Industry, COUNT(Id) cnt, SUM(AnnualRevenue) total FROM Account GROUP BY Industry",
    )
    .unwrap();
    assert_eq!(count, 3); // Technology, Retail, Healthcare
}

#[test]
fn test_account_aggregate_by_state() {
    let conn = setup_sales_cloud_db().unwrap();
    // Note: ORDER BY alias doesn't work directly, use the expression
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT BillingState, COUNT(Id) cnt FROM Account GROUP BY BillingState",
    )
    .unwrap();
    assert!(count >= 1);
}

// =============================================================================
// Contact Queries
// =============================================================================

#[test]
fn test_contact_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) =
        execute_soql(&conn, "SELECT Id, FirstName, LastName, Email FROM Contact").unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_contact_with_account() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Email, Account.Name, Account.Industry FROM Contact",
    )
    .unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_contact_filter_by_account_industry() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Email FROM Contact WHERE Account.Industry = 'Technology'",
    )
    .unwrap();
    assert_eq!(count, 4); // Acme + Global Tech contacts
}

#[test]
fn test_contact_filter_by_title() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Title FROM Contact WHERE Title LIKE '%VP%'",
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_contact_aggregate_by_department() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Department, COUNT(Id) cnt FROM Contact GROUP BY Department",
    )
    .unwrap();
    assert!(count >= 1);
}

// =============================================================================
// Lead Queries
// =============================================================================

#[test]
fn test_lead_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) =
        execute_soql(&conn, "SELECT Id, Name, Company, Status, Rating FROM Lead").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_lead_filter_unconverted() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Company FROM Lead WHERE IsConverted = false",
    )
    .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_lead_filter_by_status() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) =
        execute_soql(&conn, "SELECT Id, Name FROM Lead WHERE Status = 'New'").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_lead_aggregate_by_status() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Status, COUNT(Id) cnt FROM Lead GROUP BY Status",
    )
    .unwrap();
    assert_eq!(count, 3); // New, Working, Converted
}

// =============================================================================
// Opportunity Queries
// =============================================================================

#[test]
fn test_opportunity_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Amount, StageName, CloseDate FROM Opportunity",
    )
    .unwrap();
    assert_eq!(count, 6);
}

#[test]
fn test_opportunity_with_account() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Amount, Account.Name, Account.Industry FROM Opportunity",
    )
    .unwrap();
    assert_eq!(count, 6);
}

#[test]
fn test_opportunity_open_only() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Amount, StageName FROM Opportunity WHERE IsClosed = false",
    )
    .unwrap();
    assert_eq!(count, 4);
}

#[test]
fn test_opportunity_won_only() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Amount FROM Opportunity WHERE IsWon = true",
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_opportunity_pipeline_by_stage() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT StageName, COUNT(Id) cnt, SUM(Amount) total FROM Opportunity WHERE IsClosed = false GROUP BY StageName",
    )
    .unwrap();
    assert!(count >= 1);
}

#[test]
fn test_opportunity_total_pipeline() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT SUM(Amount) total, AVG(Amount) average, COUNT(Id) cnt FROM Opportunity WHERE IsClosed = false",
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_opportunity_by_fiscal_quarter() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT FiscalQuarter, SUM(Amount) total FROM Opportunity GROUP BY FiscalQuarter",
    )
    .unwrap();
    assert!(count >= 1);
}

#[test]
fn test_opportunity_high_value() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Amount, Account.Name FROM Opportunity WHERE Amount > 100000 ORDER BY Amount DESC",
    )
    .unwrap();
    assert_eq!(count, 4);
}

#[test]
fn test_opportunity_with_contact_roles() {
    let conn = setup_sales_cloud_db().unwrap();
    // Note: Relationship traversal in subqueries (Contact.Name) not yet supported
    // Use simple fields for now
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, (SELECT ContactId, Role, IsPrimary FROM OpportunityContactRoles) FROM Opportunity",
    )
    .unwrap();
    assert_eq!(count, 6);
}

// =============================================================================
// Case Queries
// =============================================================================

#[test]
fn test_case_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, CaseNumber, Subject, Status, Priority FROM Case",
    )
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_case_with_account_contact() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, CaseNumber, Subject, Account.Name, Contact.Name, Contact.Email FROM Case",
    )
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_case_open_high_priority() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, CaseNumber, Subject FROM Case WHERE IsClosed = false AND Priority = 'High'",
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_case_aggregate_by_status() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Status, COUNT(Id) cnt FROM Case GROUP BY Status",
    )
    .unwrap();
    assert!(count >= 1);
}

// =============================================================================
// Task Queries
// =============================================================================

#[test]
fn test_task_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Subject, Status, Priority, ActivityDate FROM Task",
    )
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_task_open_only() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Subject, Status FROM Task WHERE IsClosed = false",
    )
    .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_task_aggregate_by_status() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Status, COUNT(Id) cnt FROM Task GROUP BY Status",
    )
    .unwrap();
    assert!(count >= 1);
}

// =============================================================================
// Event Queries
// =============================================================================

#[test]
fn test_event_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Subject, StartDateTime, EndDateTime, Location FROM Event",
    )
    .unwrap();
    assert_eq!(count, 2);
}

// =============================================================================
// Campaign Queries
// =============================================================================

#[test]
fn test_campaign_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Type, Status, BudgetedCost, ActualCost FROM Campaign",
    )
    .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_campaign_active_only() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) =
        execute_soql(&conn, "SELECT Id, Name FROM Campaign WHERE IsActive = true").unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_campaign_with_members() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, (SELECT Id, Status, HasResponded FROM CampaignMembers) FROM Campaign",
    )
    .unwrap();
    assert_eq!(count, 2);
}

// =============================================================================
// Product & Pricebook Queries
// =============================================================================

#[test]
fn test_product_simple_select() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, ProductCode, Family, IsActive FROM Product2",
    )
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_product_by_family() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Family, COUNT(Id) cnt FROM Product2 GROUP BY Family",
    )
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_pricebook_entry_with_product() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, UnitPrice, Product2.Name, Product2.ProductCode FROM PricebookEntry",
    )
    .unwrap();
    assert_eq!(count, 3);
}

// =============================================================================
// Complex Multi-Object Queries
// =============================================================================

#[test]
fn test_complex_sales_pipeline_report() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Account.Name, Account.Industry, StageName, SUM(Amount) total, COUNT(Id) cnt FROM Opportunity WHERE IsClosed = false GROUP BY Account.Name, Account.Industry, StageName",
    )
    .unwrap();
    assert!(count >= 1);
}

#[test]
fn test_complex_account_with_multiple_children() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Industry, (SELECT Name, Email FROM Contacts), (SELECT Name, Amount FROM Opportunities WHERE IsClosed = false) FROM Account WHERE Industry = 'Technology'",
    )
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_order_by_with_nulls() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name, Website FROM Account ORDER BY Website NULLS LAST",
    )
    .unwrap();
    assert_eq!(count, 5);
}

#[test]
fn test_limit_offset() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name FROM Account ORDER BY Name LIMIT 3 OFFSET 1",
    )
    .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_in_clause() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name FROM Account WHERE Industry IN ('Technology', 'Healthcare')",
    )
    .unwrap();
    assert_eq!(count, 4); // 3 Tech + 1 Healthcare
}

#[test]
fn test_not_in_clause() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name FROM Account WHERE Industry NOT IN ('Technology')",
    )
    .unwrap();
    assert_eq!(count, 2); // Retail + Healthcare
}

#[test]
fn test_like_wildcard() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name FROM Account WHERE Name LIKE 'Acme%'",
    )
    .unwrap();
    assert_eq!(count, 2); // Acme Corporation, Acme West Division
}

#[test]
fn test_nested_or_and() {
    let conn = setup_sales_cloud_db().unwrap();
    let (count, _sql) = execute_soql(
        &conn,
        "SELECT Id, Name FROM Account WHERE (Industry = 'Technology' AND Rating = 'Hot') OR (Industry = 'Healthcare')",
    )
    .unwrap();
    assert_eq!(count, 3); // 2 Hot Tech + 1 Healthcare
}

// =============================================================================
// DDL Generation Test
// =============================================================================

#[test]
fn test_ddl_generates_all_tables() {
    let schema = create_sales_cloud_schema();
    let generator = DdlGenerator::new(SqlDialect::Sqlite);
    let ddl = generator.generate_schema(&schema);

    // Verify DDL contains all expected tables
    assert!(ddl.contains("CREATE TABLE \"user\""));
    assert!(ddl.contains("CREATE TABLE \"account\""));
    assert!(ddl.contains("CREATE TABLE \"contact\""));
    assert!(ddl.contains("CREATE TABLE \"lead\""));
    assert!(ddl.contains("CREATE TABLE \"opportunity\""));
    assert!(ddl.contains("CREATE TABLE \"case\""));
    assert!(ddl.contains("CREATE TABLE \"task\""));
    assert!(ddl.contains("CREATE TABLE \"event\""));
    assert!(ddl.contains("CREATE TABLE \"campaign\""));
    assert!(ddl.contains("CREATE TABLE \"product2\""));
    assert!(ddl.contains("CREATE TABLE \"pricebook2\""));
    assert!(ddl.contains("CREATE TABLE \"pricebook_entry\""));

    println!("Generated DDL length: {} chars", ddl.len());
}

#[test]
fn test_print_sample_queries() {
    let conn = setup_sales_cloud_db().unwrap();
    let schema = create_sales_cloud_schema();

    let queries = vec![
        "SELECT Id, Name, Industry, AnnualRevenue FROM Account",
        "SELECT Id, Name, Email, Account.Name FROM Contact",
        "SELECT Id, Name, Amount, StageName, Account.Name FROM Opportunity WHERE IsClosed = false",
        "SELECT Id, Name, (SELECT Name, Email FROM Contacts) FROM Account WHERE Industry = 'Technology'",
        "SELECT StageName, COUNT(Id) cnt, SUM(Amount) total FROM Opportunity GROUP BY StageName",
    ];

    println!("\n=== Sample SOQL to SQL Conversions ===\n");

    for soql_source in queries {
        let soql = extract_soql(soql_source);
        let config = ConversionConfig {
            dialect: SqlDialect::Sqlite,
            ..Default::default()
        };
        let mut converter = SoqlToSqlConverter::new(&schema, config);
        let result = converter.convert(&soql).unwrap();

        println!("SOQL: {}", soql_source);
        println!("SQL:  {}", result.sql.replace('\n', " "));
        println!();
    }
}
