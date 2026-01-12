#![forbid(unsafe_code)]

//! AST-based rule execution using tree-sitter

mod ast_rule;
mod parser_cache;

pub use ast_rule::AstRule;
pub use parser_cache::ParserCache;
