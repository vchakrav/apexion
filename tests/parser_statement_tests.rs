use apexrust::parse;

/// Helper to check if parsing succeeds
fn parses_ok(source: &str) -> bool {
    parse(source).is_ok()
}

/// Helper to wrap statements in a class/method
fn wrap_statements(stmts: &str) -> String {
    format!(
        "public class Test {{ public void test() {{ {} }} }}",
        stmts
    )
}

// ==================== Variable Declaration Tests ====================

#[test]
fn test_simple_variable_declaration() {
    assert!(parses_ok(&wrap_statements("Integer x;")));
}

#[test]
fn test_variable_with_initializer() {
    assert!(parses_ok(&wrap_statements("Integer x = 5;")));
}

#[test]
fn test_multiple_variable_declarations() {
    assert!(parses_ok(&wrap_statements("Integer x, y, z;")));
}

#[test]
fn test_multiple_variables_with_initializers() {
    assert!(parses_ok(&wrap_statements("Integer x = 1, y = 2, z = 3;")));
}

#[test]
fn test_final_variable() {
    assert!(parses_ok(&wrap_statements("final Integer x = 5;")));
}

#[test]
fn test_string_variable() {
    assert!(parses_ok(&wrap_statements("String s = 'hello';")));
}

#[test]
fn test_boolean_variable() {
    assert!(parses_ok(&wrap_statements("Boolean b = true;")));
}

#[test]
fn test_decimal_variable() {
    assert!(parses_ok(&wrap_statements("Decimal d = 3.14;")));
}

#[test]
fn test_list_variable() {
    assert!(parses_ok(&wrap_statements("List<String> names = new List<String>();")));
}

#[test]
fn test_set_variable() {
    assert!(parses_ok(&wrap_statements("Set<Id> ids = new Set<Id>();")));
}

#[test]
fn test_map_variable() {
    assert!(parses_ok(&wrap_statements("Map<String, Integer> counts = new Map<String, Integer>();")));
}

#[test]
fn test_sobject_variable() {
    assert!(parses_ok(&wrap_statements("Account acc = new Account();")));
}

#[test]
fn test_nested_generic_variable() {
    assert!(parses_ok(&wrap_statements("Map<String, List<Integer>> data = new Map<String, List<Integer>>();")));
}

// ==================== If Statement Tests ====================

#[test]
fn test_simple_if() {
    assert!(parses_ok(&wrap_statements("if (true) { }")));
}

#[test]
fn test_if_with_statement() {
    assert!(parses_ok(&wrap_statements("if (x > 0) { doSomething(); }")));
}

#[test]
fn test_if_else() {
    assert!(parses_ok(&wrap_statements("if (x > 0) { doA(); } else { doB(); }")));
}

#[test]
fn test_if_else_if() {
    assert!(parses_ok(&wrap_statements("if (x > 0) { doA(); } else if (x < 0) { doB(); }")));
}

#[test]
fn test_if_else_if_else() {
    assert!(parses_ok(&wrap_statements("if (x > 0) { doA(); } else if (x < 0) { doB(); } else { doC(); }")));
}

#[test]
fn test_nested_if() {
    assert!(parses_ok(&wrap_statements("if (a) { if (b) { doSomething(); } }")));
}

#[test]
fn test_if_without_braces() {
    assert!(parses_ok(&wrap_statements("if (x > 0) doSomething();")));
}

#[test]
fn test_if_else_without_braces() {
    assert!(parses_ok(&wrap_statements("if (x > 0) doA(); else doB();")));
}

#[test]
fn test_complex_if_condition() {
    assert!(parses_ok(&wrap_statements("if (x > 0 && y < 10 || z == null) { }")));
}

// ==================== For Loop Tests ====================

#[test]
fn test_traditional_for_loop() {
    assert!(parses_ok(&wrap_statements("for (Integer i = 0; i < 10; i++) { }")));
}

#[test]
fn test_for_loop_empty_parts() {
    assert!(parses_ok(&wrap_statements("for (;;) { break; }")));
}

#[test]
fn test_for_loop_multiple_init() {
    assert!(parses_ok(&wrap_statements("for (Integer i = 0, j = 10; i < j; i++, j--) { }")));
}

#[test]
fn test_for_loop_complex_condition() {
    assert!(parses_ok(&wrap_statements("for (Integer i = 0; i < 10 && flag; i++) { }")));
}

#[test]
fn test_for_loop_without_braces() {
    assert!(parses_ok(&wrap_statements("for (Integer i = 0; i < 10; i++) doSomething();")));
}

#[test]
fn test_nested_for_loops() {
    assert!(parses_ok(&wrap_statements("for (Integer i = 0; i < 10; i++) { for (Integer j = 0; j < 10; j++) { } }")));
}

// ==================== For-Each Loop Tests ====================

#[test]
fn test_foreach_list() {
    assert!(parses_ok(&wrap_statements("for (String s : myList) { }")));
}

