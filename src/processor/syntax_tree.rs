// Syntax tree structures for Gulf of Mexico statements
// Rust port of dreamberd/processor/syntax_tree.py

use serde::{Deserialize, Serialize};
use crate::base::Token;
use crate::processor::expression_tree::ExpressionTreeNode;

/// Base trait for code statements
pub trait CodeStatementTrait {
    fn statement_type(&self) -> &'static str;
}

/// All possible code statement types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CodeStatement {
    FunctionDefinition(FunctionDefinition),
    ClassDeclaration(ClassDeclaration),
    VariableDeclaration(VariableDeclaration),
    VariableAssignment(VariableAssignment),
    Conditional(Conditional),
    ReturnStatement(ReturnStatement),
    ExpressionStatement(ExpressionStatement),
    WhenStatement(WhenStatement),
    AfterStatement(AfterStatement),
    DeleteStatement(DeleteStatement),
    ExportStatement(ExportStatement),
    ImportStatement(ImportStatement),
    ReverseStatement(ReverseStatement),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<CodeStatement>,
    pub is_async: bool,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDeclaration {
    pub name: String,
    pub body: Vec<CodeStatement>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub name: Token,
    pub modifiers: Vec<String>, // e.g., ["const", "const"]
    pub value: Option<ExpressionTreeNode>,
    pub lifetime: Option<String>,
    pub confidence: i32,
    pub debug: u8, // 0 = none, 1 = ?, 2 = multiple ?
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableAssignment {
    pub target: ExpressionTreeNode,
    pub value: ExpressionTreeNode,
    pub debug: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conditional {
    pub condition: ExpressionTreeNode,
    pub body: Vec<CodeStatement>,
    pub else_body: Option<Vec<CodeStatement>>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub value: Option<ExpressionTreeNode>,
    pub debug: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub expression: ExpressionTreeNode,
    pub debug: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhenStatement {
    pub condition: ExpressionTreeNode,
    pub body: Vec<CodeStatement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AfterStatement {
    pub event: String,
    pub body: Vec<CodeStatement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteStatement {
    pub target: ExpressionTreeNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportStatement {
    pub name: String,
    pub target_file: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportStatement {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReverseStatement;

/// Parse tokens into a syntax tree
pub fn generate_syntax_tree(
    filename: &str,
    tokens: Vec<Token>,
    code: &str,
) -> Result<Vec<CodeStatement>, crate::base::DreamberdError> {
    let mut statements = Vec::new();
    let mut pos = 0;

    // Filter out whitespace and comments
    let filtered_tokens: Vec<Token> = tokens
        .into_iter()
        .filter(|t| !matches!(t.token_type, crate::base::TokenType::Whitespace | crate::base::TokenType::Newline))
        .collect();

    while pos < filtered_tokens.len() {
        let stmt = parse_statement(&filtered_tokens, &mut pos, filename, code)?;
        statements.push(stmt);
    }

    Ok(statements)
}

/// Parse a single statement
fn parse_statement(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    if *pos >= tokens.len() {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Unexpected end of input".to_string(),
        ));
    }

    let token = &tokens[*pos];

    match token.token_type {
        crate::base::TokenType::Name => {
            match token.value.as_str() {
                // Variable declarations
                "const" | "var" => {
                    return parse_variable_declaration(tokens, pos, filename, code);
                }
                // Function definitions
                "function" | "func" | "fun" | "fn" | "functi" | "union" => {
                    return parse_function_definition(tokens, pos, filename, code);
                }
                // Class declarations
                "class" | "className" => {
                    return parse_class_declaration(tokens, pos, filename, code);
                }
                // Control flow
                "if" => {
                    return parse_conditional(tokens, pos, filename, code);
                }
                "when" => {
                    return parse_when_statement(tokens, pos, filename, code);
                }
                "after" => {
                    return parse_after_statement(tokens, pos, filename, code);
                }
                "return" => {
                    return parse_return_statement(tokens, pos, filename, code);
                }
                "delete" => {
                    return parse_delete_statement(tokens, pos, filename, code);
                }
                "export" => {
                    return parse_export_statement(tokens, pos, filename, code);
                }
                "import" => {
                    return parse_import_statement(tokens, pos, filename, code);
                }
                "reverse" => {
                    return parse_reverse_statement(tokens, pos, filename, code);
                }
                // Expression statement (function call, assignment, etc.)
                _ => {
                    return parse_expression_statement(tokens, pos, filename, code);
                }
            }
        }
        _ => {
            return parse_expression_statement(tokens, pos, filename, code);
        }
    }
}

/// Parse variable declaration (const const x = 5!)
fn parse_variable_declaration(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    let mut modifiers = Vec::new();
    let mut lifetime = None;
    let mut confidence = 100; // default confidence

    // Parse modifiers (const/var)
    while *pos < tokens.len() && matches!(tokens[*pos].value.as_str(), "const" | "var") {
        modifiers.push(tokens[*pos].value.clone());
        *pos += 1;
    }

    // Check for global modifier (const const const)
    let is_global = modifiers.len() == 3;

    // Parse variable name
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::Name {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected variable name after modifiers".to_string(),
        ));
    }
    let name_token = tokens[*pos].clone();
    *pos += 1;

    // Parse lifetime (<5> or <20s> or <Infinity>)
    if *pos < tokens.len() && tokens[*pos].token_type == crate::base::TokenType::LessThan {
        *pos += 1; // consume '<'
        let mut lifetime_str = String::new();

        while *pos < tokens.len() && tokens[*pos].token_type != crate::base::TokenType::GreaterThan {
            lifetime_str.push_str(&tokens[*pos].value);
            *pos += 1;
        }

        if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::GreaterThan {
            return Err(crate::base::DreamberdError::NonFormattedError(
                "Expected '>' to close lifetime specifier".to_string(),
            ));
        }
        *pos += 1; // consume '>'

        lifetime = Some(lifetime_str);
    }

    // Parse assignment
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::Equal {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '=' in variable declaration".to_string(),
        ));
    }
    *pos += 1; // consume '='

    // Parse value expression
    let value_tokens = collect_expression_tokens(tokens, pos);
    let value = crate::processor::expression_tree::build_expression_tree(filename, value_tokens, code)?;

    // Parse debug markers (? or ??)
    let debug = parse_debug_markers(tokens, pos);

    Ok(CodeStatement::VariableDeclaration(VariableDeclaration {
        name: name_token,
        modifiers,
        value: Some(value),
        lifetime,
        confidence,
        debug,
    }))
}

/// Parse function definition
fn parse_function_definition(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    let mut keywords = Vec::new();
    let mut is_async = false;

    // Parse function keywords
    while *pos < tokens.len() && matches!(tokens[*pos].value.as_str(),
        "function" | "func" | "fun" | "fn" | "functi" | "union" | "async") {
        let kw = tokens[*pos].value.clone();
        if kw == "async" {
            is_async = true;
        } else {
            keywords.push(kw);
        }
        *pos += 1;
    }

    // Parse function name
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::Name {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected function name".to_string(),
        ));
    }
    let name = tokens[*pos].value.clone();
    *pos += 1;

    // Parse parameters
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::LParen {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '(' after function name".to_string(),
        ));
    }
    *pos += 1; // consume '('

    let mut args = Vec::new();
    while *pos < tokens.len() && tokens[*pos].token_type != crate::base::TokenType::RParen {
        if !args.is_empty() {
            if tokens[*pos].token_type == crate::base::TokenType::Comma {
                *pos += 1;
            }
        }
        if tokens[*pos].token_type == crate::base::TokenType::Name {
            args.push(tokens[*pos].value.clone());
            *pos += 1;
        }
    }

    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::RParen {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected ')' after function parameters".to_string(),
        ));
    }
    *pos += 1; // consume ')'

    // Parse function body
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::FuncPoint {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '=>' after function parameters".to_string(),
        ));
    }
    *pos += 1; // consume '=>'

    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::LCurly {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '{' to start function body".to_string(),
        ));
    }
    *pos += 1; // consume '{'

    let body = parse_statement_block(tokens, pos, filename, code)?;

    Ok(CodeStatement::FunctionDefinition(FunctionDefinition {
        name,
        args,
        body,
        is_async,
        keywords,
    }))
}

