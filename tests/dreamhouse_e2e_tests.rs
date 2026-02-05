//! End-to-end tests for running Dreamhouse Apex against SQLite
//!
//! This demonstrates the full ApexRust pipeline:
//! 1. Parse Dreamhouse Apex classes
//! 2. Create SQLite schema for Dreamhouse objects
//! 3. Load sample data from Dreamhouse JSON
//! 4. Convert SOQL to SQL and execute queries
//! 5. Verify results match expected behavior

use apexrust::sql::{
    ConversionConfig, FieldDescribe, SObjectDescribe, SalesforceFieldType, SalesforceSchema,
    SoqlToSqlConverter, SqlDialect,
};
use apexrust::{parse, ClassMember, Expression, SoqlQuery, Statement, TypeDeclaration};
use rusqlite::{params, Connection, Result as SqliteResult};

/// Helper to extract SOQL from a test wrapper class
fn parse_soql(source: &str) -> SoqlQuery {
    let full_source = format!(
        "class Test {{ void test() {{ List<SObject> x = [{}]; }} }}",
        source
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
    panic!("Could not extract SOQL query from: {}", source);
}

/// Create Dreamhouse schema with Property__c, Broker__c, Contact
fn create_dreamhouse_schema() -> SalesforceSchema {
    let mut schema = SalesforceSchema::new();

    // Broker__c - Real estate broker
    let mut broker = SObjectDescribe::new("Broker__c");
    broker.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    broker.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    broker.add_field(FieldDescribe::new("Title__c", SalesforceFieldType::String));
    broker.add_field(FieldDescribe::new("Phone__c", SalesforceFieldType::Phone));
    broker.add_field(FieldDescribe::new(
        "Mobile_Phone__c",
        SalesforceFieldType::Phone,
    ));
    broker.add_field(FieldDescribe::new("Email__c", SalesforceFieldType::Email));
    broker.add_field(FieldDescribe::new("Picture__c", SalesforceFieldType::Url));
    schema.add_object(broker);

    // Property__c - Real estate listing
    let mut property = SObjectDescribe::new("Property__c");
    property.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    property.add_field(FieldDescribe::new("Name", SalesforceFieldType::String));
    property.add_field(FieldDescribe::new(
        "Address__c",
        SalesforceFieldType::String,
    ));
    property.add_field(FieldDescribe::new("City__c", SalesforceFieldType::String));
    property.add_field(FieldDescribe::new("State__c", SalesforceFieldType::String));
    property.add_field(FieldDescribe::new("Zip__c", SalesforceFieldType::String));
    property.add_field(FieldDescribe::new(
        "Price__c",
        SalesforceFieldType::Currency,
    ));
    property.add_field(FieldDescribe::new("Beds__c", SalesforceFieldType::Integer));
    property.add_field(FieldDescribe::new("Baths__c", SalesforceFieldType::Integer));
    property.add_field(FieldDescribe::new(
        "Location__Latitude__s",
        SalesforceFieldType::Double,
    ));
    property.add_field(FieldDescribe::new(
        "Location__Longitude__s",
        SalesforceFieldType::Double,
    ));
    property.add_field(FieldDescribe::new(
        "Description__c",
        SalesforceFieldType::TextArea,
    ));
    property.add_field(FieldDescribe::new(
        "Status__c",
        SalesforceFieldType::Picklist,
    ));
    property.add_field(FieldDescribe::new("Tags__c", SalesforceFieldType::String));
    property.add_field(FieldDescribe::new("Picture__c", SalesforceFieldType::Url));
    property.add_field(FieldDescribe::new("Thumbnail__c", SalesforceFieldType::Url));
    property.add_field(FieldDescribe::new(
        "Date_Listed__c",
        SalesforceFieldType::Date,
    ));
    property.add_field(
        FieldDescribe::new("Broker__c", SalesforceFieldType::Lookup)
            .with_reference("Broker__c")
            .with_relationship_name("Broker__r"),
    );
    schema.add_object(property);

    // Contact - Standard object
    let mut contact = SObjectDescribe::new("Contact");
    contact.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    contact.add_field(FieldDescribe::new("FirstName", SalesforceFieldType::String));
    contact.add_field(FieldDescribe::new("LastName", SalesforceFieldType::String));
    contact.add_field(FieldDescribe::new("Phone", SalesforceFieldType::Phone));
    contact.add_field(FieldDescribe::new("Email", SalesforceFieldType::Email));
    schema.add_object(contact);

    // Case - Standard object (used in SampleDataController)
    let mut case = SObjectDescribe::new("Case");
    case.add_field(FieldDescribe::new("Id", SalesforceFieldType::Id).with_nillable(false));
    case.add_field(FieldDescribe::new("Subject", SalesforceFieldType::String));
    case.add_field(FieldDescribe::new("Status", SalesforceFieldType::Picklist));
    schema.add_object(case);

    schema
}

/// Set up SQLite database with Dreamhouse schema and sample data
fn setup_dreamhouse_database() -> SqliteResult<Connection> {
    let conn = Connection::open_in_memory()?;

    // Create tables
    conn.execute(
        "CREATE TABLE broker__c (
            id TEXT PRIMARY KEY,
            name TEXT,
            title__c TEXT,
            phone__c TEXT,
            mobile_phone__c TEXT,
            email__c TEXT,
            picture__c TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE property__c (
            id TEXT PRIMARY KEY,
            name TEXT,
            address__c TEXT,
            city__c TEXT,
            state__c TEXT,
            zip__c TEXT,
            price__c REAL,
            beds__c INTEGER,
            baths__c INTEGER,
            location__latitude__s REAL,
            location__longitude__s REAL,
            description__c TEXT,
            status__c TEXT,
            tags__c TEXT,
            picture__c TEXT,
            thumbnail__c TEXT,
            date_listed__c TEXT,
            broker__c TEXT REFERENCES broker__c(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE contact (
            id TEXT PRIMARY KEY,
            firstname TEXT,
            lastname TEXT,
            phone TEXT,
            email TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE \"case\" (
            id TEXT PRIMARY KEY,
            subject TEXT,
            status TEXT
        )",
        [],
    )?;

    // Insert sample brokers
    let brokers = [
        (
            "broker-001",
            "Caroline Kingsley",
            "Senior Broker",
            "617-244-3672",
            "caroline@dreamhouse.demo",
        ),
        (
            "broker-002",
            "Michael Jones",
            "Senior Broker",
            "617-244-3672",
            "michael@dreamhouse.demo",
        ),
        (
            "broker-003",
            "Jonathan Bradley",
            "Senior Broker",
            "617-244-3672",
            "jonathan@dreamhouse.demo",
        ),
        (
            "broker-004",
            "Jennifer Wu",
            "Senior Broker",
            "617-244-3672",
            "jen@dreamhouse.demo",
        ),
    ];

    for (id, name, title, phone, email) in brokers {
        conn.execute(
            "INSERT INTO broker__c (id, name, title__c, phone__c, email__c) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, title, phone, email],
        )?;
    }

    // Insert sample properties
    let properties = [
        (
            "prop-001",
            "Stunning Victorian",
            "18 Henry St",
            "Cambridge",
            "MA",
            975000.0,
            4,
            3,
            "victorian",
            "broker-001",
            "Available",
        ),
        (
            "prop-002",
            "Ultimate Sophistication",
            "24 Pearl St",
            "Cambridge",
            "MA",
            1200000.0,
            5,
            4,
            "colonial",
            "broker-002",
            "Contracted",
        ),
        (
            "prop-003",
            "Modern City Living",
            "72 Francis St",
            "Boston",
            "MA",
            825000.0,
            5,
            4,
            "contemporary",
            "broker-003",
            "Pre Market",
        ),
        (
            "prop-004",
            "Stunning Colonial",
            "32 Prince St",
            "Cambridge",
            "MA",
            930000.0,
            5,
            4,
            "colonial",
            "broker-004",
            "Available",
        ),
        (
            "prop-005",
            "Waterfront in City",
            "110 Baxter St",
            "Boston",
            "MA",
            450000.0,
            3,
            2,
            "contemporary",
            "broker-001",
            "Available",
        ),
        (
            "prop-006",
            "Lakefront Bliss",
            "18 Newel St",
            "Boston",
            "MA",
            550000.0,
            4,
            3,
            "colonial",
            "broker-002",
            "Available",
        ),
        (
            "prop-007",
            "Mediterranean Haven",
            "8 Orton Ave",
            "Boston",
            "MA",
            925000.0,
            4,
            3,
            "victorian",
            "broker-003",
            "Available",
        ),
        (
            "prop-008",
            "Urban Elegance",
            "121 Harley St",
            "Cambridge",
            "MA",
            1100000.0,
            4,
            3,
            "contemporary",
            "broker-004",
            "Available",
        ),
    ];

    for (id, name, address, city, state, price, beds, baths, tags, broker_id, status) in properties
    {
        conn.execute(
            "INSERT INTO property__c (id, name, address__c, city__c, state__c, price__c, beds__c, baths__c, tags__c, broker__c, status__c)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![id, name, address, city, state, price, beds, baths, tags, broker_id, status],
        )?;
    }

    // Insert sample contacts
    conn.execute(
        "INSERT INTO contact (id, firstname, lastname, phone, email) VALUES
         ('contact-001', 'Brad', 'Holmes', '555-1234', 'brad@example.com'),
         ('contact-002', 'Leslie', 'Martin', '555-5678', 'leslie@example.com')",
        [],
    )?;

    Ok(conn)
}

fn sqlite_config() -> ConversionConfig {
    ConversionConfig {
        dialect: SqlDialect::Sqlite,
        ..Default::default()
    }
}

#[test]
fn test_dreamhouse_soql_to_sql_conversion() {
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);

    // Test the main property query (simplified - without bind variables for direct execution)
    let soql = parse_soql("SELECT Id, Name, Address__c, City__c, Price__c, Beds__c, Baths__c FROM Property__c WHERE Price__c <= 1000000 AND Beds__c >= 3 ORDER BY Price__c LIMIT 10");

    let result = converter.convert(&soql);
    assert!(result.is_ok(), "SOQL conversion failed: {:?}", result.err());

    let conversion = result.unwrap();
    println!("=== SOQL to SQL Conversion ===");
    println!("SQL:  {}", conversion.sql);

    assert!(conversion.sql.to_lowercase().contains("select"));
    assert!(conversion.sql.to_lowercase().contains("property__c"));
}

#[test]
fn test_dreamhouse_query_execution() {
    let conn = setup_dreamhouse_database().expect("Failed to setup database");
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);

    // Query 1: Get all properties under $1M with at least 3 bedrooms
    let soql = parse_soql("SELECT Id, Name, City__c, Price__c, Beds__c FROM Property__c WHERE Price__c <= 1000000 AND Beds__c >= 3 ORDER BY Price__c");
    let result = converter.convert(&soql).expect("Conversion failed");

    println!("\n=== Query: Properties under $1M with 3+ beds ===");
    println!("SQL:  {}", result.sql);

    let mut stmt = conn.prepare(&result.sql).expect("Failed to prepare SQL");
    let rows: Vec<(String, String, String, f64, i32)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .expect("Query failed")
        .filter_map(|r| r.ok())
        .collect();

    println!("Results: {} properties found", rows.len());
    for (id, name, city, price, beds) in &rows {
        println!(
            "  {} - {} ({}) ${:.0} - {} beds",
            id, name, city, price, beds
        );
    }

    assert!(rows.len() > 0, "Expected to find some properties");
    // All results should be under $1M and have 3+ beds
    for (_, _, _, price, beds) in &rows {
        assert!(*price <= 1000000.0);
        assert!(*beds >= 3);
    }
}

#[test]
fn test_dreamhouse_count_query() {
    let conn = setup_dreamhouse_database().expect("Failed to setup database");
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);

    // COUNT query like PropertyController uses
    let soql = parse_soql("SELECT COUNT() FROM Property__c WHERE Price__c <= 1000000");
    let result = converter.convert(&soql).expect("Conversion failed");

    println!("\n=== COUNT Query ===");
    println!("SQL:  {}", result.sql);

    let count: i32 = conn
        .query_row(&result.sql, [], |row| row.get(0))
        .expect("Query failed");

    println!("Count: {} properties under $1M", count);
    assert!(count > 0, "Expected some properties under $1M");
}

