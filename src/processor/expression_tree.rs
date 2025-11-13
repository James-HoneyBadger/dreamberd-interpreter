// Expression tree for evaluating Gulf of Mexico expressions
// Rust port of dreamberd/processor/expression_tree.py

use serde::{Deserialize, Serialize};
use crate::base::{Token, OperatorType};

/// Expression tree node types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionTreeNode {
    Value(ValueNode),
    UnaryOp(SingleOperatorNode),
    BinaryOp(ExpressionNode),
    Function(FunctionNode),
    Constructor(ConstructorNode),
    Index(IndexNode),
    List(ListNode),
}

/// Value node (literal or variable reference)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueNode {
    pub token: Token,
}

impl ValueNode {
    pub fn new(token: Token) -> Self {
        ValueNode { token }
    }
}

/// Unary operator node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleOperatorNode {
    pub operator: Token,
    pub expression: Box<ExpressionTreeNode>,
}

impl SingleOperatorNode {
    pub fn new(operator: Token, expression: ExpressionTreeNode) -> Self {
        SingleOperatorNode {
            operator,
            expression: Box::new(expression),
        }
    }
}

/// Binary operator node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionNode {
    pub left: Box<ExpressionTreeNode>,
    pub right: Box<ExpressionTreeNode>,
    pub operator: OperatorType,
    pub operator_token: Token,
}

impl ExpressionNode {
    pub fn new(
        left: ExpressionTreeNode,
        right: ExpressionTreeNode,
        operator: OperatorType,
        operator_token: Token,
    ) -> Self {
        ExpressionNode {
            left: Box::new(left),
            right: Box::new(right),
            operator,
            operator_token,
        }
    }
}

/// Function call node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionNode {
    pub name: Token,
    pub args: Vec<ExpressionTreeNode>,
}

impl FunctionNode {
    pub fn new(name: Token, args: Vec<ExpressionTreeNode>) -> Self {
        FunctionNode { name, args }
    }
}

/// Constructor call node (new ClassName())
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructorNode {
    pub class_name: Token,
    pub args: Vec<ExpressionTreeNode>,
}

impl ConstructorNode {
    pub fn new(class_name: Token, args: Vec<ExpressionTreeNode>) -> Self {
        ConstructorNode { class_name, args }
    }
}

/// Index access node (array[index] or object.property)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexNode {
    pub value: Box<ExpressionTreeNode>,
    pub index: Box<ExpressionTreeNode>,
}

impl IndexNode {
    pub fn new(value: ExpressionTreeNode, index: ExpressionTreeNode) -> Self {
        IndexNode {
            value: Box::new(value),
            index: Box::new(index),
        }
    }
}

/// List literal node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListNode {
    pub values: Vec<ExpressionTreeNode>,
}

impl ListNode {
    pub fn new(values: Vec<ExpressionTreeNode>) -> Self {
        ListNode { values }
    }
}

/// Build an expression tree from tokens
pub fn build_expression_tree(
    filename: &str,
    tokens: Vec<Token>,
    code: &str,
) -> Result<ExpressionTreeNode, crate::base::DreamberdError> {
    if tokens.is_empty() {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Empty expression".to_string(),
        ));
    }

    let mut pos = 0;
    let result = parse_expression_recursive(&tokens, &mut pos, 0, filename, code)?;
    
    // Make sure we consumed all tokens
    if pos < tokens.len() {
        return Err(crate::base::DreamberdError::NonFormattedError(
            format!("Unexpected tokens at end of expression: {:?}", &tokens[pos..])
        ));
    }
    
    Ok(result)
}

