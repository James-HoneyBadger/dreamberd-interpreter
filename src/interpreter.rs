// Main interpreter engine for GulfOfMexico
// Rust port of dreamberd/interpreter.py

use std::collections::HashMap;
use crate::base::{DreamberdError, TokenType, OperatorType};
use crate::builtin::{DreamberdValue, Variable, Name, NamespaceEntry, VariableLifetime, DreamberdFunction, DreamberdUndefined, DreamberdNumber, DreamberdString, DreamberdBoolean, DreamberdList, DreamberdMap, BuiltinFunction, create_builtin_function, db_to_string, db_to_boolean, db_not};
use crate::processor::syntax_tree::CodeStatement;
use crate::processor::expression_tree::{ExpressionTreeNode, ValueNode, ConstructorNode};

pub type Namespace = HashMap<String, NamespaceEntry>;

/// Main interpreter struct
pub struct Interpreter {
    pub namespaces: Vec<Namespace>,
    pub filename: String,
    pub code: String,
    pub current_line: u64,
}

impl Interpreter {
    pub fn new(filename: String, code: String) -> Self {
        let mut interpreter = Interpreter {
            namespaces: vec![HashMap::new()],
            filename,
            code,
            current_line: 1,
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



        // Add boolean literals
        let boolean_literals = vec![
            ("true", DreamberdValue::Boolean(DreamberdBoolean::true_val())),
            ("false", DreamberdValue::Boolean(DreamberdBoolean::false_val())),
            ("maybe", DreamberdValue::Boolean(DreamberdBoolean::maybe())),
        ];

        for (name, value) in boolean_literals {
            let lifetime = VariableLifetime::new(
                value,
                100000000000,
                100,
                true,
                true
            );
            let variable = Variable::new(name.to_string(), lifetime);
            self.namespaces[0].insert(name.to_string(), NamespaceEntry::Variable(variable));
        }
    }

    /// Clean up expired variables from all namespaces
    fn cleanup_expired_variables(&mut self) {
        for namespace in &mut self.namespaces {
            let expired_keys: Vec<String> = namespace
                .iter()
                .filter_map(|(key, entry)| {
                    if let NamespaceEntry::Variable(var) = entry {
                        if var.lifetimes.first().map_or(false, |lt| lt.is_expired(self.current_line)) {
                            Some(key.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            for key in expired_keys {
                namespace.remove(&key);
            }
        }
    }

    /// Execute a list of code statements
    pub fn interpret_code_statements(
        &mut self,
        statements: Vec<CodeStatement>,
    ) -> Result<Option<DreamberdValue>, DreamberdError> {
        let mut result = None;

        for statement in statements {
            // Clean up expired variables before executing each statement
            self.cleanup_expired_variables();
            
            result = self.execute_statement(statement)?;
            
            // Increment line counter (approximation - each statement is a line)
            self.current_line += 1;
            
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
            CodeStatement::VariableAssignment(var_assign) => {
                self.execute_variable_assignment(var_assign)?;
                Ok(None)
            }
            CodeStatement::AfterStatement(after_stmt) => {
                self.execute_after_statement(after_stmt)?;
                Ok(None)
            }
            CodeStatement::IndexAssignment(idx_assign) => {
                self.execute_index_assignment(idx_assign)?;
                Ok(None)
            }
            CodeStatement::ClassDeclaration(class_decl) => {
                self.execute_class_declaration(class_decl)?;
                Ok(None)
            }
            CodeStatement::ImportStatement(import_stmt) => {
                self.execute_import_statement(import_stmt)?;
                Ok(None)
            }
            CodeStatement::ExportStatement(export_stmt) => {
                self.execute_export_statement(export_stmt)?;
                Ok(None)
            }
            CodeStatement::ReverseStatement(_) => {
                self.execute_reverse_statement()?;
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

        let is_time_based = var_decl.lifetime.as_ref().map_or(false, |lt| lt.ends_with('s'));
        let lifetime = VariableLifetime::new_with_tracking(
            value, 
            variable_duration, 
            var_decl.confidence, 
            can_be_reset, 
            can_edit_value, 
            self.current_line,
            is_time_based
        );
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
        when_stmt: crate::processor::syntax_tree::WhenStatement,
    ) -> Result<(), DreamberdError> {
        // Evaluate the condition
        let condition_result = self.evaluate_expression(&when_stmt.condition)?;
        let condition_bool = db_to_boolean(&condition_result);

        // If condition is true or maybe, execute the body
        if let DreamberdBoolean { value: Some(true), .. } = condition_bool {
            self.interpret_code_statements(when_stmt.body)?;
        }
        // Note: In a full implementation, this would set up reactive watchers
        // that trigger when variables referenced in the condition change
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

    /// Evaluate interpolated string by parsing ${...} expressions
    fn evaluate_interpolated_string(&mut self, input: &str) -> Result<DreamberdValue, DreamberdError> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                
                // Collect expression until '}'
                let mut expr_str = String::new();
                let mut brace_count = 1;
                
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                    expr_str.push(ch);
                }
                
                // Parse and evaluate the expression
                if expr_str.trim().is_empty() {
                    result.push_str("${EMPTY}");
                } else {
                    match crate::processor::lexer::tokenize("interpolation", &expr_str) {
                        Ok(raw_tokens) => {
                            // Filter out whitespace tokens
                            let tokens: Vec<_> = raw_tokens.into_iter()
                                .filter(|token| token.token_type != crate::base::TokenType::Whitespace)
                                .collect();
                            
                            match crate::processor::expression_tree::build_expression_tree("interpolation", tokens, &expr_str) {
                                Ok(expr_tree) => {
                                    match self.evaluate_expression(&expr_tree) {
                                        Ok(value) => result.push_str(&db_to_string(&value).value),
                                        Err(e) => result.push_str(&format!("${{ERROR: {}}}", e)),
                                    }
                                }
                                Err(e) => result.push_str(&format!("${{PARSE_ERROR: {}}}", e)),
                            }
                        }
                        Err(e) => result.push_str(&format!("${{TOKEN_ERROR: {}}}", e)),
                    }
                }
            } else {
                result.push(ch);
            }
        }
        
        Ok(DreamberdValue::String(DreamberdString::new(result)))
    }

    /// Evaluate an expression and return its value
    fn evaluate_expression(
        &mut self,
        expr: &ExpressionTreeNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        match expr {
            ExpressionTreeNode::Value(v) => self.evaluate_value_node(v),
            ExpressionTreeNode::BinaryOp(op) => self.evaluate_binary_operation(op),
            ExpressionTreeNode::Function(f) => self.evaluate_function_call(f),
            ExpressionTreeNode::Constructor(c) => self.evaluate_constructor_call(c),
            ExpressionTreeNode::List(l) => self.evaluate_list_literal(l),
            ExpressionTreeNode::UnaryOp(op) => self.evaluate_unary_operation(op),
            ExpressionTreeNode::Index(i) => self.evaluate_index_operation(i),
        }
    }

    /// Evaluate a value node (literal or variable reference)
    fn evaluate_value_node(
        &mut self,
        node: &ValueNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        match node.token.token_type {
            TokenType::String => {
                Ok(DreamberdValue::String(DreamberdString::new(node.token.value.clone())))
            }
            TokenType::InterpolatedString => {
                self.evaluate_interpolated_string(&node.token.value)
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
            OperatorType::E => {
                // Check if this is an assignment (left side is assignable) or equality comparison
                match op.left.as_ref() {
                    // Variable assignment: x = value
                    ExpressionTreeNode::Value(value_node) if value_node.token.token_type == crate::base::TokenType::Name => {
                        let var_name = &value_node.token.value;
                        self.assign_variable(var_name, right.clone())?;
                        Ok(right) // Assignment returns the assigned value
                    }
                    // Index assignment: arr[index] = value  
                    ExpressionTreeNode::Index(index_node) => {
                        self.assign_index(index_node, right.clone())?;
                        Ok(right) // Assignment returns the assigned value
                    }
                    // Otherwise treat as equality comparison
                    _ => Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(left == right))))
                }
            }
            OperatorType::Ee | OperatorType::Eee | OperatorType::Eeee => {
                // Equality with different strictness levels (always comparison, never assignment)
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
        // Handle temporal keywords specially
        match func.name.value.as_str() {
            "current" => {
                if func.args.len() != 1 {
                    return Err(DreamberdError::NonFormattedError("current() expects 1 argument".to_string()));
                }
                // For current, just return the variable's current value
                return self.evaluate_expression(&func.args[0]);
            }
            "previous" => {
                if func.args.len() != 1 {
                    return Err(DreamberdError::NonFormattedError("previous() expects 1 argument".to_string()));
                }
                // Extract variable name from the argument
                if let ExpressionTreeNode::Value(value_node) = &func.args[0] {
                    if value_node.token.token_type == TokenType::Name {
                        let var_name = &value_node.token.value;
                        // Look up variable and get its previous value
                        for namespace in self.namespaces.iter().rev() {
                            if let Some(NamespaceEntry::Variable(var)) = namespace.get(var_name) {
                                return Ok(var.get_previous(0).cloned()
                                    .unwrap_or(DreamberdValue::Undefined(DreamberdUndefined)));
                            }
                        }
                        return Ok(DreamberdValue::Undefined(DreamberdUndefined));
                    }
                }
                return Err(DreamberdError::NonFormattedError("previous() expects a variable name".to_string()));
            }
            "next" => {
                if func.args.len() != 1 {
                    return Err(DreamberdError::NonFormattedError("next() expects 1 argument".to_string()));
                }
                // For now, next returns undefined (future value not available)
                return Ok(DreamberdValue::Undefined(DreamberdUndefined));
            }
            _ => {}
        }

        // Look up function
        let func_value = self.evaluate_value_node(&ValueNode::new(func.name.clone()))?;

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

    /// Evaluate constructor call (new ClassName())
    fn evaluate_constructor_call(
        &mut self,
        constructor: &ConstructorNode,
    ) -> Result<DreamberdValue, DreamberdError> {
        // For now, create a basic object with the constructor call information
        // In a full implementation, this would:
        // 1. Look up the class constructor
        // 2. Create a new instance
        // 3. Call the constructor with the provided arguments
        
        // Evaluate arguments
        let mut args = Vec::new();
        for arg_expr in &constructor.args {
            args.push(self.evaluate_expression(arg_expr)?);
        }

        // Create a simple object representing the constructed instance
        // In Gulf of Mexico, objects are typically maps/dictionaries
        let mut object_values = HashMap::new();
        
        // Add a special property to identify the class
        object_values.insert(
            "__class__".to_string(),
            DreamberdValue::String(DreamberdString::new(constructor.class_name.value.clone()))
        );
        
        // Add constructor arguments as properties (simple implementation)
        for (i, arg) in args.iter().enumerate() {
            object_values.insert(
                format!("arg{}", i),
                arg.clone()
            );
        }

        // Return the object as a Map
        Ok(DreamberdValue::Map(DreamberdMap { values: object_values }))
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
                // Use the new get method that supports float indexing
                if let Some(val) = list.get(idx.value) {
                    Ok(val.clone())
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

    /// Assign a value to a variable
    fn assign_variable(&mut self, var_name: &str, value: DreamberdValue) -> Result<(), DreamberdError> {
        // Find the variable in namespaces (starting from most recent)
        for namespace in self.namespaces.iter_mut().rev() {
            if let Some(entry) = namespace.get_mut(var_name) {
                match entry {
                    NamespaceEntry::Variable(var) => {
                        var.set_value(value);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
        
        // If variable doesn't exist, create it in the current namespace
        let lifetime = VariableLifetime::new(
            value,
            100000000000, // High duration
            100, // High confidence
            true, // Can be reset
            true, // Can edit value
        );
        let variable = Variable::new(var_name.to_string(), lifetime);
        
        if let Some(current_namespace) = self.namespaces.last_mut() {
            current_namespace.insert(var_name.to_string(), NamespaceEntry::Variable(variable));
        }
        
        Ok(())
    }

    /// Assign a value to an array/object index
    fn assign_index(&mut self, index_node: &crate::processor::expression_tree::IndexNode, value: DreamberdValue) -> Result<(), DreamberdError> {
        // Check if the target is a variable reference
        if let ExpressionTreeNode::Value(value_node) = index_node.value.as_ref() {
            if value_node.token.token_type == crate::base::TokenType::Name {
                let var_name = &value_node.token.value;
                let index = self.evaluate_expression(&index_node.index)?;
                
                // Find the variable in namespaces and modify it directly
                for namespace in self.namespaces.iter_mut().rev() {
                    if let Some(entry) = namespace.get_mut(var_name) {
                        if let NamespaceEntry::Variable(var) = entry {
                            if let Some(lifetime) = var.lifetimes.first_mut() {
                                if let DreamberdValue::List(ref mut list) = lifetime.value {
                                    if let DreamberdValue::Number(idx) = index {
                                        // Use the new insert_at method for both integer and float indices
                                        list.insert_at(idx.value, value);
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                
                return Err(DreamberdError::NonFormattedError(
                    format!("Variable '{}' not found or is not a list", var_name)
                ));
            }
        }
        
        Err(DreamberdError::NonFormattedError("Index assignment only supported on variable references".to_string()))
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

    /// Execute variable assignment
    fn execute_variable_assignment(
        &mut self,
        var_assign: crate::processor::syntax_tree::VariableAssignment,
    ) -> Result<(), DreamberdError> {
        // Evaluate the new value
        let new_value = self.evaluate_expression(&var_assign.value)?;
        
        // Get the target variable name (assuming it's a simple value node for now)
        if let crate::processor::expression_tree::ExpressionTreeNode::Value(value_node) = &var_assign.target {
            let var_name = &value_node.token.value;
            
            // Find the variable in the current namespaces
            for namespace in &mut self.namespaces {
                if let Some(entry) = namespace.get_mut(var_name) {
                    match entry {
                        NamespaceEntry::Variable(var) => {
                            if let Some(lifetime) = var.lifetimes.first() {
                                if lifetime.can_be_reset {
                                    var.set_value(new_value.clone());
                                    return Ok(());
                                } else {
                                    return Err(DreamberdError::InterpretationError(
                                        format!("Cannot reassign constant variable '{}'", var_name)
                                    ));
                                }
                            }
                        }
                        NamespaceEntry::Name(name_struct) => {
                            // For now, assume names can always be reassigned
                            name_struct.value = new_value;
                            return Ok(());
                        }
                    }
                }
            }
            
            // If we reach here, the variable was not found
            Err(DreamberdError::InterpretationError(
                format!("Variable '{}' not found", var_name)
            ))
        } else {
            // For now, only handle simple name assignments
            Err(DreamberdError::InterpretationError(
                "Complex assignment targets not yet implemented".to_string()
            ))
        }
    }

    /// Execute after statement (time-based event)
    fn execute_after_statement(
        &mut self,
        after_stmt: crate::processor::syntax_tree::AfterStatement,
    ) -> Result<(), DreamberdError> {
        // For now, just execute the body immediately (placeholder implementation)
        // In a full implementation, this would schedule the execution for later
        println!("After statement executing (placeholder): {:?}", after_stmt);
        
        // Execute the body statements
        for statement in after_stmt.body {
            self.execute_statement(statement)?;
        }
        
        Ok(())
    }

    /// Execute index assignment (array[index] = value)
    fn execute_index_assignment(
        &mut self,
        idx_assign: crate::processor::syntax_tree::IndexAssignment,
    ) -> Result<(), DreamberdError> {
        // Evaluate the index
        let index_value = self.evaluate_expression(&idx_assign.index)?;
        
        // Evaluate the new value
        let new_value = self.evaluate_expression(&idx_assign.value)?;
        
        // For now, handle simple cases where target is a list variable
        if let crate::processor::expression_tree::ExpressionTreeNode::Value(target_node) = &idx_assign.target {
            let var_name = &target_node.token.value;
            
            // Find the variable in namespaces
            for namespace in &mut self.namespaces {
                if let Some(entry) = namespace.get_mut(var_name) {
                    match entry {
                        NamespaceEntry::Variable(var) => {
                            if let Some(DreamberdValue::List(list)) = var.lifetimes.first_mut().map(|lt| &mut lt.value) {
                                // Handle different index types
                                match &index_value {
                                    DreamberdValue::Number(num) => {
                                        let index = num.value as i32;
                                        // Gulf of Mexico uses -1 based indexing
                                        let actual_index = (index + 1) as usize;
                                        
                                        if actual_index < list.values.len() {
                                            list.values[actual_index] = new_value;
                                            return Ok(());
                                        } else {
                                            return Err(DreamberdError::InterpretationError(
                                                format!("Index {} out of bounds for list of length {}", 
                                                       index, list.values.len())
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(DreamberdError::InterpretationError(
                                            "Index must be a number".to_string()
                                        ));
                                    }
                                }
                            } else {
                                return Err(DreamberdError::InterpretationError(
                                    format!("Variable '{}' is not a list", var_name)
                                ));
                            }
                        }
                        NamespaceEntry::Name(_) => {
                            return Err(DreamberdError::InterpretationError(
                                "Cannot assign to index of a name".to_string()
                            ));
                        }
                    }
                }
            }
            
            Err(DreamberdError::InterpretationError(
                format!("Variable '{}' not found", var_name)
            ))
        } else {
            Err(DreamberdError::InterpretationError(
                "Complex index assignment targets not yet supported".to_string()
            ))
        }
    }

    /// Execute class declaration
    fn execute_class_declaration(
        &mut self,
        class_decl: crate::processor::syntax_tree::ClassDeclaration,
    ) -> Result<(), DreamberdError> {
        // For now, create a simple class object that stores methods and properties
        // In Gulf of Mexico, classes are basically namespaces with methods
        
        // Create a namespace for the class
        let _class_namespace: std::collections::HashMap<String, NamespaceEntry> = HashMap::new();
        
        // For basic implementation, just store the class as a special object type
        // TODO: Implement proper class instantiation, inheritance, etc.
        
        println!("Class '{}' declared (placeholder implementation)", class_decl.name);
        
        // Execute the class body statements to define methods and properties
        for statement in class_decl.body {
            // For now, just execute statements in class context
            // In full implementation, this would populate the class namespace
            self.execute_statement(statement)?;
        }
        
        Ok(())
    }

    /// Execute import statement - loads variables from another file
    fn execute_import_statement(
        &mut self,
        import_stmt: crate::processor::syntax_tree::ImportStatement,
    ) -> Result<(), DreamberdError> {
        // Import functionality: load a .gom file and execute it to populate variables
        // The imported variables become available in current scope
        
        let file_path = format!("{}.gom", import_stmt.name);
        
        // Read the file content
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| DreamberdError::InterpretationError(
                format!("Failed to import file '{}': {}", file_path, e)
            ))?;
        
        // Parse and execute the imported file
        let tokens = match crate::processor::lexer::tokenize(&file_path, &content) {
            Ok(tokens) => tokens,
            Err(e) => return Err(DreamberdError::InterpretationError(
                format!("Failed to parse imported file '{}': {:?}", file_path, e)
            )),
        };
        
        let statements = match crate::processor::syntax_tree::generate_syntax_tree(
            &file_path,
            tokens,
            &content,
        ) {
            Ok(statements) => statements,
            Err(e) => return Err(DreamberdError::InterpretationError(
                format!("Failed to generate syntax tree for imported file '{}': {:?}", file_path, e)
            )),
        };
        
        // Execute the imported statements in current context
        // This will populate our variable namespace with imported variables
        self.interpret_code_statements(statements)?;
        
        println!("Successfully imported variables from '{}'", file_path);
        Ok(())
    }

    /// Execute export statement - saves a variable to another file
    fn execute_export_statement(
        &mut self,
        export_stmt: crate::processor::syntax_tree::ExportStatement,
    ) -> Result<(), DreamberdError> {
        // Export functionality: serialize a variable and write it to a file
        // The target file will contain Gulf of Mexico code that recreates the variable
        
        // Look up the variable in our namespace
        let variable_name = &export_stmt.name;
        let target_file = &export_stmt.target_file;
        
        // Find the variable in our current namespaces (search from innermost to outermost)
        let mut variable_value = None;
        for namespace in self.namespaces.iter().rev() {
            if let Some(NamespaceEntry::Variable(var)) = namespace.get(variable_name) {
                variable_value = var.value();
                break;
            }
        }
        
        let variable_value = variable_value
            .ok_or_else(|| DreamberdError::InterpretationError(
                format!("Cannot export undefined variable '{}'", variable_name)
            ))?;
        
        // Generate Gulf of Mexico code to recreate this variable
        let export_code = match variable_value {
            DreamberdValue::Number(num) => {
                format!("const {} = {}!", variable_name, num.value)
            }
            DreamberdValue::String(s) => {
                format!("const {} = \"{}\"!", variable_name, s.value.replace('"', "\\\""))
            }
            DreamberdValue::Boolean(b) => {
                let bool_val = match b.value {
                    Some(true) => "true",
                    Some(false) => "false", 
                    None => "maybe",
                };
                format!("const {} = {}!", variable_name, bool_val)
            }
            DreamberdValue::List(arr) => {
                // Simple array export - convert each element
                let elements: Vec<String> = arr.values.iter().map(|v| match v {
                    DreamberdValue::Number(n) => n.value.to_string(),
                    DreamberdValue::String(s) => format!("\"{}\"", s.value.replace('"', "\\\"")),
                    DreamberdValue::Boolean(b) => match b.value {
                        Some(true) => "true".to_string(),
                        Some(false) => "false".to_string(),
                        None => "maybe".to_string(),
                    },
                    _ => "undefined".to_string(),
                }).collect();
                format!("const {} = [{}]!", variable_name, elements.join(", "))
            }
            DreamberdValue::Map(map) => {
                // Export as object constructor calls
                let mut object_code = format!("const {} = new Object()!\n", variable_name);
                for (key, value) in &map.values {
                    match value {
                        DreamberdValue::Number(n) => {
                            object_code.push_str(&format!("{}[\"{}\"] = {}!\n", variable_name, key, n.value));
                        }
                        DreamberdValue::String(s) => {
                            object_code.push_str(&format!("{}[\"{}\"] = \"{}\"!\n", variable_name, key, s.value.replace('"', "\\\"")));
                        }
                        DreamberdValue::Boolean(b) => {
                            let bool_val = match b.value {
                                Some(true) => "true",
                                Some(false) => "false",
                                None => "maybe",
                            };
                            object_code.push_str(&format!("{}[\"{}\"] = {}!\n", variable_name, key, bool_val));
                        }
                        _ => {
                            object_code.push_str(&format!("{}[\"{}\"] = undefined!\n", variable_name, key));
                        }
                    }
                }
                object_code.trim_end().to_string()
            }
            _ => {
                format!("const {} = undefined!", variable_name)
            }
        };
        
        // Write to target file
        std::fs::write(target_file, export_code)
            .map_err(|e| DreamberdError::InterpretationError(
                format!("Failed to export to file '{}': {}", target_file, e)
            ))?;
        
        println!("Successfully exported variable '{}' to '{}'", variable_name, target_file);
        Ok(())
    }

    /// Execute reverse statement - reverses the effects of previous operations
    fn execute_reverse_statement(&mut self) -> Result<(), DreamberdError> {
        // Reverse functionality: undo the most recent variable assignment or operation
        // In Gulf of Mexico, reverse statements can undo variable changes and operations
        
        // For simplicity, we'll implement reverse as restoring the previous value of 
        // the most recently modified variable
        
        // Find the most recently modified variable across all namespaces
        let mut most_recent_var: Option<(String, String)> = None; // (namespace_key, var_name)
        
        for namespace in &mut self.namespaces {
            for (var_name, entry) in namespace.iter_mut() {
                if let NamespaceEntry::Variable(var) = entry {
                    // Check if this variable has previous values to restore
                    if !var.prev_values.is_empty() {
                        most_recent_var = Some((var_name.clone(), var_name.clone()));
                        break;
                    }
                }
            }
            if most_recent_var.is_some() {
                break;
            }
        }
        
        if let Some((_, var_name)) = most_recent_var {
            // Find the variable and restore its previous value
            for namespace in &mut self.namespaces {
                if let Some(NamespaceEntry::Variable(var)) = namespace.get_mut(&var_name) {
                    if let Some(prev_value) = var.prev_values.pop() {
                        // Restore the previous value
                        if let Some(lifetime) = var.lifetimes.first_mut() {
                            lifetime.value = prev_value;
                            println!("Reversed variable '{}' to previous value", var_name);
                            return Ok(());
                        }
                    }
                }
            }
        }
        
        println!("No operations to reverse");
        Ok(())
    }
}
