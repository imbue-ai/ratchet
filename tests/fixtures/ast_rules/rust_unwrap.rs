// Test fixture for no-unwrap rule
// This file contains .unwrap() calls at specific lines for testing

fn example1() {
    let option = Some(42);
    let value = option.unwrap(); // Line 6 - should be detected
}

fn example2() {
    let result: Result<i32, String> = Ok(100);
    let value = result.unwrap(); // Line 11 - should be detected
}

fn nested_example() {
    let x = Some(Some(5));
    let y = x.unwrap().unwrap(); // Line 16 - should detect first unwrap
}

fn chain_example() {
    let values = vec![Some(1), Some(2), Some(3)];
    let first = values.first().unwrap(); // Line 21 - should be detected
}

// This should not trigger - no unwrap
fn clean_code() {
    let option = Some(42);
    if let Some(value) = option {
        println!("{}", value);
    }
}
