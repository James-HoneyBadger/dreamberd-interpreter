// Lexer module - tokenizes Gulf of Mexico source code
// Rust port of dreamberd/processor/lexer.py

use crate::base::{raise_error_at_line, DreamberdError, Token, TokenType};

/// Add a token to the token list
fn add_token(
    tokens: &mut Vec<Token>,
    line: usize,
    col: usize,
    token_type: TokenType,
    value: Option<String>,
) {
    let val = value.unwrap_or_else(|| token_type.as_str().to_string());
    tokens.push(Token::new(token_type, val, line, col));
}

/// Get effective whitespace value for a character
fn get_effective_whitespace_value(ch: char) -> &'static str {
    match ch {
        ' ' => " ",
        '\t' => "\t",
        _ => "",
    }
}

/// Get quote count (double quotes count as 2, single quotes as 1)
fn get_quote_count(quote_value: &str) -> usize {
    quote_value
        .chars()
        .map(|c| if c == '"' { 2 } else { 1 })
        .sum()
}

/// Check if a quote string has matching pairs
fn is_matching_pair(quote_value: &str) -> bool {
    let total_sum = get_quote_count(quote_value);
    if total_sum % 2 != 0 {
        return false;
    }
    
    for i in 0..quote_value.len() {
        if get_quote_count(&quote_value[..i]) == total_sum / 2 {
            return true;
        }
    }
    false
}

/// Scan for a string token with flexible quote matching
/// Returns (new_cursor_position, string_value)
fn get_string_token(
    code: &str,
    start: usize,
    filename: &str,
    error_line: usize,
) -> Result<(usize, String), DreamberdError> {
    let chars: Vec<char> = code.chars().collect();
    let mut curr = start;
    let mut quote_value = String::new();

    // Collect opening quotes
    while curr < chars.len() && (chars[curr] == '"' || chars[curr] == '\'') {
        quote_value.push(chars[curr]);
        if is_matching_pair(&quote_value) {
            return Ok((curr, String::new()));
        }
        curr += 1;
    }

    let quote_count = get_quote_count(&quote_value);
    let mut value = String::new();

    // Scan for closing quotes
    while curr < chars.len() {
        let mut running_count = 0;
        let quote_start = curr;

        while curr < chars.len() && (chars[curr] == '"' || chars[curr] == '\'') {
            running_count += if chars[curr] == '"' { 2 } else { 1 };
            if running_count == quote_count {
                return Ok((curr, value));
            }
            curr += 1;
        }

        // Add characters between quotes
        if curr < chars.len() {
            for i in quote_start..=curr {
                if i < chars.len() {
                    value.push(chars[i]);
                }
            }
        }
        curr += 1;
    }

    Err(raise_error_at_line(
        filename,
        code,
        error_line,
        "Invalid string. Starting quotes do not match closing quotes.",
    ))
}

