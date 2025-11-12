// Syntax tree structures for DreamBerd statements
// Rust port of dreamberd/processor/syntax_tree.py

use serde::{Deserialize, Serialize};
use crate::base::Token;
use crate::processor::expression_tree::ExpressionTreeNode;

/// Base trait for code statements
pub trait CodeStatementTrait {
    fn statement_type(&self) -> &'static str;
}

/// All possible code statement types
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<CodeStatement>,
    pub is_async: bool,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDeclaration {
    pub name: String,
    pub body: Vec<CodeStatement>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub name: Token,
    pub modifiers: Vec<String>, // e.g., ["const", "const"]
    pub value: Option<ExpressionTreeNode>,
    pub lifetime: Option<String>,
    pub confidence: i32,
    pub debug: u8, // 0 = none, 1 = ?, 2 = multiple ?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableAssignment {
    pub target: ExpressionTreeNode,
    pub value: ExpressionTreeNode,
    pub debug: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditional {
    pub condition: ExpressionTreeNode,
    pub body: Vec<CodeStatement>,
    pub else_body: Option<Vec<CodeStatement>>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub value: Option<ExpressionTreeNode>,
    pub debug: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub expression: ExpressionTreeNode,
    pub debug: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhenStatement {
    pub condition: ExpressionTreeNode,
    pub body: Vec<CodeStatement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfterStatement {
    pub event: String,
    pub body: Vec<CodeStatement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteStatement {
    pub target: ExpressionTreeNode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatement {
    pub name: String,
    pub target_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseStatement;

/// Parse tokens into a syntax tree (stub implementation)
pub fn generate_syntax_tree(
    _filename: &str,
    _tokens: Vec<Token>,
    _code: &str,
) -> Result<Vec<CodeStatement>, crate::base::DreamberdError> {
    // TODO: Implement full parser
    Ok(Vec::new())
}