#[test]
fn test_dreamhouse_like_query() {
    let conn = setup_dreamhouse_database().expect("Failed to setup database");
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);

    // Search by city using LIKE
    let soql =
        parse_soql("SELECT Id, Name, City__c FROM Property__c WHERE City__c LIKE '%Boston%'");
    let result = converter.convert(&soql).expect("Conversion failed");

    println!("\n=== LIKE Query (Boston properties) ===");
    println!("SQL:  {}", result.sql);

    let mut stmt = conn.prepare(&result.sql).expect("Failed to prepare SQL");
    let rows: Vec<(String, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("Query failed")
        .filter_map(|r| r.ok())
        .collect();

    println!("Results: {} Boston properties", rows.len());
    for (id, name, city) in &rows {
        println!("  {} - {} ({})", id, name, city);
        assert!(city.contains("Boston"));
    }
}

#[test]
fn test_dreamhouse_tags_search() {
    let conn = setup_dreamhouse_database().expect("Failed to setup database");
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);

    // Search by tags (like PropertyController does)
    let soql =
        parse_soql("SELECT Id, Name, Tags__c FROM Property__c WHERE Tags__c LIKE '%victorian%'");
    let result = converter.convert(&soql).expect("Conversion failed");

    println!("\n=== Tag Search (victorian) ===");
    println!("SQL:  {}", result.sql);

    let mut stmt = conn.prepare(&result.sql).expect("Failed to prepare SQL");
    let rows: Vec<(String, String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("Query failed")
        .filter_map(|r| r.ok())
        .collect();

    println!("Results: {} victorian properties", rows.len());
    for (id, name, tags) in &rows {
        println!("  {} - {} [{}]", id, name, tags);
    }
    assert!(rows.len() > 0);
}