/// Parse class declaration
fn parse_class_declaration(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    let mut keywords = Vec::new();

    // Parse class keyword
    if matches!(tokens[*pos].value.as_str(), "class" | "className") {
        keywords.push(tokens[*pos].value.clone());
        *pos += 1;
    }

    // Parse class name
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::Name {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected class name".to_string(),
        ));
    }
    let name = tokens[*pos].value.clone();
    *pos += 1;

    // Parse class body
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::LCurly {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '{' to start class body".to_string(),
        ));
    }
    *pos += 1; // consume '{'

    let body = parse_statement_block(tokens, pos, filename, code)?;

    Ok(CodeStatement::ClassDeclaration(ClassDeclaration {
        name,
        body,
        keywords,
    }))
}

/// Parse conditional (if-else)
fn parse_conditional(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'if'

    // Parse condition
    let condition_tokens = collect_expression_tokens_until(tokens, pos, &[crate::base::TokenType::LCurly]);
    let condition = crate::processor::expression_tree::build_expression_tree(filename, condition_tokens, code)?;

    // Parse if body
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::LCurly {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '{' after if condition".to_string(),
        ));
    }
    *pos += 1; // consume '{'

    let body = parse_statement_block(tokens, pos, filename, code)?;
    let mut else_body = None;

    // Parse else clause
    if *pos < tokens.len() && tokens[*pos].value == "else" {
        *pos += 1; // consume 'else'

        if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::LCurly {
            return Err(crate::base::DreamberdError::NonFormattedError(
                "Expected '{' after else".to_string(),
            ));
        }
        *pos += 1; // consume '{'

        else_body = Some(parse_statement_block(tokens, pos, filename, code)?);
    }

    Ok(CodeStatement::Conditional(Conditional {
        condition,
        body,
        else_body,
        keywords: vec!["if".to_string()],
    }))
}

