use apexrust::parse;

#[test]
fn test_get() {
    let code = "public class Test { void test() { if (a && (b.get() == c)) { x = 1; } } }";
    eprintln!("Testing get");
    match parse(code) {
        Ok(_) => (),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_foo() {
    let code = "public class Test { void test() { if (a && (b.foo() == c)) { x = 1; } } }";
    eprintln!("Testing foo");
    match parse(code) {
        Ok(_) => (),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_instanceof_in_parentheses() {
    let code = r#"
public class Test {
    public void test() {
        if (!(obj instanceof MyClass)) {
            return;
        }
    }
}
"#;
    match parse(code) {
        Ok(_) => (),
        Err(e) => panic!("Error: {:?}", e),
    }
}