#[test]
fn test_foreach_set() {
    assert!(parses_ok(&wrap_statements("for (Id id : idSet) { }")));
}

#[test]
fn test_foreach_sobject() {
    assert!(parses_ok(&wrap_statements("for (Account acc : accounts) { }")));
}

#[test]
fn test_foreach_with_soql() {
    assert!(parses_ok(&wrap_statements("for (Account acc : [SELECT Id FROM Account]) { }")));
}

#[test]
fn test_foreach_without_braces() {
    assert!(parses_ok(&wrap_statements("for (String s : items) doSomething(s);")));
}

// ==================== While Loop Tests ====================

#[test]
fn test_simple_while() {
    assert!(parses_ok(&wrap_statements("while (true) { }")));
}

#[test]
fn test_while_with_condition() {
    assert!(parses_ok(&wrap_statements("while (x > 0) { x--; }")));
}

#[test]
fn test_while_without_braces() {
    assert!(parses_ok(&wrap_statements("while (x > 0) x--;")));
}

#[test]
fn test_while_complex_condition() {
    assert!(parses_ok(&wrap_statements("while (x > 0 && !done) { }")));
}

// ==================== Do-While Loop Tests ====================

#[test]
fn test_simple_do_while() {
    assert!(parses_ok(&wrap_statements("do { } while (true);")));
}

#[test]
fn test_do_while_with_body() {
    assert!(parses_ok(&wrap_statements("do { x++; } while (x < 10);")));
}

#[test]
fn test_do_while_complex_condition() {
    assert!(parses_ok(&wrap_statements("do { process(); } while (hasMore() && !cancelled);")));
}

// ==================== Switch Statement Tests ====================

#[test]
fn test_simple_switch() {
    assert!(parses_ok(&wrap_statements("switch on x { when 1 { doA(); } when 2 { doB(); } }")));
}

#[test]
fn test_switch_with_else() {
    assert!(parses_ok(&wrap_statements("switch on x { when 1 { doA(); } when else { doDefault(); } }")));
}

#[test]
fn test_switch_multiple_values() {
    assert!(parses_ok(&wrap_statements("switch on x { when 1, 2, 3 { doA(); } when 4, 5 { doB(); } }")));
}

#[test]
fn test_switch_on_string() {
    assert!(parses_ok(&wrap_statements("switch on status { when 'Active' { } when 'Inactive' { } when else { } }")));
}

#[test]
fn test_switch_on_enum() {
    assert!(parses_ok(&wrap_statements("switch on season { when SPRING { } when SUMMER { } when else { } }")));
}

#[test]
fn test_switch_with_type_check() {
    assert!(parses_ok(&wrap_statements("switch on obj { when Account a { } when Contact c { } when else { } }")));
}

// ==================== Try-Catch-Finally Tests ====================

#[test]
fn test_simple_try_catch() {
    assert!(parses_ok(&wrap_statements("try { doSomething(); } catch (Exception e) { }")));
}

#[test]
fn test_try_catch_finally() {
    assert!(parses_ok(&wrap_statements("try { doSomething(); } catch (Exception e) { } finally { cleanup(); }")));
}

#[test]
fn test_try_finally() {
    assert!(parses_ok(&wrap_statements("try { doSomething(); } finally { cleanup(); }")));
}

#[test]
fn test_multiple_catch() {
    assert!(parses_ok(&wrap_statements(
        "try { doSomething(); } catch (DmlException e) { } catch (QueryException e) { } catch (Exception e) { }"
    )));
}

#[test]
fn test_nested_try_catch() {
    assert!(parses_ok(&wrap_statements(
        "try { try { inner(); } catch (Exception e) { } } catch (Exception e) { }"
    )));
}

#[test]
fn test_try_catch_with_throw() {
    assert!(parses_ok(&wrap_statements(
        "try { doSomething(); } catch (Exception e) { throw e; }"
    )));
}

// ==================== Return Statement Tests ====================

#[test]
fn test_return_void() {
    assert!(parses_ok(&wrap_statements("return;")));
}

#[test]
fn test_return_value() {
    let source = "public class Test { public Integer test() { return 5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_return_expression() {
    let source = "public class Test { public Integer test() { return a + b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_return_null() {
    let source = "public class Test { public String test() { return null; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_return_method_call() {
    let source = "public class Test { public String test() { return getValue(); } }";
    assert!(parses_ok(source));
}

// ==================== Throw Statement Tests ====================

#[test]
fn test_throw_new_exception() {
    assert!(parses_ok(&wrap_statements("throw new CustomException('Error');")));
}

#[test]
fn test_throw_variable() {
    assert!(parses_ok(&wrap_statements("throw e;")));
}

#[test]
fn test_throw_in_catch() {
    assert!(parses_ok(&wrap_statements(
        "try { } catch (Exception e) { throw new CustomException('Wrapped', e); }"
    )));
}

