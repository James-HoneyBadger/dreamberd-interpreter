// Base types: Token, TokenType, and error handling for Gulf of Mexico interpreter
// Rust port of dreamberd/base.py

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Characters that can be part of names (alphanumeric + underscore + dot)
pub const ALPH_NUMS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.";

#[derive(Error, Debug)]
pub enum DreamberdError {
    #[error("{0}")]
    InterpretationError(String),
    
    #[error("{0}")]
    NonFormattedError(String),
}

/// Token types in the Gulf of Mexico language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    // Brackets
    RCurly,    // }
    LCurly,    // {
    RSquare,   // ]
    LSquare,   // [
    LParen,    // (
    RParen,    // )

    // Operators
    Dot,       // .
    Add,       // +
    Increment, // ++
    Decrement, // --
    Equal,     // =
    Divide,    // /
    Multiply,  // *
    Subtract,  // -

    // Punctuation
    Comma,     // ,
    Colon,     // :
    Semicolon, // ;
    Bang,      // !
    Question,  // ?
    Caret,     // ^
    FuncPoint, // =>

    // Comparison
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    NotEqual,     // ;=
    Pipe,         // |
    And,          // &

    // Literals
    Number,       // numeric literal

    // Special
    Whitespace,
    Name,
    String,
    Newline,
    SingleQuote,  // '
    DoubleQuote,  // "
}

impl TokenType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "}" => Some(TokenType::RCurly),
            "{" => Some(TokenType::LCurly),
            "]" => Some(TokenType::RSquare),
            "[" => Some(TokenType::LSquare),
            "(" => Some(TokenType::LParen),
            ")" => Some(TokenType::RParen),
            "." => Some(TokenType::Dot),
            "+" => Some(TokenType::Add),
            "++" => Some(TokenType::Increment),
            "--" => Some(TokenType::Decrement),
            "=" => Some(TokenType::Equal),
            "/" => Some(TokenType::Divide),
            "*" => Some(TokenType::Multiply),
            "-" => Some(TokenType::Subtract),
            "," => Some(TokenType::Comma),
            ":" => Some(TokenType::Colon),
            ";" => Some(TokenType::Semicolon),
            "!" => Some(TokenType::Bang),
            "?" => Some(TokenType::Question),
            "^" => Some(TokenType::Caret),
            "=>" => Some(TokenType::FuncPoint),
            "<" => Some(TokenType::LessThan),
            ">" => Some(TokenType::GreaterThan),
            "<=" => Some(TokenType::LessEqual),
            ">=" => Some(TokenType::GreaterEqual),
            ";=" => Some(TokenType::NotEqual),
            "|" => Some(TokenType::Pipe),
            "&" => Some(TokenType::And),
            "\n" => Some(TokenType::Newline),
            "'" => Some(TokenType::SingleQuote),
            "\"" => Some(TokenType::DoubleQuote),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::RCurly => "}",
            TokenType::LCurly => "{",
            TokenType::RSquare => "]",
            TokenType::LSquare => "[",
            TokenType::LParen => "(",
            TokenType::RParen => ")",
            TokenType::Dot => ".",
            TokenType::Add => "+",
            TokenType::Increment => "++",
            TokenType::Decrement => "--",
            TokenType::Equal => "=",
            TokenType::Divide => "/",
            TokenType::Multiply => "*",
            TokenType::Subtract => "-",
            TokenType::Comma => ",",
            TokenType::Colon => ":",
            TokenType::Semicolon => ";",
            TokenType::Bang => "!",
            TokenType::Question => "?",
            TokenType::Caret => "^",
            TokenType::FuncPoint => "=>",
            TokenType::LessThan => "<",
            TokenType::GreaterThan => ">",
            TokenType::LessEqual => "<=",
            TokenType::GreaterEqual => ">=",
            TokenType::NotEqual => ";=",
            TokenType::Pipe => "|",
            TokenType::And => "&",
            TokenType::Number => "NUMBER",
            TokenType::Newline => "\n",
            TokenType::SingleQuote => "'",
            TokenType::DoubleQuote => "\"",
            TokenType::Whitespace => " ",
            TokenType::Name => "NAME",
            TokenType::String => "STRING",
        }
    }
}

/// Operator types for expression evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatorType {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Exp,    // ^
    Gt,     // >
    Ge,     // >=
    Lt,     // <
    Le,     // <=
    Or,     // |
    And,    // &
    Com,    // , (comma for function arguments)
    E,      // =
    Ee,     // ==
    Eee,    // ===
    Eeee,   // ====
    Ne,     // ;=
    Nee,    // ;==
    Neee,   // ;===
}

