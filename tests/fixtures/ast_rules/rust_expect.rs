// Test fixture for no-expect rule
// This file contains .expect() calls at specific lines for testing

fn example1() {
    let option = Some(42);
    let value = option.expect("should have value"); // Line 6 - should be detected
}

fn example2() {
    let result: Result<i32, String> = Ok(100);
    let value = result.expect("should be ok"); // Line 11 - should be detected
}

fn nested_example() {
    let x = Some(Some(5));
    let y = x.expect("outer").expect("inner"); // Line 16 - should detect first expect
}

fn chain_example() {
    let values = vec![Some(1), Some(2), Some(3)];
    let first = values.first().expect("should have first"); // Line 21 - should be detected
}

// This should not trigger - no expect
fn clean_code() {
    let option = Some(42);
    match option {
        Some(value) => println!("{}", value),
        None => println!("no value"),
    }
}
