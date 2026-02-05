use apexrust::parse;
use std::fs;
use std::path::Path;

/// Helper to parse a file and return detailed error info
fn parse_file(path: &str) -> Result<(), String> {
    let full_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("apex_files")
        .join(path);

    let source = fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    match parse(&source) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Parse error in {}: {:?}", path, e)),
    }
}

// ==================== FFLib Enterprise Patterns Tests ====================

#[test]
fn test_parse_fflib_application() {
    if let Err(e) = parse_file("fflib_Application.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_fflib_query_factory() {
    if let Err(e) = parse_file("fflib_QueryFactory.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_fflib_sobject_domain() {
    if let Err(e) = parse_file("fflib_SObjectDomain.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_fflib_sobject_selector() {
    if let Err(e) = parse_file("fflib_SObjectSelector.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_fflib_sobject_unit_of_work() {
    if let Err(e) = parse_file("fflib_SObjectUnitOfWork.cls") {
        panic!("{}", e);
    }
}

// ==================== Apex Recipes Tests ====================

#[test]
fn test_parse_account_trigger_handler() {
    if let Err(e) = parse_file("AccountTriggerHandler.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_batch_apex_recipes() {
    if let Err(e) = parse_file("BatchApexRecipes.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_queueable_recipes() {
    if let Err(e) = parse_file("QueueableRecipes.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_dml_recipes() {
    if let Err(e) = parse_file("DMLRecipes.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_soql_recipes() {
    if let Err(e) = parse_file("SOQLRecipes.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_iteration_recipes() {
    if let Err(e) = parse_file("IterationRecipes.cls") {
        panic!("{}", e);
    }
}

// ==================== NebulaLogger Tests ====================

#[test]
fn test_parse_logger() {
    if let Err(e) = parse_file("Logger.cls") {
        panic!("{}", e);
    }
}

#[test]
fn test_parse_log_entry_event_builder() {
    if let Err(e) = parse_file("LogEntryEventBuilder.cls") {
        panic!("{}", e);
    }
}

// ==================== Summary Test ====================

#[test]
fn test_all_real_files_summary() {
    let files = [
        "fflib_Application.cls",
        "fflib_QueryFactory.cls",
        "fflib_SObjectDomain.cls",
        "fflib_SObjectSelector.cls",
        "fflib_SObjectUnitOfWork.cls",
        "AccountTriggerHandler.cls",
        "BatchApexRecipes.cls",
        "QueueableRecipes.cls",
        "DMLRecipes.cls",
        "SOQLRecipes.cls",
        "IterationRecipes.cls",
        "Logger.cls",
        "LogEntryEventBuilder.cls",
    ];

    let mut passed = 0;
    let mut failed = Vec::new();

    for file in &files {
        match parse_file(file) {
            Ok(_) => {
                passed += 1;
                println!("PASS: {}", file);
            }
            Err(e) => {
                failed.push(e);
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Passed: {}/{}", passed, files.len());

    if !failed.is_empty() {
        println!("Failed: {}", failed.len());
        for e in &failed {
            println!("  - {}", e);
        }
        panic!("{} files failed to parse", failed.len());
    }
}