/// Parse when statement
fn parse_when_statement(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'when'

    // Parse condition
    let condition_tokens = collect_expression_tokens_until(tokens, pos, &[crate::base::TokenType::LCurly]);
    let condition = crate::processor::expression_tree::build_expression_tree(filename, condition_tokens, code)?;

    // Parse body
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::LCurly {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '{' after when condition".to_string(),
        ));
    }
    *pos += 1; // consume '{'

    let body = parse_statement_block(tokens, pos, filename, code)?;

    Ok(CodeStatement::WhenStatement(WhenStatement {
        condition,
        body,
    }))
}

/// Parse expression statement
fn parse_expression_statement(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    let expr_tokens = collect_expression_tokens(tokens, pos);
    let expression = crate::processor::expression_tree::build_expression_tree(filename, expr_tokens, code)?;
    
    // Consume semicolon if present
    if *pos < tokens.len() && tokens[*pos].token_type == crate::base::TokenType::Semicolon {
        *pos += 1;
    }
    
    // Consume trailing ! if present
    if *pos < tokens.len() && tokens[*pos].token_type == crate::base::TokenType::Bang {
        *pos += 1;
    }
    
    let debug = parse_debug_markers(tokens, pos);

    Ok(CodeStatement::ExpressionStatement(ExpressionStatement {
        expression,
        debug,
    }))
}

/// Parse return statement
fn parse_return_statement(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'return'

    let mut value = None;
    let debug = parse_debug_markers(tokens, pos);

    // Check if there's a return value
    if *pos < tokens.len() && !matches!(tokens[*pos].token_type, crate::base::TokenType::Bang | crate::base::TokenType::Question) {
        let expr_tokens = collect_expression_tokens(tokens, pos);
        value = Some(crate::processor::expression_tree::build_expression_tree(filename, expr_tokens, code)?);
    }

    Ok(CodeStatement::ReturnStatement(ReturnStatement {
        value,
        debug,
    }))
}

/// Parse delete statement
fn parse_delete_statement(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'delete'

    let expr_tokens = collect_expression_tokens(tokens, pos);
    let target = crate::processor::expression_tree::build_expression_tree(filename, expr_tokens, code)?;

    Ok(CodeStatement::DeleteStatement(DeleteStatement {
        target,
    }))
}