/// Tokenize Gulf of Mexico source code
pub fn tokenize(filename: &str, code: &str) -> Result<Vec<Token>, DreamberdError> {
    let padded_code = format!("{}   ", code); // Add padding to avoid bounds checks
    let chars: Vec<char> = padded_code.chars().collect();
    let mut tokens = Vec::new();
    let mut line_count = 1;
    let mut curr = 0;
    let mut start = 0; // Position of start of current line

    while curr < chars.len() {
        let ch = chars[curr];
        let col = curr - start;

        match ch {
            '\n' => {
                line_count += 1;
                start = curr; // Update line start position
                add_token(&mut tokens, line_count, col, TokenType::Newline, None);
            }
            '}' => add_token(&mut tokens, line_count, col, TokenType::RCurly, None),
            '{' => add_token(&mut tokens, line_count, col, TokenType::LCurly, None),
            '[' => add_token(&mut tokens, line_count, col, TokenType::LSquare, None),
            ']' => add_token(&mut tokens, line_count, col, TokenType::RSquare, None),
            '.' => add_token(&mut tokens, line_count, col, TokenType::Dot, None),
            ':' => add_token(&mut tokens, line_count, col, TokenType::Colon, None),
            '|' => add_token(&mut tokens, line_count, col, TokenType::Pipe, None),
            '&' => add_token(&mut tokens, line_count, col, TokenType::And, None),
            ';' => {
                let mut value = String::from(";");
                while curr + 1 < chars.len() && chars[curr + 1] == '=' {
                    value.push('=');
                    curr += 1;
                }
                if value.len() == 1 {
                    add_token(&mut tokens, line_count, col, TokenType::Semicolon, None);
                } else {
                    add_token(&mut tokens, line_count, col, TokenType::NotEqual, Some(value));
                }
            }
            ',' => add_token(&mut tokens, line_count, col, TokenType::Comma, None),
            '+' => {
                if curr + 1 < chars.len() && chars[curr + 1] == '+' {
                    add_token(&mut tokens, line_count, col, TokenType::Increment, None);
                    curr += 1;
                } else {
                    add_token(&mut tokens, line_count, col, TokenType::Add, None);
                }
            }
            '-' => {
                if curr + 1 < chars.len() && chars[curr + 1] == '-' {
                    add_token(&mut tokens, line_count, col, TokenType::Decrement, None);
                    curr += 1;
                } else if curr + 1 < chars.len() && (chars[curr + 1].is_ascii_digit() || 
                    (chars[curr + 1] == '.' && curr + 2 < chars.len() && chars[curr + 2].is_ascii_digit())) {
                    // Negative number
                    let num_start = curr;
                    curr += 1; // Skip the minus sign
                    let mut has_dot = false;
                    
                    while curr < chars.len() {
                        let c = chars[curr];
                        if c.is_ascii_digit() {
                            curr += 1;
                        } else if c == '.' && !has_dot {
                            has_dot = true;
                            curr += 1;
                        } else {
                            break;
                        }
                    }
                    
                    let number: String = chars[num_start..curr].iter().collect();
                    add_token(
                        &mut tokens,
                        line_count,
                        col,
                        TokenType::Number,
                        Some(number),
                    );
                    curr -= 1; // Back up since we'll increment at the end of the loop
                } else {
                    add_token(&mut tokens, line_count, col, TokenType::Subtract, None);
                }
            }
            '*' => add_token(&mut tokens, line_count, col, TokenType::Multiply, None),
            '/' => {
                // Check for comments
                if curr + 1 < chars.len() && chars[curr + 1] == '/' {
                    // Single-line comment: skip until newline
                    while curr < chars.len() && chars[curr] != '\n' {
                        curr += 1;
                    }
                    continue;
                }
                add_token(&mut tokens, line_count, col, TokenType::Divide, None);
            }
            '^' => add_token(&mut tokens, line_count, col, TokenType::Caret, None),
            '=' => {
                if curr + 1 < chars.len() && chars[curr + 1] == '>' {
                    add_token(&mut tokens, line_count, col, TokenType::FuncPoint, None);
                    curr += 1;
                } else {
                    // Check for multiple equals (==, ===, ====)
                    let mut value = String::from("=");
                    while curr + 1 < chars.len() && chars[curr + 1] == '=' {
                        value.push('=');
                        curr += 1;
                    }
                    match value.len() {
                        1 => add_token(&mut tokens, line_count, col, TokenType::Equal, None),
                        2 => add_token(&mut tokens, line_count, col, TokenType::EqualEqual, Some(value)),
                        3 => add_token(&mut tokens, line_count, col, TokenType::EqualEqualEqual, Some(value)),
                        4 => add_token(&mut tokens, line_count, col, TokenType::EqualEqualEqualEqual, Some(value)),
                        _ => add_token(&mut tokens, line_count, col, TokenType::EqualEqualEqualEqual, Some(value)), // Default to ====
                    }
                }
            }
            '<' => {
                if curr + 1 < chars.len() && chars[curr + 1] == '=' {
                    add_token(&mut tokens, line_count, col, TokenType::LessEqual, None);
                    curr += 1;
                } else {
                    add_token(&mut tokens, line_count, col, TokenType::LessThan, None);
                }
            }
            '>' => {
                if curr + 1 < chars.len() && chars[curr + 1] == '=' {
                    add_token(&mut tokens, line_count, col, TokenType::GreaterEqual, None);
                    curr += 1;
                } else {
                    add_token(&mut tokens, line_count, col, TokenType::GreaterThan, None);
                }
            }
            '!' => add_token(&mut tokens, line_count, col, TokenType::Bang, None),
            '?' => add_token(&mut tokens, line_count, col, TokenType::Question, None),
            '(' => add_token(&mut tokens, line_count, col, TokenType::LParen, None),
            ')' => add_token(&mut tokens, line_count, col, TokenType::RParen, None),
            '"' | '\'' => {
                // String literal - check for interpolation
                let (end_pos, string_value) = get_string_token(&padded_code, curr, filename, line_count)?;
                
                // Check if string contains interpolation patterns ${...}
                if string_value.contains("${") {
                    add_token(
                        &mut tokens,
                        line_count,
                        col,
                        TokenType::InterpolatedString,
                        Some(string_value),
                    );
                } else {
                    add_token(
                        &mut tokens,
                        line_count,
                        col,
                        TokenType::String,
                        Some(string_value),
                    );
                }
                curr = end_pos;
            }
            ' ' | '\t' => {
                // Whitespace
                let ws_value = get_effective_whitespace_value(ch);
                if !ws_value.is_empty() {
                    add_token(
                        &mut tokens,
                        line_count,
                        col,
                        TokenType::Whitespace,
                        Some(ws_value.to_string()),
                    );
                }
            }
            _ => {
                // Check for numbers first
                if ch.is_ascii_digit() || (ch == '.' && curr + 1 < chars.len() && chars[curr + 1].is_ascii_digit()) {
                    let num_start = curr;
                    let mut has_dot = false;
                    
                    while curr < chars.len() {
                        let c = chars[curr];
                        if c.is_ascii_digit() {
                            curr += 1;
                        } else if c == '.' && !has_dot {
                            has_dot = true;
                            curr += 1;
                        } else {
                            break;
                        }
                    }
                    
                    let number: String = chars[num_start..curr].iter().collect();
                    add_token(
                        &mut tokens,
                        line_count,
                        col,
                        TokenType::Number,
                        Some(number),
                    );
                    curr -= 1; // Back up since we'll increment at the end of the loop
                }
                // Name/identifier
                else if ch.is_alphabetic() || ch == '_' {
                    let name_start = curr;
                    while curr < chars.len()
                        && (chars[curr].is_alphanumeric() || chars[curr] == '_' || chars[curr] == '.')
                    {
                        curr += 1;
                    }
                    let name: String = chars[name_start..curr].iter().collect();
                    add_token(
                        &mut tokens,
                        line_count,
                        col,
                        TokenType::Name,
                        Some(name),
                    );
                    curr -= 1; // Back up since we'll increment at the end of the loop
                } else {
                    // Unknown character - treat as whitespace or skip
                    if !ch.is_whitespace() {
                        // Could log warning here
                    }
                }
            }
        }

        curr += 1;
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let code = "const const x = 5!";
        let tokens = tokenize("test", code).unwrap();
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_string_tokenization() {
        let code = r#"const const name = "Luke"!"#;
        let tokens = tokenize("test", code).unwrap();
        let string_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::String)
            .collect();
        assert_eq!(string_tokens.len(), 1);
        assert_eq!(string_tokens[0].value, "Luke");
    }
}