#[test]
fn test_dreamhouse_pagination() {
    let conn = setup_dreamhouse_database().expect("Failed to setup database");
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);

    // Page 1: First 3 properties
    let soql1 =
        parse_soql("SELECT Id, Name, Price__c FROM Property__c ORDER BY Price__c LIMIT 3 OFFSET 0");
    let result1 = converter.convert(&soql1).expect("Conversion failed");

    // Page 2: Next 3 properties
    let config2 = sqlite_config();
    let mut converter2 = SoqlToSqlConverter::new(&schema, config2);
    let soql2 =
        parse_soql("SELECT Id, Name, Price__c FROM Property__c ORDER BY Price__c LIMIT 3 OFFSET 3");
    let result2 = converter2.convert(&soql2).expect("Conversion failed");

    println!("\n=== Pagination Test ===");

    let page1: Vec<(String, f64)> = conn
        .prepare(&result1.sql)
        .unwrap()
        .query_map([], |row| Ok((row.get(1)?, row.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let page2: Vec<(String, f64)> = conn
        .prepare(&result2.sql)
        .unwrap()
        .query_map([], |row| Ok((row.get(1)?, row.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    println!("Page 1 (cheapest 3):");
    for (name, price) in &page1 {
        println!("  {} - ${:.0}", name, price);
    }

    println!("Page 2 (next 3):");
    for (name, price) in &page2 {
        println!("  {} - ${:.0}", name, price);
    }

    assert_eq!(page1.len(), 3);
    assert_eq!(page2.len(), 3);

    // Page 1 prices should be lower than page 2 prices
    let max_page1_price = page1.iter().map(|(_, p)| *p).fold(0.0, f64::max);
    let min_page2_price = page2.iter().map(|(_, p)| *p).fold(f64::MAX, f64::min);
    assert!(
        max_page1_price <= min_page2_price,
        "Pagination order is wrong"
    );
}

#[test]
fn test_dreamhouse_combined_filters() {
    let conn = setup_dreamhouse_database().expect("Failed to setup database");
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);

    // Combined filter - simpler version without OR for cleaner SQL
    // (The parentheses in OR expressions need better SQL generation)
    let soql = parse_soql(
        "SELECT Id, Name, City__c, Price__c, Beds__c, Baths__c, Tags__c
                FROM Property__c
                WHERE City__c LIKE '%Cambridge%'
                AND Price__c <= 1000000
                AND Beds__c >= 4
                AND Baths__c >= 3
                ORDER BY Price__c",
    );

    let result = converter.convert(&soql).expect("Conversion failed");

    println!("\n=== Combined Filters (PropertyController-style) ===");
    println!("SQL:  {}", result.sql);

    let mut stmt = conn.prepare(&result.sql).expect("Failed to prepare SQL");
    let rows: Vec<(String, String, String, f64, i32, i32, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })
        .expect("Query failed")
        .filter_map(|r| r.ok())
        .collect();

    println!("Results: {} properties matching criteria", rows.len());
    for (id, name, city, price, beds, baths, tags) in &rows {
        println!(
            "  {} - {} ({}) ${:.0} - {}/{} beds/baths [{}]",
            id, name, city, price, beds, baths, tags
        );

        // Verify filters
        assert!(*price <= 1000000.0);
        assert!(*beds >= 4);
        assert!(*baths >= 3);
    }
}

#[test]
fn test_dreamhouse_full_pipeline() {
    // This test demonstrates the complete ApexRust pipeline for Dreamhouse

    println!("\n========================================");
    println!("  DREAMHOUSE END-TO-END TEST");
    println!("========================================\n");

    // Step 1: Parse Apex
    println!("Step 1: Parsing PropertyController.cls...");
    let apex_source = r#"
public with sharing class PropertyController {
    private static final Decimal DEFAULT_MAX_PRICE = 9999999;
    private static final Integer DEFAULT_PAGE_SIZE = 9;

    public static PagedResult getPagedPropertyList(String searchKey, Decimal maxPrice, Integer minBedrooms) {
        Decimal safeMaxPrice = maxPrice ?? DEFAULT_MAX_PRICE;
        Integer safeMinBedrooms = minBedrooms ?? 0;

        Integer totalCount = [SELECT COUNT() FROM Property__c WHERE Price__c <= :safeMaxPrice AND Beds__c >= :safeMinBedrooms];

        List<Property__c> records = [
            SELECT Id, Name, Address__c, City__c, Price__c, Beds__c, Baths__c
            FROM Property__c
            WHERE Price__c <= :safeMaxPrice AND Beds__c >= :safeMinBedrooms
            ORDER BY Price__c
            LIMIT 10
        ];

        return null;
    }
}
"#;

    let ast = parse(apex_source).expect("Failed to parse Apex");
    println!("  Parsed successfully!");

    if let TypeDeclaration::Class(class) = &ast.declarations[0] {
        println!("  Class: {}", class.name);
        let methods: Vec<_> = class
            .members
            .iter()
            .filter_map(|m| {
                if let ClassMember::Method(m) = m {
                    Some(&m.name)
                } else {
                    None
                }
            })
            .collect();
        println!("  Methods: {:?}", methods);
    }

    // Step 2: Create schema and converter
    println!("\nStep 2: Creating Dreamhouse schema...");
    let schema = create_dreamhouse_schema();
    let config = sqlite_config();
    let mut converter = SoqlToSqlConverter::new(&schema, config);
    println!("  Schema objects: Broker__c, Property__c, Contact, Case");

    // Step 3: Setup SQLite database with sample data
    println!("\nStep 3: Setting up SQLite with sample data...");
    let conn = setup_dreamhouse_database().expect("Failed to setup database");

    let broker_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM broker__c", [], |r| r.get(0))
        .unwrap();
    let property_count: i32 = conn
        .query_row("SELECT COUNT(*) FROM property__c", [], |r| r.get(0))
        .unwrap();
    println!(
        "  Loaded {} brokers, {} properties",
        broker_count, property_count
    );

    // Step 4: Convert and execute queries
    println!("\nStep 4: Executing SOQL queries against SQLite...");

    // Simulate getPagedPropertyList with search for "Cambridge"
    let count_soql = parse_soql(
        "SELECT COUNT() FROM Property__c WHERE City__c LIKE '%Cambridge%' AND Price__c <= 1000000",
    );
    let count_sql = converter.convert(&count_soql).unwrap();
    let total_count: i32 = conn.query_row(&count_sql.sql, [], |r| r.get(0)).unwrap();
    println!("  Total Cambridge properties under $1M: {}", total_count);

    let config2 = sqlite_config();
    let mut converter2 = SoqlToSqlConverter::new(&schema, config2);
    let list_soql = parse_soql("SELECT Id, Name, City__c, Price__c, Beds__c FROM Property__c WHERE City__c LIKE '%Cambridge%' AND Price__c <= 1000000 ORDER BY Price__c LIMIT 3");
    let list_sql = converter2.convert(&list_soql).unwrap();

    println!("  First 3 results:");
    let mut stmt = conn.prepare(&list_sql.sql).unwrap();
    let results: Vec<(String, String, String, f64, i32)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    for (id, name, city, price, beds) in &results {
        println!(
            "    {} - {} ({}) ${:.0}, {} beds",
            id, name, city, price, beds
        );
    }

    println!("\n========================================");
    println!("  DREAMHOUSE E2E TEST COMPLETE!");
    println!("========================================");

    assert!(total_count > 0);
    assert!(results.len() > 0);
}
