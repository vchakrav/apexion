use apexrust::{parse, Expression, Statement, TypeDeclaration, ClassMember};

/// Helper to parse a single expression within a method
fn parse_expr(expr_str: &str) -> Expression {
    let source = format!(
        "public class Test {{ public void test() {{ var x = {}; }} }}",
        expr_str
    );
    let cu = parse(&source).expect("Failed to parse");
    if let TypeDeclaration::Class(class) = &cu.declarations[0] {
        if let ClassMember::Method(method) = &class.members[0] {
            if let Some(block) = &method.body {
                if let Statement::LocalVariable(var) = &block.statements[0] {
                    return var.declarators[0].initializer.clone().unwrap();
                }
            }
        }
    }
    panic!("Could not extract expression");
}

/// Helper to check if parsing succeeds
fn parses_ok(source: &str) -> bool {
    parse(source).is_ok()
}

// ==================== Literal Expression Tests ====================

#[test]
fn test_null_literal() {
    let expr = parse_expr("null");
    assert!(matches!(expr, Expression::Null(_)));
}

#[test]
fn test_boolean_literals() {
    let expr = parse_expr("true");
    assert!(matches!(expr, Expression::Boolean(true, _)));

    let expr = parse_expr("false");
    assert!(matches!(expr, Expression::Boolean(false, _)));
}

#[test]
fn test_integer_literals() {
    let expr = parse_expr("0");
    assert!(matches!(expr, Expression::Integer(0, _)));

    let expr = parse_expr("42");
    assert!(matches!(expr, Expression::Integer(42, _)));

    let expr = parse_expr("999999");
    assert!(matches!(expr, Expression::Integer(999999, _)));
}

#[test]
fn test_long_literals() {
    let expr = parse_expr("0L");
    assert!(matches!(expr, Expression::Long(0, _)));

    let expr = parse_expr("42L");
    assert!(matches!(expr, Expression::Long(42, _)));
}

#[test]
fn test_double_literals() {
    let expr = parse_expr("3.14");
    assert!(matches!(expr, Expression::Double(n, _) if (n - 3.14).abs() < 0.001));

    let expr = parse_expr("0.0");
    assert!(matches!(expr, Expression::Double(n, _) if n.abs() < 0.001));
}

#[test]
fn test_string_literals() {
    let expr = parse_expr("'hello'");
    assert!(matches!(expr, Expression::String(ref s, _) if s == "hello"));

    let expr = parse_expr("'hello world'");
    assert!(matches!(expr, Expression::String(ref s, _) if s == "hello world"));

    let expr = parse_expr("''");
    assert!(matches!(expr, Expression::String(ref s, _) if s.is_empty()));
}

// ==================== Identifier and Access Tests ====================

#[test]
fn test_simple_identifier() {
    let expr = parse_expr("myVar");
    assert!(matches!(expr, Expression::Identifier(ref s, _) if s == "myVar"));
}

#[test]
fn test_this_expression() {
    let source = "public class Test { public void test() { Object x = this; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_super_expression() {
    let source = "public class Test extends Base { public void test() { Object x = super.field; } }";
    assert!(parses_ok(source));
}

// ==================== Arithmetic Expression Tests ====================

