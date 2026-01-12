// Test fixture for no-panic rule
// This file contains panic!() calls at specific lines for testing

fn example1() {
    let value = 42;
    if value == 0 {
        panic!("value cannot be zero"); // Line 7 - should be detected
    }
}

fn example2() {
    panic!("always fails"); // Line 12 - should be detected
}

fn conditional_panic() {
    let x = 10;
    if x > 5 {
        panic!("x is too large: {}", x); // Line 18 - should be detected
    }
}

fn nested_panic() {
    if true {
        if false {
            panic!(); // Line 25 - should be detected
        }
    }
}

// This should not trigger - no panic
fn clean_code() -> Result<i32, String> {
    let value = 42;
    if value == 0 {
        return Err("value cannot be zero".to_string());
    }
    Ok(value)
}