/// Parse export statement
fn parse_export_statement(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    _code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'export'

    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::Name {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected name after export".to_string(),
        ));
    }
    let name = tokens[*pos].value.clone();
    *pos += 1;

    if *pos >= tokens.len() || tokens[*pos].value != "to" {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected 'to' after export name".to_string(),
        ));
    }
    *pos += 1; // consume 'to'

    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::String {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected target filename after 'to'".to_string(),
        ));
    }
    let target_file = tokens[*pos].value.clone();
    *pos += 1;

    Ok(CodeStatement::ExportStatement(ExportStatement {
        name,
        target_file,
    }))
}

/// Parse import statement
fn parse_import_statement(
    tokens: &[Token],
    pos: &mut usize,
    _filename: &str,
    _code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'import'

    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::Name {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected name after import".to_string(),
        ));
    }
    let name = tokens[*pos].value.clone();
    *pos += 1;

    Ok(CodeStatement::ImportStatement(ImportStatement {
        name,
    }))
}

/// Parse reverse statement
fn parse_reverse_statement(
    tokens: &[Token],
    pos: &mut usize,
    _filename: &str,
    _code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'reverse'
    Ok(CodeStatement::ReverseStatement(ReverseStatement))
}

/// Parse after statement (for events)
fn parse_after_statement(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<CodeStatement, crate::base::DreamberdError> {
    *pos += 1; // consume 'after'

    // Parse event (string literal)
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::String {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected event string after 'after'".to_string(),
        ));
    }
    let event = tokens[*pos].value.clone();
    *pos += 1;

    // Parse body
    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::LCurly {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Expected '{' after after event".to_string(),
        ));
    }
    *pos += 1; // consume '{'

    let body = parse_statement_block(tokens, pos, filename, code)?;

    Ok(CodeStatement::AfterStatement(AfterStatement {
        event,
        body,
    }))
}

/// Parse a block of statements until closing brace
fn parse_statement_block(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<Vec<CodeStatement>, crate::base::DreamberdError> {
    let mut statements = Vec::new();
    let mut brace_count = 1; // We already consumed the opening brace

    while *pos < tokens.len() && brace_count > 0 {
        match tokens[*pos].token_type {
            crate::base::TokenType::LCurly => {
                brace_count += 1;
                *pos += 1;
            }
            crate::base::TokenType::RCurly => {
                brace_count -= 1;
                if brace_count > 0 {
                    *pos += 1;
                }
            }
            _ => {
                let stmt = parse_statement(tokens, pos, filename, code)?;
                statements.push(stmt);
            }
        }
    }

    if brace_count > 0 {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Unclosed brace in statement block".to_string(),
        ));
    }

    *pos += 1; // consume final '}'
    Ok(statements)
}

/// Collect tokens for an expression until certain stop tokens
fn collect_expression_tokens_until(
    tokens: &[Token],
    pos: &mut usize,
    stop_tokens: &[crate::base::TokenType],
) -> Vec<Token> {
    let mut expr_tokens = Vec::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut brace_depth = 0;

    while *pos < tokens.len() {
        let token = &tokens[*pos];

        if stop_tokens.contains(&token.token_type) && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
            break;
        }

        match token.token_type {
            crate::base::TokenType::LParen => paren_depth += 1,
            crate::base::TokenType::RParen => paren_depth -= 1,
            crate::base::TokenType::LSquare => bracket_depth += 1,
            crate::base::TokenType::RSquare => bracket_depth -= 1,
            crate::base::TokenType::LCurly => brace_depth += 1,
            crate::base::TokenType::RCurly => brace_depth -= 1,
            _ => {}
        }

        expr_tokens.push(token.clone());
        *pos += 1;
    }

    expr_tokens
}

/// Collect tokens for an expression until statement end
fn collect_expression_tokens(tokens: &[Token], pos: &mut usize) -> Vec<Token> {
    collect_expression_tokens_until(
        tokens,
        pos,
        &[crate::base::TokenType::Bang, crate::base::TokenType::Question, crate::base::TokenType::Semicolon],
    )
}

/// Parse debug markers (? or ??)
fn parse_debug_markers(tokens: &[Token], pos: &mut usize) -> u8 {
    let mut debug = 0;
    while *pos < tokens.len() && tokens[*pos].token_type == crate::base::TokenType::Question {
        debug += 1;
        *pos += 1;
    }
    debug
}
