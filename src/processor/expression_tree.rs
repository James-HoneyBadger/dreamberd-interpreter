// Expression tree for evaluating DreamBerd expressions
// Rust port of dreamberd/processor/expression_tree.py

use serde::{Deserialize, Serialize};
use crate::base::{Token, OperatorType};

/// Expression tree node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpressionTreeNode {
    Value(ValueNode),
    UnaryOp(SingleOperatorNode),
    BinaryOp(ExpressionNode),
    Function(FunctionNode),
    Index(IndexNode),
    List(ListNode),
}

/// Value node (literal or variable reference)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueNode {
    pub token: Token,
}

impl ValueNode {
    pub fn new(token: Token) -> Self {
        ValueNode { token }
    }
}

/// Unary operator node
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    pub name: Token,
    pub args: Vec<ExpressionTreeNode>,
}

impl FunctionNode {
    pub fn new(name: Token, args: Vec<ExpressionTreeNode>) -> Self {
        FunctionNode { name, args }
    }
}

/// Index access node (array[index] or object.property)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNode {
    pub values: Vec<ExpressionTreeNode>,
}

impl ListNode {
    pub fn new(values: Vec<ExpressionTreeNode>) -> Self {
        ListNode { values }
    }
}

/// Build an expression tree from tokens (stub implementation)
pub fn build_expression_tree(
    _filename: &str,
    _tokens: Vec<Token>,
    _code: &str,
) -> Result<ExpressionTreeNode, crate::base::DreamberdError> {
    // TODO: Implement full expression parser
    Err(crate::base::DreamberdError::NonFormattedError(
        "Expression parsing not yet implemented".to_string(),
    ))
}

/// Get the first token from an expression tree
pub fn get_expr_first_token(expr: &ExpressionTreeNode) -> Option<&Token> {
    match expr {
        ExpressionTreeNode::Value(v) => Some(&v.token),
        ExpressionTreeNode::UnaryOp(op) => Some(&op.operator),
        ExpressionTreeNode::BinaryOp(op) => get_expr_first_token(&op.left),
        ExpressionTreeNode::Function(f) => Some(&f.name),
        ExpressionTreeNode::Index(i) => get_expr_first_token(&i.value),
        ExpressionTreeNode::List(_) => None,
    }
}
