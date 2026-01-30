// Test fixture for Rust comment rules
// Tests: rust-no-todo-comments and rust-no-fixme-comments

// Five TODO cases that should be detected

// TODO: Implement this feature
fn function_with_todo() {
    // TODO: Add validation
    let x = 42;
}

// todo: lowercase also matches
fn lowercase_todo() {
    /* TODO: block comment */
    let y = 100;
}

/// TODO: This is a doc comment
fn doc_comment_todo() {
    /** TODO: block doc comment */
    let z = 200;
}

// This is a TODO comment
fn todo_in_middle() {
    let a = 1;
}

fn another_function() {
    // TODO(alice): Refactor this
    let b = 2;
}

// Five FIXME cases that should be detected

// FIXME: This is broken
fn function_with_fixme() {
    // FIXME: Memory leak here
    let c = 3;
}

// fixme: lowercase also matches
fn lowercase_fixme() {
    /* FIXME: block comment */
    let d = 4;
}

/// FIXME: This is a doc comment
fn doc_comment_fixme() {
    /** FIXME: block doc comment */
    let e = 5;
}

// This is a FIXME comment
fn fixme_in_middle() {
    let f = 6;
}

fn final_function() {
    // FIXME(bob): Fix before release
    let g = 7;
}

// False positives - strings should NOT trigger (AST rules only match comments)
fn string_literals() {
    let message = "TODO: This is in a string";
    let warning = "FIXME: This is also in a string";
    let raw_string = r#"TODO: Raw string with TODO"#;
    let multiline = r#"
        // TODO: This looks like a comment but it's in a string
        // FIXME: This is also in a string
    "#;
    println!("{} {} {} {}", message, warning, raw_string, multiline);
}

// Valid comments should NOT trigger
// Note: This is important
// HACK: This is a different marker
// XXX: Also different
fn clean_comments() {
    // Regular comment
    /* Block comment */
    let h = 8;
}