/// Parse expression with precedence climbing (recursive implementation)
fn parse_expression_recursive(
    tokens: &[Token],
    pos: &mut usize,
    min_precedence: i32,
    filename: &str,
    code: &str,
) -> Result<ExpressionTreeNode, crate::base::DreamberdError> {
    let mut left = parse_primary(tokens, pos, filename, code)?;

    while *pos < tokens.len() {
        let token = &tokens[*pos];
        
        // Handle postfix operations (indexing) - these have highest precedence
        if token.token_type == crate::base::TokenType::LSquare {
            // Index operation
            *pos += 1; // consume '['
            let index_expr = parse_expression_recursive(tokens, pos, 0, filename, code)?;
            
            if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::RSquare {
                return Err(crate::base::DreamberdError::NonFormattedError(
                    "Expected closing bracket in index operation".to_string(),
                ));
            }
            *pos += 1; // consume ']'
            
            left = ExpressionTreeNode::Index(IndexNode {
                value: Box::new(left),
                index: Box::new(index_expr),
            });
            continue;
        }
        
        // Handle binary operators
        if !matches!(token.token_type, crate::base::TokenType::Add | 
                                      crate::base::TokenType::Subtract |
                                      crate::base::TokenType::Multiply |
                                      crate::base::TokenType::Divide |
                                      crate::base::TokenType::Caret |
                                      crate::base::TokenType::Equal |
                                      crate::base::TokenType::NotEqual |
                                      crate::base::TokenType::EqualEqual |
                                      crate::base::TokenType::EqualEqualEqual |
                                      crate::base::TokenType::EqualEqualEqualEqual |
                                      crate::base::TokenType::LessThan |
                                      crate::base::TokenType::GreaterThan |
                                      crate::base::TokenType::LessEqual |
                                      crate::base::TokenType::GreaterEqual |
                                      crate::base::TokenType::Pipe |
                                      crate::base::TokenType::And) {
            break;
        }

        let op = match token.token_type {
            crate::base::TokenType::Add => crate::base::OperatorType::Add,
            crate::base::TokenType::Subtract => crate::base::OperatorType::Sub,
            crate::base::TokenType::Multiply => crate::base::OperatorType::Mul,
            crate::base::TokenType::Divide => crate::base::OperatorType::Div,
            crate::base::TokenType::Caret => crate::base::OperatorType::Exp,
            crate::base::TokenType::Equal => crate::base::OperatorType::E,
            crate::base::TokenType::NotEqual => crate::base::OperatorType::Ne,
            crate::base::TokenType::EqualEqual => crate::base::OperatorType::Ee,
            crate::base::TokenType::EqualEqualEqual => crate::base::OperatorType::Eee,
            crate::base::TokenType::EqualEqualEqualEqual => crate::base::OperatorType::Eeee,
            crate::base::TokenType::LessThan => crate::base::OperatorType::Lt,
            crate::base::TokenType::GreaterThan => crate::base::OperatorType::Gt,
            crate::base::TokenType::LessEqual => crate::base::OperatorType::Le,
            crate::base::TokenType::GreaterEqual => crate::base::OperatorType::Ge,
            crate::base::TokenType::Pipe => crate::base::OperatorType::Or,
            crate::base::TokenType::And => crate::base::OperatorType::And,
            _ => break,
        };

        let precedence = get_precedence(op);
        if precedence < min_precedence {
            break;
        }

        *pos += 1; // consume operator
        let right = parse_expression_recursive(tokens, pos, precedence + 1, filename, code)?;
        left = ExpressionTreeNode::BinaryOp(ExpressionNode::new(left, right, op, token.clone()));
    }

    Ok(left)
}

/// Get operator precedence (higher number = higher precedence)
fn get_precedence(op: crate::base::OperatorType) -> i32 {
    match op {
        crate::base::OperatorType::Or => 1,
        crate::base::OperatorType::And => 2,
        crate::base::OperatorType::E | crate::base::OperatorType::Ee | 
        crate::base::OperatorType::Eee | crate::base::OperatorType::Eeee |
        crate::base::OperatorType::Ne | crate::base::OperatorType::Nee |
        crate::base::OperatorType::Neee |
        crate::base::OperatorType::Lt | crate::base::OperatorType::Le |
        crate::base::OperatorType::Gt | crate::base::OperatorType::Ge => 3,
        crate::base::OperatorType::Add | crate::base::OperatorType::Sub => 4,
        crate::base::OperatorType::Mul | crate::base::OperatorType::Div => 5,
        crate::base::OperatorType::Exp => 6,
        _ => 0,
    }
}