// ==================== Break and Continue Tests ====================

#[test]
fn test_break_in_for() {
    assert!(parses_ok(&wrap_statements("for (Integer i = 0; i < 10; i++) { if (i == 5) break; }")));
}

#[test]
fn test_break_in_while() {
    assert!(parses_ok(&wrap_statements("while (true) { break; }")));
}

#[test]
fn test_continue_in_for() {
    assert!(parses_ok(&wrap_statements("for (Integer i = 0; i < 10; i++) { if (i == 5) continue; }")));
}

#[test]
fn test_continue_in_foreach() {
    assert!(parses_ok(&wrap_statements("for (String s : items) { if (s == null) continue; }")));
}

// ==================== DML Statement Tests ====================

#[test]
fn test_insert_single() {
    assert!(parses_ok(&wrap_statements("insert acc;")));
}

#[test]
fn test_insert_list() {
    assert!(parses_ok(&wrap_statements("insert accounts;")));
}

#[test]
fn test_insert_new() {
    assert!(parses_ok(&wrap_statements("insert new Account(Name = 'Test');")));
}

#[test]
fn test_update_single() {
    assert!(parses_ok(&wrap_statements("update acc;")));
}

#[test]
fn test_update_list() {
    assert!(parses_ok(&wrap_statements("update accounts;")));
}

#[test]
fn test_upsert_single() {
    assert!(parses_ok(&wrap_statements("upsert acc;")));
}

#[test]
fn test_delete_single() {
    assert!(parses_ok(&wrap_statements("delete acc;")));
}

#[test]
fn test_delete_list() {
    assert!(parses_ok(&wrap_statements("delete accounts;")));
}

#[test]
fn test_undelete() {
    assert!(parses_ok(&wrap_statements("undelete acc;")));
}

// ==================== Empty Statement Tests ====================

#[test]
fn test_empty_statement() {
    assert!(parses_ok(&wrap_statements(";")));
}

#[test]
fn test_multiple_empty_statements() {
    assert!(parses_ok(&wrap_statements(";;;")));
}

// ==================== Block Statement Tests ====================

#[test]
fn test_empty_block() {
    assert!(parses_ok(&wrap_statements("{ }")));
}

#[test]
fn test_nested_blocks() {
    assert!(parses_ok(&wrap_statements("{ { { } } }")));
}

#[test]
fn test_block_with_statements() {
    assert!(parses_ok(&wrap_statements("{ Integer x = 1; Integer y = 2; }")));
}

// ==================== Expression Statement Tests ====================

#[test]
fn test_method_call_statement() {
    assert!(parses_ok(&wrap_statements("doSomething();")));
}

#[test]
fn test_assignment_statement() {
    assert!(parses_ok(&wrap_statements("x = 5;")));
}

#[test]
fn test_increment_statement() {
    assert!(parses_ok(&wrap_statements("x++;")));
}

#[test]
fn test_method_chain_statement() {
    assert!(parses_ok(&wrap_statements("builder.append('a').append('b').build();")));
}

// ==================== Complex Statement Tests ====================

#[test]
fn test_complex_control_flow() {
    let stmts = r#"
        for (Account acc : [SELECT Id, Name FROM Account WHERE IsActive__c = true]) {
            if (acc.Name == null) {
                continue;
            }
            try {
                processAccount(acc);
            } catch (Exception e) {
                System.debug('Error: ' + e.getMessage());
                throw e;
            }
        }
    "#;
    assert!(parses_ok(&wrap_statements(stmts)));
}

#[test]
fn test_nested_loops_with_conditions() {
    let stmts = r#"
        for (Integer i = 0; i < 10; i++) {
            for (Integer j = 0; j < 10; j++) {
                if (i == j) {
                    continue;
                }
                if (i * j > 50) {
                    break;
                }
                process(i, j);
            }
        }
    "#;
    assert!(parses_ok(&wrap_statements(stmts)));
}

#[test]
fn test_switch_with_nested_control() {
    let stmts = r#"
        switch on status {
            when 'Active' {
                for (Item i : items) {
                    if (i.isValid()) {
                        process(i);
                    }
                }
            }
            when 'Pending' {
                while (hasMore()) {
                    processNext();
                }
            }
            when else {
                throw new InvalidStatusException(status);
            }
        }
    "#;
    assert!(parses_ok(&wrap_statements(stmts)));
}

#[test]
fn test_dml_with_try_catch() {
    let stmts = r#"
        try {
            insert new Account(Name = 'Test');
            update accounts;
            delete oldAccounts;
        } catch (DmlException e) {
            for (Integer i = 0; i < e.getNumDml(); i++) {
                System.debug(e.getDmlMessage(i));
            }
        } finally {
            cleanup();
        }
    "#;
    assert!(parses_ok(&wrap_statements(stmts)));
}
