// Processor module - lexer, parser, and syntax tree
pub mod lexer;
pub mod syntax_tree;
pub mod expression_tree;

pub use lexer::tokenize;
pub use syntax_tree::generate_syntax_tree;