impl OperatorType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(OperatorType::Add),
            "-" => Some(OperatorType::Sub),
            "*" => Some(OperatorType::Mul),
            "/" => Some(OperatorType::Div),
            "^" => Some(OperatorType::Exp),
            ">" => Some(OperatorType::Gt),
            ">=" => Some(OperatorType::Ge),
            "<" => Some(OperatorType::Lt),
            "<=" => Some(OperatorType::Le),
            "|" => Some(OperatorType::Or),
            "&" => Some(OperatorType::And),
            "," => Some(OperatorType::Com),
            "=" => Some(OperatorType::E),
            "==" => Some(OperatorType::Ee),
            "===" => Some(OperatorType::Eee),
            "====" => Some(OperatorType::Eeee),
            ";=" => Some(OperatorType::Ne),
            ";==" => Some(OperatorType::Nee),
            ";===" => Some(OperatorType::Neee),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OperatorType::Add => "+",
            OperatorType::Sub => "-",
            OperatorType::Mul => "*",
            OperatorType::Div => "/",
            OperatorType::Exp => "^",
            OperatorType::Gt => ">",
            OperatorType::Ge => ">=",
            OperatorType::Lt => "<",
            OperatorType::Le => "<=",
            OperatorType::Or => "|",
            OperatorType::And => "&",
            OperatorType::Com => ",",
            OperatorType::E => "=",
            OperatorType::Ee => "==",
            OperatorType::Eee => "===",
            OperatorType::Eeee => "====",
            OperatorType::Ne => ";=",
            OperatorType::Nee => ";==",
            OperatorType::Neee => ";===",
        }
    }
}

/// Token structure representing a lexical token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: String, line: usize, col: usize) -> Self {
        Token {
            token_type,
            value,
            line,
            col,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({:?}, {:?})", self.token_type, self.value)
    }
}

/// Print debug information with token context
pub fn debug_print(filename: &str, code: &str, message: &str, token: &Token) {
    if code.is_empty() {
        println!("\n{}\n", message.yellow());
        return;
    }

    let lines: Vec<&str> = code.split('\n').collect();
    if token.line == 0 || token.line > lines.len() {
        println!("\n{}\n", message.yellow());
        return;
    }

    let line = lines[token.line - 1];
    let num_carrots = token.value.len();
    let num_spaces = token.col.saturating_sub(num_carrots) + 1;

    let debug_string = format!(
        "{}, line {}\n\n  {}\n {}{}\n{}",
        filename.yellow(),
        token.line,
        line,
        " ".repeat(num_spaces),
        "^".repeat(num_carrots),
        message.yellow()
    );

    println!("\n{}\n", debug_string);
}

/// Print debug information without token context
pub fn debug_print_no_token(filename: &str, message: &str) {
    let debug_string = format!("{}\n\n{}", filename.yellow(), message.yellow());
    println!("\n{}\n", debug_string);
}

/// Raise an error at a specific token location
pub fn raise_error_at_token(
    filename: &str,
    code: &str,
    message: &str,
    token: &Token,
) -> DreamberdError {
    if code.is_empty() {
        return DreamberdError::InterpretationError(format!("\n{}\n", message.red()));
    }

    let lines: Vec<&str> = code.split('\n').collect();
    if token.line == 0 || token.line > lines.len() {
        return DreamberdError::InterpretationError(format!("\n{}\n", message.red()));
    }

    let line = lines[token.line - 1];
    let num_carrots = token.value.len();
    let num_spaces = token.col.saturating_sub(num_carrots) + 1;

    let error_string = format!(
        "{}, line {}\n\n  {}\n {}{}\n{}",
        filename.yellow(),
        token.line,
        line,
        " ".repeat(num_spaces),
        "^".repeat(num_carrots),
        message.red()
    );

    DreamberdError::InterpretationError(error_string)
}

/// Raise an error at a specific line
pub fn raise_error_at_line(
    filename: &str,
    code: &str,
    line: usize,
    message: &str,
) -> DreamberdError {
    if code.is_empty() {
        return DreamberdError::InterpretationError(format!("\n{}\n", message.red()));
    }

    let lines: Vec<&str> = code.split('\n').collect();
    if line == 0 || line > lines.len() {
        return DreamberdError::InterpretationError(format!("\n{}\n", message.red()));
    }

    let error_string = format!(
        "{}, line {}\n\n  {}\n\n{}",
        filename.yellow(),
        line,
        lines[line - 1],
        message.red()
    );

    DreamberdError::InterpretationError(error_string)
}
