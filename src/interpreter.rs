// Main interpreter engine for GulfOfMexico
// Rust port of dreamberd/interpreter.py

use std::collections::HashMap;
use crate::base::{DreamberdError, TokenType, OperatorType};
use crate::builtin::{DreamberdValue, Variable, Name, NamespaceEntry, VariableLifetime, DreamberdFunction, DreamberdUndefined, DreamberdNumber, DreamberdString, DreamberdBoolean, DreamberdList, create_builtin_function, db_to_string, db_to_boolean, db_not};
use crate::processor::syntax_tree::CodeStatement;
use crate::processor::expression_tree::ExpressionTreeNode;

pub type Namespace = HashMap<String, NamespaceEntry>;

/// Main interpreter struct
pub struct Interpreter {
    pub namespaces: Vec<Namespace>,
    pub filename: String,
    pub code: String,
}

impl Interpreter {
    pub fn new(filename: String, code: String) -> Self {
        let mut interpreter = Interpreter {
            namespaces: vec![HashMap::new()],
            filename,
            code,
        };

        // Initialize builtin functions and keywords
        interpreter.initialize_builtins();
        interpreter
    }

    fn initialize_builtins(&mut self) {
        // Add builtin functions
        let builtins = vec![
            ("print", create_builtin_function("print", -1, |args| {
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { print!(" "); }
                    print!("{}", db_to_string(arg));
                }
                println!();
                Some(DreamberdValue::Undefined(DreamberdUndefined))
            })),
            ("sqrt", create_builtin_function("sqrt", 1, |args| {
                if let Some(DreamberdValue::Number(n)) = args.get(0) {
                    Some(DreamberdValue::Number(DreamberdNumber::new(n.value.sqrt())))
                } else {
                    Some(DreamberdValue::Undefined(DreamberdUndefined))
                }
            })),
            ("len", create_builtin_function("len", 1, |args| {
                match args.get(0) {
                    Some(DreamberdValue::String(s)) => {
                        Some(DreamberdValue::Number(DreamberdNumber::new(s.value.chars().count() as f64)))
                    }
                    Some(DreamberdValue::List(l)) => {
                        Some(DreamberdValue::Number(DreamberdNumber::new(l.values.len() as f64)))
                    }
                    _ => Some(DreamberdValue::Undefined(DreamberdUndefined)),
                }
            })),
        ];