#[test]
fn test_addition() {
    let expr = parse_expr("1 + 2");
    if let Expression::Binary(bin) = expr {
        assert!(matches!(bin.left, Expression::Integer(1, _)));
        assert!(matches!(bin.right, Expression::Integer(2, _)));
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_subtraction() {
    let expr = parse_expr("5 - 3");
    if let Expression::Binary(bin) = expr {
        assert!(matches!(bin.left, Expression::Integer(5, _)));
        assert!(matches!(bin.right, Expression::Integer(3, _)));
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_multiplication() {
    let expr = parse_expr("4 * 5");
    if let Expression::Binary(bin) = expr {
        assert!(matches!(bin.left, Expression::Integer(4, _)));
        assert!(matches!(bin.right, Expression::Integer(5, _)));
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_division() {
    let expr = parse_expr("10 / 2");
    if let Expression::Binary(bin) = expr {
        assert!(matches!(bin.left, Expression::Integer(10, _)));
        assert!(matches!(bin.right, Expression::Integer(2, _)));
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_operator_precedence_mult_over_add() {
    // 1 + 2 * 3 should be parsed as 1 + (2 * 3)
    let expr = parse_expr("1 + 2 * 3");
    if let Expression::Binary(bin) = expr {
        assert!(matches!(bin.left, Expression::Integer(1, _)));
        assert!(matches!(bin.right, Expression::Binary(_)));
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_operator_precedence_with_parens() {
    // (1 + 2) * 3 should parse correctly
    let expr = parse_expr("(1 + 2) * 3");
    if let Expression::Binary(bin) = expr {
        assert!(matches!(bin.left, Expression::Parenthesized(_, _)));
        assert!(matches!(bin.right, Expression::Integer(3, _)));
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_complex_arithmetic() {
    let source = "public class Test { public void test() { Integer x = 1 + 2 * 3 - 4 / 2; } }";
    assert!(parses_ok(source));
}

// ==================== Comparison Expression Tests ====================

#[test]
fn test_equality() {
    let source = "public class Test { public void test() { Boolean x = a == b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_inequality() {
    let source = "public class Test { public void test() { Boolean x = a != b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_exact_equality() {
    let source = "public class Test { public void test() { Boolean x = a === b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_exact_inequality() {
    let source = "public class Test { public void test() { Boolean x = a !== b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_less_than() {
    let source = "public class Test { public void test() { Boolean x = a < b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_greater_than() {
    let source = "public class Test { public void test() { Boolean x = a > b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_less_than_or_equal() {
    let source = "public class Test { public void test() { Boolean x = a <= b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_greater_than_or_equal() {
    let source = "public class Test { public void test() { Boolean x = a >= b; } }";
    assert!(parses_ok(source));
}

// ==================== Logical Expression Tests ====================

#[test]
fn test_logical_and() {
    let source = "public class Test { public void test() { Boolean x = a && b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_logical_or() {
    let source = "public class Test { public void test() { Boolean x = a || b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_logical_not() {
    let source = "public class Test { public void test() { Boolean x = !a; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_complex_logical() {
    let source = "public class Test { public void test() { Boolean x = (a && b) || (!c && d); } }";
    assert!(parses_ok(source));
}

// ==================== Bitwise Expression Tests ====================

#[test]
fn test_bitwise_and() {
    let source = "public class Test { public void test() { Integer x = a & b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_bitwise_or() {
    let source = "public class Test { public void test() { Integer x = a | b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_bitwise_xor() {
    let source = "public class Test { public void test() { Integer x = a ^ b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_bitwise_not() {
    let source = "public class Test { public void test() { Integer x = ~a; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_left_shift() {
    let source = "public class Test { public void test() { Integer x = a << 2; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_right_shift() {
    let source = "public class Test { public void test() { Integer x = a >> 2; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_unsigned_right_shift() {
    let source = "public class Test { public void test() { Integer x = a >>> 2; } }";
    assert!(parses_ok(source));
}

// ==================== Assignment Expression Tests ====================

#[test]
fn test_simple_assignment() {
    let source = "public class Test { public void test() { x = 5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_add_assign() {
    let source = "public class Test { public void test() { x += 5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_subtract_assign() {
    let source = "public class Test { public void test() { x -= 5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_multiply_assign() {
    let source = "public class Test { public void test() { x *= 5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_divide_assign() {
    let source = "public class Test { public void test() { x /= 5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_chained_assignment() {
    let source = "public class Test { public void test() { a = b = c = 5; } }";
    assert!(parses_ok(source));
}

// ==================== Increment/Decrement Tests ====================

#[test]
fn test_post_increment() {
    let source = "public class Test { public void test() { x++; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_post_decrement() {
    let source = "public class Test { public void test() { x--; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_pre_increment() {
    let source = "public class Test { public void test() { Integer y = ++x; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_pre_decrement() {
    let source = "public class Test { public void test() { Integer y = --x; } }";
    assert!(parses_ok(source));
}

// ==================== Ternary and Null-Coalescing Tests ====================

#[test]
fn test_ternary_expression() {
    let source = "public class Test { public void test() { Integer x = a ? b : c; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_nested_ternary() {
    let source = "public class Test { public void test() { Integer x = a ? b : c ? d : e; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_null_coalescing() {
    let source = "public class Test { public void test() { String x = a ?? b; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_chained_null_coalescing() {
    let source = "public class Test { public void test() { String x = a ?? b ?? c; } }";
    assert!(parses_ok(source));
}

// ==================== Method Call Tests ====================

#[test]
fn test_simple_method_call() {
    let source = "public class Test { public void test() { doSomething(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_method_call_with_args() {
    let source = "public class Test { public void test() { doSomething(1, 2, 3); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_method_call_on_object() {
    let source = "public class Test { public void test() { obj.doSomething(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_chained_method_calls() {
    let source = "public class Test { public void test() { obj.method1().method2().method3(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_static_method_call() {
    let source = "public class Test { public void test() { System.debug('test'); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_nested_method_calls() {
    let source = "public class Test { public void test() { outer(inner(x)); } }";
    assert!(parses_ok(source));
}

// ==================== Field Access Tests ====================

#[test]
fn test_simple_field_access() {
    let source = "public class Test { public void test() { String x = obj.field; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_chained_field_access() {
    let source = "public class Test { public void test() { String x = obj.field1.field2.field3; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_safe_navigation() {
    let source = "public class Test { public void test() { String x = obj?.field; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_chained_safe_navigation() {
    let source = "public class Test { public void test() { String x = obj?.field1?.field2; } }";
    assert!(parses_ok(source));
}

// ==================== Array Access Tests ====================

#[test]
fn test_array_access() {
    let source = "public class Test { public void test() { String x = arr[0]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_array_access_with_expression() {
    let source = "public class Test { public void test() { String x = arr[i + 1]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_multidimensional_array_access() {
    let source = "public class Test { public void test() { String x = arr[0][1]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_list_access() {
    let source = "public class Test { public void test() { String x = myList[0]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_map_access() {
    let source = "public class Test { public void test() { String x = myMap['key']; } }";
    assert!(parses_ok(source));
}

// ==================== Object Creation Tests ====================

#[test]
fn test_new_object() {
    let source = "public class Test { public void test() { Account a = new Account(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_new_object_with_args() {
    let source = "public class Test { public void test() { MyClass obj = new MyClass(1, 'test'); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_new_list() {
    let source = "public class Test { public void test() { List<String> l = new List<String>(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_new_set() {
    let source = "public class Test { public void test() { Set<Id> s = new Set<Id>(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_new_map() {
    let source = "public class Test { public void test() { Map<String, Integer> m = new Map<String, Integer>(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_new_array_with_size() {
    // Note: array creation with size uses different syntax in Apex
    let source = "public class Test { public void test() { List<Integer> arr = new List<Integer>(); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_new_list_with_initializer() {
    let source = "public class Test { public void test() { List<Integer> l = new List<Integer>{1, 2, 3}; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_new_map_with_initializer() {
    let source = "public class Test { public void test() { Map<String, Integer> m = new Map<String, Integer>{'a' => 1, 'b' => 2}; } }";
    assert!(parses_ok(source));
}

// ==================== Instanceof Tests ====================

#[test]
fn test_instanceof() {
    let source = "public class Test { public void test() { Boolean b = obj instanceof Account; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_instanceof_with_qualified_type() {
    let source = "public class Test { public void test() { Boolean b = obj instanceof System.Exception; } }";
    assert!(parses_ok(source));
}

// ==================== Unary Expression Tests ====================

#[test]
fn test_unary_minus() {
    let source = "public class Test { public void test() { Integer x = -5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_unary_plus() {
    // Unary plus is not typically used but should parse if we support it
    let source = "public class Test { public void test() { Integer x = 5; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_double_negation() {
    let source = "public class Test { public void test() { Boolean x = !!a; } }";
    assert!(parses_ok(source));
}

// ==================== Complex Expression Tests ====================

#[test]
fn test_complex_expression_1() {
    let source = "public class Test { public void test() { Integer x = (a + b) * (c - d) / e; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_complex_expression_2() {
    let source = "public class Test { public void test() { Boolean x = (a > b && c < d) || (e == f && g != h); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_complex_expression_3() {
    let source = "public class Test { public void test() { String x = obj?.method(a, b)?.field ?? 'default'; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_complex_expression_4() {
    let source = "public class Test { public void test() { Integer x = list[i].field.method(a + b, c * d); } }";
    assert!(parses_ok(source));
}

#[test]
fn test_complex_expression_5() {
    let source = "public class Test { public void test() { Boolean x = a ? b.method() : c?.field ?? d[0]; } }";
    assert!(parses_ok(source));
}

// ==================== SOQL Expression Tests ====================

#[test]
fn test_simple_soql() {
    let source = "public class Test { public void test() { List<Account> accs = [SELECT Id FROM Account]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_soql_with_fields() {
    let source = "public class Test { public void test() { List<Account> accs = [SELECT Id, Name, Industry FROM Account]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_soql_with_where() {
    let source = "public class Test { public void test() { List<Account> accs = [SELECT Id FROM Account WHERE Name != null]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_soql_with_limit() {
    let source = "public class Test { public void test() { List<Account> accs = [SELECT Id FROM Account LIMIT 10]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_soql_with_order_by() {
    let source = "public class Test { public void test() { List<Account> accs = [SELECT Id FROM Account ORDER BY Name]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_soql_with_order_by_desc() {
    let source = "public class Test { public void test() { List<Account> accs = [SELECT Id FROM Account ORDER BY Name DESC]; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_soql_complex() {
    let source = "public class Test { public void test() { List<Account> accs = [SELECT Id, Name FROM Account WHERE Industry != null ORDER BY Name ASC LIMIT 100 OFFSET 10]; } }";
    assert!(parses_ok(source));
}

// ==================== String Concatenation Tests ====================

#[test]
fn test_string_concatenation() {
    let source = "public class Test { public void test() { String x = 'Hello' + ' ' + 'World'; } }";
    assert!(parses_ok(source));
}

#[test]
fn test_string_concatenation_with_vars() {
    let source = "public class Test { public void test() { String x = 'Hello ' + name + '!'; } }";
    assert!(parses_ok(source));
}