/// Parse primary expressions (literals, variables, function calls, etc.)
fn parse_primary(
    tokens: &[Token],
    pos: &mut usize,
    filename: &str,
    code: &str,
) -> Result<ExpressionTreeNode, crate::base::DreamberdError> {
    if *pos >= tokens.len() {
        return Err(crate::base::DreamberdError::NonFormattedError(
            "Unexpected end of expression".to_string(),
        ));
    }

    let token = &tokens[*pos];
    *pos += 1;

    match token.token_type {
        crate::base::TokenType::Name => {
            // Check if this is the 'new' keyword for constructor calls
            if token.value == "new" {
                // Parse constructor call: new ClassName(args)
                if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::Name {
                    return Err(crate::base::DreamberdError::NonFormattedError(
                        "Expected class name after 'new'".to_string(),
                    ));
                }
                
                let class_name = tokens[*pos].clone();
                *pos += 1; // consume class name
                
                // Check for constructor arguments
                if *pos < tokens.len() && tokens[*pos].token_type == crate::base::TokenType::LParen {
                    *pos += 1; // consume '('
                    let mut args = Vec::new();

                    // Parse arguments
                    while *pos < tokens.len() && tokens[*pos].token_type != crate::base::TokenType::RParen {
                        if !args.is_empty() {
                            if tokens[*pos].token_type == crate::base::TokenType::Comma {
                                *pos += 1;
                            }
                        }
                        let arg = parse_expression_recursive(tokens, pos, 0, filename, code)?;
                        args.push(arg);
                    }

                    if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::RParen {
                        return Err(crate::base::DreamberdError::NonFormattedError(
                            "Expected closing parenthesis in constructor call".to_string(),
                        ));
                    }
                    *pos += 1; // consume ')'

                    Ok(ExpressionTreeNode::Constructor(ConstructorNode::new(class_name, args)))
                } else {
                    return Err(crate::base::DreamberdError::NonFormattedError(
                        "Expected parentheses after constructor class name".to_string(),
                    ));
                }
            }
            // Check if this is a special function that can be called without parentheses (like 'print')
            else if token.value == "print" && *pos < tokens.len() && 
                     !matches!(tokens[*pos].token_type, crate::base::TokenType::LParen) {
                // Parse print statement without parentheses: print arg1 arg2 arg3
                let mut args = Vec::new();
                
                // Parse arguments separated by spaces until statement end
                let stop_tokens = &[
                    crate::base::TokenType::Bang, 
                    crate::base::TokenType::Question, 
                    crate::base::TokenType::Semicolon,
                    crate::base::TokenType::RCurly, // End of block
                ];
                
                // Parse each argument as a separate primary expression
                while *pos < tokens.len() && !stop_tokens.contains(&tokens[*pos].token_type) {
                    // Parse one argument (literal, variable, etc.)
                    let arg_expr = parse_primary(tokens, pos, filename, code)?;
                    args.push(arg_expr);
                }
                
                Ok(ExpressionTreeNode::Function(FunctionNode::new(token.clone(), args)))
            }
            // Check if this is a function call
            else if *pos < tokens.len() && tokens[*pos].token_type == crate::base::TokenType::LParen {
                *pos += 1; // consume '('
                let mut args = Vec::new();

                // Parse arguments
                while *pos < tokens.len() && tokens[*pos].token_type != crate::base::TokenType::RParen {
                    if !args.is_empty() {
                        if tokens[*pos].token_type == crate::base::TokenType::Comma {
                            *pos += 1;
                        }
                    }
                    let arg = parse_expression_recursive(tokens, pos, 0, filename, code)?;
                    args.push(arg);
                }

                if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::RParen {
                    return Err(crate::base::DreamberdError::NonFormattedError(
                        "Expected closing parenthesis in function call".to_string(),
                    ));
                }
                *pos += 1; // consume ')'

                Ok(ExpressionTreeNode::Function(FunctionNode::new(token.clone(), args)))
            } else {
                Ok(ExpressionTreeNode::Value(ValueNode::new(token.clone())))
            }
        }
        crate::base::TokenType::String | crate::base::TokenType::Number => {
            Ok(ExpressionTreeNode::Value(ValueNode::new(token.clone())))
        }
        crate::base::TokenType::InterpolatedString => {
            // Handle interpolated string - for now, treat as regular value node
            // The interpolation logic will be handled in the interpreter
            Ok(ExpressionTreeNode::Value(ValueNode::new(token.clone())))
        }
        crate::base::TokenType::LSquare => {
            // Parse list literal
            let mut values = Vec::new();
            while *pos < tokens.len() && tokens[*pos].token_type != crate::base::TokenType::RSquare {
                if !values.is_empty() {
                    if tokens[*pos].token_type == crate::base::TokenType::Comma {
                        *pos += 1;
                    }
                }
                let value = parse_expression_recursive(tokens, pos, 0, filename, code)?;
                values.push(value);
            }

            if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::RSquare {
                return Err(crate::base::DreamberdError::NonFormattedError(
                    "Expected closing bracket in list literal".to_string(),
                ));
            }
            *pos += 1; // consume ']'

            Ok(ExpressionTreeNode::List(ListNode::new(values)))
        }
        crate::base::TokenType::LParen => {
            // Parentheses - parse subexpression
            let expr = parse_expression_recursive(tokens, pos, 0, filename, code)?;
            if *pos >= tokens.len() || tokens[*pos].token_type != crate::base::TokenType::RParen {
                return Err(crate::base::DreamberdError::NonFormattedError(
                    "Expected closing parenthesis".to_string(),
                ));
            }
            *pos += 1; // consume ')'
            Ok(expr)
        }
        crate::base::TokenType::Semicolon => {
            // NOT operator
            let expr = parse_primary(tokens, pos, filename, code)?;
            Ok(ExpressionTreeNode::UnaryOp(SingleOperatorNode::new(token.clone(), expr)))
        }
        _ => {
            Err(crate::base::DreamberdError::NonFormattedError(format!(
                "Unexpected token in expression: {:?}",
                token.token_type
            )))
        }
    }
}

/// Get the first token from an expression tree
pub fn get_expr_first_token(expr: &ExpressionTreeNode) -> Option<&Token> {
    match expr {
        ExpressionTreeNode::Value(v) => Some(&v.token),
        ExpressionTreeNode::UnaryOp(op) => Some(&op.operator),
        ExpressionTreeNode::BinaryOp(op) => get_expr_first_token(&op.left),
        ExpressionTreeNode::Function(f) => Some(&f.name),
        ExpressionTreeNode::Constructor(c) => Some(&c.class_name),
        ExpressionTreeNode::Index(i) => get_expr_first_token(&i.value),
        ExpressionTreeNode::List(_) => None,
    }
}
