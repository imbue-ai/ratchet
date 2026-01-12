// Clean Rust file with no violations

fn clean_function() {
    println!("This is a complete implementation");
}

struct CompleteStruct {
    field: String,
}

impl CompleteStruct {
    fn new(field: String) -> Self {
        Self { field }
    }

    fn get_field(&self) -> &str {
        &self.field
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_struct() {
        let s = CompleteStruct::new("test".to_string());
        assert_eq!(s.get_field(), "test");
    }
}
