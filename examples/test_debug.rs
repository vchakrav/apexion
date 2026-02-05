use apexrust::parse;

fn main() {
    let source1 = "public class Test { public void test() { String x = obj?.method(a, b)?.field ?? 'default'; } }";
    println!("Test 1: Safe navigation with method call");
    match parse(source1) {
        Ok(_) => println!("  PASSED"),
        Err(e) => println!("  FAILED: {}", e),
    }

    let source2 = "public class Test { public void test() { Integer x = list[i].field.method(a + b, c * d); } }";
    println!("\nTest 2: Index access with chained field and method");
    match parse(source2) {
        Ok(_) => println!("  PASSED"),
        Err(e) => println!("  FAILED: {}", e),
    }

    // Simpler versions to debug
    let source3 = "public class Test { public void test() { Integer x = list[i]; } }";
    println!("\nTest 3: Simple index access");
    match parse(source3) {
        Ok(_) => println!("  PASSED"),
        Err(e) => println!("  FAILED: {}", e),
    }

    let source4 = "public class Test { public void test() { Integer x = list[i].field; } }";
    println!("\nTest 4: Index access with field");
    match parse(source4) {
        Ok(_) => println!("  PASSED"),
        Err(e) => println!("  FAILED: {}", e),
    }

    let source5 = "public class Test { public void test() { String x = obj?.field; } }";
    println!("\nTest 5: Simple safe navigation");
    match parse(source5) {
        Ok(_) => println!("  PASSED"),
        Err(e) => println!("  FAILED: {}", e),
    }

    let source6 = "public class Test { public void test() { String x = obj?.method(); } }";
    println!("\nTest 6: Safe navigation method call");
    match parse(source6) {
        Ok(_) => println!("  PASSED"),
        Err(e) => println!("  FAILED: {}", e),
    }

    // Test switch on enum
    let source7 = "public class Test { public void test() { switch on season { when SPRING { } when SUMMER { } when else { } } } }";
    println!("\nTest 7: Switch on enum");
    match parse(source7) {
        Ok(_) => println!("  PASSED"),
        Err(e) => println!("  FAILED: {}", e),
    }
}