        for (name, func) in builtins {
            let lifetime = VariableLifetime::new(
                DreamberdValue::BuiltinFunction(func),
                100000000000,
                100,
                true,
                true
            );
            let variable = Variable::new(name.to_string(), lifetime);
            self.namespaces[0].insert(name.to_string(), NamespaceEntry::Variable(variable));
        }
    }

    /// Execute a list of code statements
    pub fn interpret_code_statements(
        &mut self,
        statements: Vec<CodeStatement>,
    ) -> Result<Option<DreamberdValue>, DreamberdError> {
        let mut result = None;

        for statement in statements {
            result = self.execute_statement(statement)?;
            if result.is_some() {
                break; // Return statement encountered
            }
        }

        Ok(result)
    }

    /// Execute a single statement
    fn execute_statement(
        &mut self,
        statement: CodeStatement,
    ) -> Result<Option<DreamberdValue>, DreamberdError> {
        match statement {
            CodeStatement::ExpressionStatement(expr_stmt) => {
                self.evaluate_expression(&expr_stmt.expression)?;
                Ok(None)
            }
            CodeStatement::VariableDeclaration(var_decl) => {
                self.execute_variable_declaration(var_decl)?;
                Ok(None)
            }
            CodeStatement::FunctionDefinition(func_def) => {
                self.execute_function_definition(func_def)?;
                Ok(None)
            }
            CodeStatement::ReturnStatement(ret) => {
                if let Some(expr) = ret.value {
                    Ok(Some(self.evaluate_expression(&expr)?))
                } else {
                    Ok(Some(DreamberdValue::Undefined(DreamberdUndefined)))
                }
            }
            CodeStatement::Conditional(cond) => {
                self.execute_conditional(cond)
            }
            CodeStatement::WhenStatement(when_stmt) => {
                self.execute_when_statement(when_stmt)?;
                Ok(None)
            }
            CodeStatement::DeleteStatement(del_stmt) => {
                self.execute_delete_statement(del_stmt)?;
                Ok(None)
            }
            _ => {
                // Other statement types not yet implemented
                Ok(None)
            }
        }
    }

    /// Execute variable declaration
    fn execute_variable_declaration(
        &mut self,
        var_decl: crate::processor::syntax_tree::VariableDeclaration,
    ) -> Result<(), DreamberdError> {
        let value = if let Some(expr) = var_decl.value {
            self.evaluate_expression(&expr)?
        } else {
            DreamberdValue::Undefined(DreamberdUndefined)
        };

        // Determine variable properties based on modifiers
        let can_be_reset = var_decl.modifiers.iter().any(|m| m == "var");
        let can_edit_value = var_decl.modifiers.iter().any(|m| m == "var");

        // Parse lifetime
        let variable_duration = if let Some(lifetime_str) = &var_decl.lifetime {
            if lifetime_str == "Infinity" {
                100000000000
            } else if lifetime_str.ends_with('s') {
                // Time-based lifetime
                lifetime_str.trim_end_matches('s').parse().unwrap_or(100000000000)
            } else {
                // Line-based lifetime
                lifetime_str.parse().unwrap_or(100000000000)
            }
        } else {
            100000000000 // Default: very long lifetime
        };

        let lifetime = VariableLifetime::new(value, variable_duration, var_decl.confidence, can_be_reset, can_edit_value);
        let variable = Variable::new(var_decl.name.value.clone(), lifetime);

        self.namespaces.last_mut().unwrap().insert(
            var_decl.name.value,
            NamespaceEntry::Variable(variable),
        );

        Ok(())
    }

    /// Execute function definition
    fn execute_function_definition(
        &mut self,
        func_def: crate::processor::syntax_tree::FunctionDefinition,
    ) -> Result<(), DreamberdError> {
        let function = DreamberdFunction {
            name: func_def.name.clone(),
            args: func_def.args.clone(),
            code: func_def.body,
            is_async: func_def.is_async,
        };

        let value = DreamberdValue::Function(function);
        let lifetime = VariableLifetime::new(value, 100000000000, 100, true, true);
        let variable = Variable::new(func_def.name.clone(), lifetime);

        self.namespaces.last_mut().unwrap().insert(
            func_def.name,
            NamespaceEntry::Variable(variable),
        );

        Ok(())
    }

    /// Execute conditional statement
    fn execute_conditional(
        &mut self,
        cond: crate::processor::syntax_tree::Conditional,
    ) -> Result<Option<DreamberdValue>, DreamberdError> {
        let condition_value = self.evaluate_expression(&cond.condition)?;
        let condition_bool = db_to_boolean(&condition_value);

        if let DreamberdBoolean { value: Some(true), .. } = condition_bool {
            self.interpret_code_statements(cond.body)
        } else if let Some(else_body) = cond.else_body {
            self.interpret_code_statements(else_body)
        } else {
            Ok(None)
        }
    }

    /// Execute when statement (reactive programming)
    fn execute_when_statement(
        &mut self,
        _when_stmt: crate::processor::syntax_tree::WhenStatement,
    ) -> Result<(), DreamberdError> {
        // TODO: Implement when statement watchers
        // This would set up reactive watchers that trigger when variables change
        Ok(())
    }

    /// Execute delete statement
    fn execute_delete_statement(
        &mut self,
        del_stmt: crate::processor::syntax_tree::DeleteStatement,
    ) -> Result<(), DreamberdError> {
        // For now, just evaluate the expression to "delete" it
        // In a full implementation, this would remove keywords or values
        let _ = self.evaluate_expression(&del_stmt.target)?;
        Ok(())
    }

    /// Evaluate an expression and return its value
    pub fn evaluate_expression(
        &mut self,
        expr: &ExpressionTreeNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        match expr {
            ExpressionTreeNode::Value(v) => self.evaluate_value_node(v),
            ExpressionTreeNode::BinaryOp(op) => self.evaluate_binary_operation(op),
            ExpressionTreeNode::Function(f) => self.evaluate_function_call(f),
            ExpressionTreeNode::List(l) => self.evaluate_list_literal(l),
            ExpressionTreeNode::UnaryOp(op) => self.evaluate_unary_operation(op),
            ExpressionTreeNode::Index(i) => self.evaluate_index_operation(i),
        }
    }

    /// Evaluate a value node (literal or variable reference)
    fn evaluate_value_node(
        &self,
        node: &crate::processor::expression_tree::ValueNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        match node.token.token_type {
            TokenType::String => {
                Ok(DreamberdValue::String(DreamberdString::new(node.token.value.clone())))
            }
            TokenType::Number => {
                let num: f64 = node.token.value.parse().unwrap_or(0.0);
                Ok(DreamberdValue::Number(DreamberdNumber::new(num)))
            }
            TokenType::Name => {
                // Look up variable in namespaces
                for namespace in self.namespaces.iter().rev() {
                    if let Some(entry) = namespace.get(&node.token.value) {
                        match entry {
                            NamespaceEntry::Variable(var) => {
                                return Ok(var.value().cloned().unwrap_or(
                                    DreamberdValue::Undefined(DreamberdUndefined)
                                ));
                            }
                            NamespaceEntry::Name(name) => {
                                return Ok(name.value.clone());
                            }
                        }
                    }
                }
                Err(DreamberdError::NonFormattedError(format!(
                    "Undefined variable: {}",
                    node.token.value
                )))
            }
            _ => Err(DreamberdError::NonFormattedError(format!(
                "Unexpected token type in value node: {:?}",
                node.token.token_type
            ))),
        }
    }

    /// Evaluate binary operation
    fn evaluate_binary_operation(
        &mut self,
        op: &crate::processor::expression_tree::ExpressionNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        let left = self.evaluate_expression(&op.left)?;
        let right = self.evaluate_expression(&op.right)?;

        match op.operator {
            OperatorType::Add => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Number(DreamberdNumber::new(l.value + r.value)))
                    }
                    (DreamberdValue::String(l), DreamberdValue::String(r)) => {
                        Ok(DreamberdValue::String(DreamberdString::new(format!("{}{}", l.value, r.value))))
                    }
                    _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
                }
            }
            OperatorType::Sub => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Number(DreamberdNumber::new(l.value - r.value)))
                    }
                    _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
                }
            }
            OperatorType::Mul => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Number(DreamberdNumber::new(l.value * r.value)))
                    }
                    _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
                }
            }
            OperatorType::Div => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        if r.value == 0.0 {
                            Ok(DreamberdValue::Undefined(DreamberdUndefined))
                        } else {
                            Ok(DreamberdValue::Number(DreamberdNumber::new(l.value / r.value)))
                        }
                    }
                    _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
                }
            }
            OperatorType::Exp => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Number(DreamberdNumber::new(l.value.powf(r.value))))
                    }
                    _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
                }
            }
            OperatorType::E | OperatorType::Ee | OperatorType::Eee | OperatorType::Eeee => {
                // Equality with different strictness levels
                Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(left == right))))
            }
            OperatorType::Ne | OperatorType::Nee | OperatorType::Neee => {
                // Inequality
                Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(left != right))))
            }
            OperatorType::Lt => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(l.value < r.value))))
                    }
                    _ => Ok(DreamberdValue::Boolean(DreamberdBoolean::maybe())),
                }
            }
            OperatorType::Le => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(l.value <= r.value))))
                    }
                    _ => Ok(DreamberdValue::Boolean(DreamberdBoolean::maybe())),
                }
            }
            OperatorType::Gt => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(l.value > r.value))))
                    }
                    _ => Ok(DreamberdValue::Boolean(DreamberdBoolean::maybe())),
                }
            }
            OperatorType::Ge => {
                match (&left, &right) {
                    (DreamberdValue::Number(l), DreamberdValue::Number(r)) => {
                        Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(l.value >= r.value))))
                    }
                    _ => Ok(DreamberdValue::Boolean(DreamberdBoolean::maybe())),
                }
            }
            _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
        }
    }

    /// Evaluate unary operation
    fn evaluate_unary_operation(
        &mut self,
        op: &crate::processor::expression_tree::SingleOperatorNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        let operand = self.evaluate_expression(&op.expression)?;

        match op.operator.token_type {
            TokenType::Semicolon => {
                // NOT operator
                let bool_val = db_to_boolean(&operand);
                Ok(DreamberdValue::Boolean(db_not(&bool_val)))
            }
            _ => Ok(operand), // Pass through for other operators
        }
    }

    /// Evaluate function call
    fn evaluate_function_call(
        &mut self,
        func: &crate::processor::expression_tree::FunctionNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        // Look up function
        let func_value = self.evaluate_value_node(&crate::processor::expression_tree::ValueNode::new(func.name.clone()))?;

        match func_value {
            DreamberdValue::Function(f) => {
                // Evaluate arguments
                let mut args = Vec::new();
                for arg_expr in &func.args {
                    args.push(self.evaluate_expression(arg_expr)?);
                }

                // Create new namespace for function call
                let mut new_namespace = HashMap::new();
                for (arg_name, arg_value) in f.args.iter().zip(args.iter()) {
                    new_namespace.insert(
                        arg_name.clone(),
                        NamespaceEntry::Name(Name::new(arg_name.clone(), arg_value.clone())),
                    );
                }

                self.namespaces.push(new_namespace);

                // Execute function body
                let result = self.interpret_code_statements(f.code.clone())?;

                // Pop function namespace
                self.namespaces.pop();

                Ok(result.unwrap_or(DreamberdValue::Undefined(DreamberdUndefined)))
            }
            DreamberdValue::BuiltinFunction(f) => {
                // Evaluate arguments
                let mut args = Vec::new();
                for arg_expr in &func.args {
                    args.push(self.evaluate_expression(arg_expr)?);
                }

                // Call builtin function
                Ok((f.function)(&args).unwrap_or(DreamberdValue::Undefined(DreamberdUndefined)))
            }
            _ => Err(DreamberdError::NonFormattedError("Not a function".to_string())),
        }
    }

    /// Evaluate list literal
    fn evaluate_list_literal(
        &mut self,
        list: &crate::processor::expression_tree::ListNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        let mut values = Vec::new();
        for expr in &list.values {
            values.push(self.evaluate_expression(expr)?);
        }

        Ok(DreamberdValue::List(DreamberdList::new(values)))
    }

    /// Evaluate index operation
    fn evaluate_index_operation(
        &mut self,
        index_op: &crate::processor::expression_tree::IndexNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        let value = self.evaluate_expression(&index_op.value)?;
        let index = self.evaluate_expression(&index_op.index)?;

        match (&value, &index) {
            (DreamberdValue::List(list), DreamberdValue::Number(idx)) => {
                // Arrays start at -1 in GulfOfMexico
                let actual_index = (idx.value as i32 + 1) as usize;
                if actual_index < list.values.len() {
                    Ok(list.values[actual_index].clone())
                } else {
                    Ok(DreamberdValue::Undefined(DreamberdUndefined))
                }
            }
            (DreamberdValue::String(s), DreamberdValue::Number(idx)) => {
                let chars: Vec<char> = s.value.chars().collect();
                let actual_index = (idx.value as i32 + 1) as usize;
                if actual_index < chars.len() {
                    Ok(DreamberdValue::String(DreamberdString::new(chars[actual_index].to_string())))
                } else {
                    Ok(DreamberdValue::Undefined(DreamberdUndefined))
                }
            }
            _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
        }
    }

    /// Get value from namespaces (helper method)
    pub fn get_value_from_namespaces(&self, name: &str) -> Option<DreamberdValue> {
        for namespace in self.namespaces.iter().rev() {
            if let Some(entry) = namespace.get(name) {
                match entry {
                    NamespaceEntry::Variable(var) => {
                        return var.value().cloned();
                    }
                    NamespaceEntry::Name(name_struct) => {
                        return Some(name_struct.value.clone());
                    }
                }
            }
        }
        None
    }
}
