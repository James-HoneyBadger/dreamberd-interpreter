// Main interpreter engine for GulfOfMexico
// Rust port of dreamberd/interpreter.py

use std::collections::{HashMap, VecDeque};
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::base::{DreamberdError, TokenType, OperatorType};
use crate::builtin::{DreamberdValue, Variable, Name, NamespaceEntry, VariableLifetime, DreamberdFunction, DreamberdUndefined, DreamberdNumber, DreamberdString, DreamberdBoolean, DreamberdList, DreamberdMap, DreamberdObject, create_builtin_function, db_to_string, db_to_boolean, db_not};
use crate::alias::{load_aliases, alias as alias_set, unalias as alias_remove, list_aliases, canonicalize_tokens, bool_value};
use crate::processor::syntax_tree::CodeStatement;
use crate::processor::expression_tree::{ExpressionTreeNode, ValueNode, ConstructorNode};
// Event input types for after-statements
#[derive(Debug, Clone)]
enum InputEvent {
    KeyDown(String),
    KeyUp(String),
    MouseClick(String),
    MouseRelease(String),
}

// Global synthetic event sender for trigger() builtin
lazy_static! {
    static ref SYNTHETIC_EVENT_SENDER: Mutex<Option<crossbeam_channel::Sender<InputEvent>>> = Mutex::new(None);
}

pub type Namespace = HashMap<String, NamespaceEntry>;

/// Reactive when statement registration
#[derive(Debug, Clone)]
pub struct ReactiveWhen {
    pub condition: crate::processor::expression_tree::ExpressionTreeNode,
    pub body: Vec<crate::processor::syntax_tree::CodeStatement>,
    pub dependent_variables: Vec<String>,
}

/// Main interpreter struct
pub struct Interpreter {
    pub namespaces: Vec<Namespace>,
    pub filename: String,
    pub code: String,
    pub current_line: u64,
    pub when_statements: Vec<ReactiveWhen>,
    pub after_statements: VecDeque<crate::processor::syntax_tree::AfterStatement>,
    // Event channel receiver for input events
    event_rx: Option<crossbeam_channel::Receiver<InputEvent>>,
}

impl Interpreter {
    pub fn new(filename: String, code: String) -> Self {
        let mut interpreter = Interpreter {
            namespaces: vec![HashMap::new()],
            filename,
            code,
            current_line: 1,
            when_statements: Vec::new(),
            after_statements: VecDeque::new(),
            event_rx: None,
        };

        // Initialize builtin functions and keywords
        interpreter.initialize_builtins();
        // Load persisted keyword aliases
        load_aliases();
        
        // Load immutable globals from persistent storage
        interpreter.load_immutable_globals();

        // Start input event listener thread
        interpreter.initialize_event_loop();
        
        interpreter
    }

    /// Initialize global input event loop (keyboard & mouse)
    fn initialize_event_loop(&mut self) {
        use std::thread;
    use crossbeam_channel::unbounded;

    let (tx, rx) = unbounded();
        self.event_rx = Some(rx);
    if let Ok(mut global) = SYNTHETIC_EVENT_SENDER.lock() { *global = Some(tx.clone()); }

        thread::spawn(move || {
            // Use rdev for global event listening
            if let Err(err) = rdev::listen(move |event| {
                let mapped = match event.event_type {
                    rdev::EventType::KeyPress(key) => Some(InputEvent::KeyDown(format!("{:?}", key))),
                    rdev::EventType::KeyRelease(key) => Some(InputEvent::KeyUp(format!("{:?}", key))),
                    rdev::EventType::ButtonPress(button) => Some(InputEvent::MouseClick(format!("{:?}", button))),
                    rdev::EventType::ButtonRelease(button) => Some(InputEvent::MouseRelease(format!("{:?}", button))),
                    _ => None,
                };
                if let Some(ev) = mapped {
                    // Ignore send errors (receiver dropped)
                    let _ = tx.send(ev);
                }
            }) {
                eprintln!("Failed to start input listener: {:?}", err);
            }
        });
    }

    /// Poll pending input events and execute matching after statements
    fn poll_input_events(&mut self) -> Result<(), DreamberdError> {
        if let Some(rx) = &self.event_rx {
            // Collect events first to avoid borrow conflicts
            let mut pending = Vec::new();
            while let Ok(ev) = rx.try_recv() {
                pending.push(ev);
            }
            for ev in pending {
                self.process_input_event(ev)?;
            }
        }
        Ok(())
    }

    /// Process a single input event
    fn process_input_event(&mut self, ev: InputEvent) -> Result<(), DreamberdError> {
        // Normalize event string
        let key = match &ev {
            InputEvent::KeyDown(k) => format!("keydown:{}", k),
            InputEvent::KeyUp(k) => format!("keyup:{}", k),
            InputEvent::MouseClick(b) => format!("click:{}", b),
            InputEvent::MouseRelease(b) => format!("release:{}", b),
        };

        // Collect matching after statements (retain those that are persistent)
        // Split matching and remaining to avoid borrow conflicts
        let mut matching = Vec::new();
        let mut remaining = VecDeque::new();
        for stmt in self.after_statements.drain(..) {
            if stmt.event == key || (stmt.event == "click" && key.starts_with("click")) {
                matching.push(stmt);
            } else {
                remaining.push_back(stmt);
            }
        }
        self.after_statements = remaining;
        // Execute matches after we've updated the queue
        for stmt in matching {
            self.interpret_code_statements(stmt.body.clone())?;
        }
        Ok(())
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
            // Alias management builtins
            ("alias", create_builtin_function("alias", 2, |args| {
                if let (Some(DreamberdValue::String(orig)), Some(DreamberdValue::String(newn))) = (args.get(0), args.get(1)) {
                    Some(bool_value(alias_set(&orig.value, &newn.value)))
                } else { Some(bool_value(false)) }
            })),
            ("unalias", create_builtin_function("unalias", 1, |args| {
                if let Some(DreamberdValue::String(name)) = args.get(0) {
                    Some(bool_value(alias_remove(&name.value)))
                } else { Some(bool_value(false)) }
            })),
            ("list_aliases", create_builtin_function("list_aliases", 0, |_args| {
                let map = list_aliases();
                let mut values = HashMap::new();
                for (k,v) in map.into_iter() {
                    values.insert(k, DreamberdValue::String(DreamberdString::new(v)));
                }
                Some(DreamberdValue::Map(DreamberdMap { values }))
            })),
            // Synthetic input event trigger
            ("trigger", create_builtin_function("trigger", 1, |args| {
                use crate::builtin::DreamberdBoolean;
                if let Some(DreamberdValue::String(s)) = args.get(0) {
                    let ev = if let Some(rest) = s.value.strip_prefix("keydown:") { InputEvent::KeyDown(rest.to_string()) }
                        else if let Some(rest) = s.value.strip_prefix("keyup:") { InputEvent::KeyUp(rest.to_string()) }
                        else if let Some(rest) = s.value.strip_prefix("click:") { InputEvent::MouseClick(rest.to_string()) }
                        else if let Some(rest) = s.value.strip_prefix("release:") { InputEvent::MouseRelease(rest.to_string()) }
                        else if s.value == "click" { InputEvent::MouseClick("Synthetic".to_string()) }
                        else { InputEvent::KeyDown(s.value.clone()) };
                    if let Ok(lock) = SYNTHETIC_EVENT_SENDER.lock() {
                        if let Some(tx) = &*lock { let _ = tx.send(ev); return Some(DreamberdValue::Boolean(DreamberdBoolean::true_val())); }
                    }
                    return Some(DreamberdValue::Boolean(DreamberdBoolean::false_val()));
                }
                Some(DreamberdValue::Boolean(DreamberdBoolean::false_val()))
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

    /// Store an immutable global variable (const const const)
    fn store_immutable_global(&self, name: &str, value: &DreamberdValue, confidence: Option<f64>) -> Result<(), DreamberdError> {
        crate::storage::store_immutable_global(name, value, confidence)
    }

    /// Load immutable globals from persistent storage
    fn load_immutable_globals(&mut self) {
        if let Ok(globals) = crate::storage::load_immutable_globals() {
            for (name, (value, confidence)) in globals {
                // Convert confidence back to i32 (0-100 scale)
                let confidence_i32 = confidence.map(|c| (c * 100.0) as i32).unwrap_or(100);
                
                // Create a variable lifetime with maximum lifespan (immutable globals never expire)
                let lifetime = VariableLifetime::new(
                    value,
                    u64::MAX, // Never expires
                    confidence_i32,
                    false,    // Cannot be reset (it's const const const)
                    false,    // Cannot edit value
                );
                
                let variable = Variable::new(name.clone(), lifetime);
                self.namespaces[0].insert(name, NamespaceEntry::Variable(variable));
            }
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
            // Poll input events so reactive after statements can trigger between statements
            self.poll_input_events()?;
            
            result = self.execute_statement(statement)?;
            
            // Increment line counter (approximation - each statement is a line)
            self.current_line += 1;
            
            if result.is_some() {
                break; // Return statement encountered
            }
        }

        self.execute_scheduled_after_statements()?;
        // Final poll after finishing batch
        self.poll_input_events()?;

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

        // Check for immutable global (const const const)
        let is_immutable_global = var_decl.modifiers.len() >= 3 
            && var_decl.modifiers.iter().all(|m| m == "const");
        
        if is_immutable_global {
            // Persist to local storage
            let confidence = if var_decl.confidence == 100 {
                None
            } else {
                Some(var_decl.confidence as f64 / 100.0)
            };
            self.store_immutable_global(&var_decl.name.value, &value, confidence)?;
        }

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
        // Extract variables referenced in the condition
        let dependent_vars = self.extract_variable_dependencies(&when_stmt.condition);
        
        // Register this when statement as reactive
        let reactive_when = ReactiveWhen {
            condition: when_stmt.condition.clone(),
            body: when_stmt.body.clone(),
            dependent_variables: dependent_vars,
        };
        
        // Evaluate the condition initially
        let condition_result = self.evaluate_expression(&when_stmt.condition)?;
        let condition_bool = db_to_boolean(&condition_result);

        // If condition is true or maybe, execute the body immediately
        if let DreamberdBoolean { value: Some(true), .. } = condition_bool {
            self.interpret_code_statements(when_stmt.body.clone())?;
        }
        
        // Register for future reactive execution
        self.when_statements.push(reactive_when);
        
        Ok(())
    }
    
    /// Extract variable names referenced in an expression
    fn extract_variable_dependencies(&self, expr: &crate::processor::expression_tree::ExpressionTreeNode) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables_from_expression(expr, &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }
    
    /// Recursively collect variable names from expression tree
    fn collect_variables_from_expression(
        &self, 
        expr: &crate::processor::expression_tree::ExpressionTreeNode, 
        vars: &mut Vec<String>
    ) {
        use crate::processor::expression_tree::ExpressionTreeNode;
        use crate::base::TokenType;
        
        match expr {
            ExpressionTreeNode::Value(val) => {
                if val.token.token_type == TokenType::Name {
                    vars.push(val.token.value.clone());
                }
            }
            ExpressionTreeNode::BinaryOp(bin_op) => {
                self.collect_variables_from_expression(&bin_op.left, vars);
                self.collect_variables_from_expression(&bin_op.right, vars);
            }
            ExpressionTreeNode::UnaryOp(unary_op) => {
                self.collect_variables_from_expression(&unary_op.expression, vars);
            }
            ExpressionTreeNode::Function(func) => {
                for arg in &func.args {
                    self.collect_variables_from_expression(arg, vars);
                }
            }
            ExpressionTreeNode::Index(index_op) => {
                self.collect_variables_from_expression(&index_op.value, vars);
                self.collect_variables_from_expression(&index_op.index, vars);
            }
            ExpressionTreeNode::List(list) => {
                for item in &list.values {
                    self.collect_variables_from_expression(item, vars);
                }
            }
            ExpressionTreeNode::Constructor(_) => {
                // Constructor calls don't typically reference variables directly
            }
        }
    }

    /// Execute delete statement
    fn execute_delete_statement(
        &mut self,
        del_stmt: crate::processor::syntax_tree::DeleteStatement,
    ) -> Result<(), DreamberdError> {
        match &del_stmt.target {
            ExpressionTreeNode::Value(value_node) => {
                // Check if it's a variable reference
                if let TokenType::Name = value_node.token.token_type {
                    let var_name = &value_node.token.value;
                    // Delete the variable from current namespace
                    let mut removed = false;
                    for namespace in self.namespaces.iter_mut().rev() {
                        if namespace.remove(var_name).is_some() {
                            removed = true;
                        }
                    }
                    if removed {
                        println!("Deleted variable: {}", var_name);
                    }
                } else if let TokenType::String = value_node.token.token_type {
                    // Delete a keyword (in a real implementation, this would affect parsing)
                    println!("Deleted keyword: {}", value_node.token.value);
                    // For now, we can't actually remove language keywords, but we acknowledge it
                } else {
                    // Delete a literal value
                    println!("Deleted literal: {}", value_node.token.value);
                }
                Ok(())
            }
            _ => {
                // Evaluate and "delete" the expression result
                let value = self.evaluate_expression(&del_stmt.target)?;
                match value {
                    DreamberdValue::Number(n) => {
                        println!("Deleted number: {:?}", n);
                        // In DreamBerd, deleting a number could prevent it from existing
                        Ok(())
                    }
                    DreamberdValue::String(s) => {
                        println!("Deleted string: \"{}\"", s);
                        Ok(())
                    }
                    DreamberdValue::Boolean(b) => {
                        println!("Deleted boolean: {:?}", b);
                        Ok(())
                    }
                    _ => {
                        println!("Deleted value: {:?}", value);
                        Ok(())
                    }
                }
            }
        }
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
                            
                            // Canonicalize tokens via alias map before building expression tree
                            let mut tokens = tokens; // make mutable
                            canonicalize_tokens(&mut tokens);
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
        if op.operator == OperatorType::E {
            let right = self.evaluate_expression(&op.right)?;
            match op.left.as_ref() {
                ExpressionTreeNode::Value(value_node) if value_node.token.token_type == crate::base::TokenType::Name => {
                    let var_name = &value_node.token.value;
                    self.assign_variable(var_name, right.clone())?;
                    Ok(right) // Assignment returns the assigned value
                }
                ExpressionTreeNode::Index(index_node) => {
                    self.assign_index(index_node, right.clone())?;
                    Ok(right)
                }
                _ => {
                    let left_value = self.evaluate_expression(&op.left)?;
                    Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(left_value == right))))
                }
            }
        } else {
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
                OperatorType::Ee | OperatorType::Eee | OperatorType::Eeee => {
                    Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(left == right))))
                }
                OperatorType::Ne | OperatorType::Nee | OperatorType::Neee => {
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
                OperatorType::And => {
                    let left_bool = db_to_boolean(&left);
                    let right_bool = db_to_boolean(&right);
                    match (&left_bool.value, &right_bool.value) {
                        (Some(l), Some(r)) => {
                            Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(*l && *r))))
                        }
                        _ => Ok(DreamberdValue::Boolean(DreamberdBoolean::maybe())),
                    }
                }
                OperatorType::Or => {
                    let left_bool = db_to_boolean(&left);
                    let right_bool = db_to_boolean(&right);
                    match (&left_bool.value, &right_bool.value) {
                        (Some(l), Some(r)) => {
                            Ok(DreamberdValue::Boolean(DreamberdBoolean::new(Some(*l || *r))))
                        }
                        _ => Ok(DreamberdValue::Boolean(DreamberdBoolean::maybe())),
                    }
                }
                _ => Ok(DreamberdValue::Undefined(DreamberdUndefined)),
            }
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
                // Extract variable name from the argument
                if let ExpressionTreeNode::Value(value_node) = &func.args[0] {
                    if value_node.token.token_type == TokenType::Name {
                        let var_name = &value_node.token.value;
                        // Look up variable and try to predict next value
                        for namespace in self.namespaces.iter().rev() {
                            if let Some(NamespaceEntry::Variable(var)) = namespace.get(var_name) {
                                // Use pattern analysis to predict next value
                                return Ok(var.get_next().unwrap_or(DreamberdValue::Undefined(DreamberdUndefined)));
                            }
                        }
                        return Ok(DreamberdValue::Undefined(DreamberdUndefined));
                    }
                }
                return Err(DreamberdError::NonFormattedError("next() expects a variable name".to_string()));
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
                        // Trigger reactive when statements that depend on this variable
                        self.trigger_when_statements_for_variable(var_name)?;
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
        self.after_statements.push_back(after_stmt);
        Ok(())
    }

    fn execute_scheduled_after_statements(&mut self) -> Result<(), DreamberdError> {
        while let Some(after_stmt) = self.after_statements.pop_front() {
            println!("Executing scheduled after statement for {}: {}", self.filename, after_stmt.event);
            self.interpret_code_statements(after_stmt.body)?;
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
        // In Gulf of Mexico, classes can only have one instance (singleton pattern)
        let class_name = &class_decl.name;
        
        // Create a class object with empty namespace for methods/properties
        let class_obj = DreamberdObject {
            class_name: class_name.clone(),
            namespace: HashMap::new(),
        };
        
        // Execute class body to populate class methods/properties
        // Create a new namespace scope for the class
        self.namespaces.push(HashMap::new());
        
        // Execute all statements in the class body
        for statement in &class_decl.body {
            self.execute_statement(statement.clone())?;
        }
        
        // Pop the class namespace and merge its contents into the class object
        let class_namespace = self.namespaces.pop().unwrap();
        let mut final_class_obj = class_obj;
        
        // Copy methods and properties from class namespace
        for (name, entry) in class_namespace {
            match entry {
                NamespaceEntry::Variable(var) => {
                    if let Some(value) = var.value() {
                        final_class_obj.namespace.insert(name, value.clone());
                    }
                }
                NamespaceEntry::Name(name_struct) => {
                    final_class_obj.namespace.insert(name, DreamberdValue::String(
                        DreamberdString::new(name_struct.name.clone())
                    ));
                }
            }
        }
        
        // Store the class as a variable that contains the class object
        let class_variable = Variable::new(
            class_name.clone(),
            VariableLifetime::new(
                DreamberdValue::Object(final_class_obj),
                u64::MAX, // Persistent (never expires)
                100, // Full confidence
                false, // Can't be reset
                true, // Can edit value (for adding methods)
            )
        );
        
        // Store the class definition in the current namespace
        if let Some(namespace) = self.namespaces.last_mut() {
            namespace.insert(
                class_name.clone(),
                NamespaceEntry::Variable(class_variable),
            );
        }
        
        println!("Class '{}' declared", class_name);
        
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
            Ok(mut tokens) => { canonicalize_tokens(&mut tokens); tokens },
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
        let export_code = self.serialize_value_to_code(variable_name, variable_value)?;
        
        // Write to target file
        std::fs::write(target_file, export_code)
            .map_err(|e| DreamberdError::InterpretationError(
                format!("Failed to export to file '{}': {}", target_file, e)
            ))?;
        
        println!("Successfully exported variable '{}' to '{}'", variable_name, target_file);
        Ok(())
    }

    /// Serialize a DreamberdValue to Gulf of Mexico code
    fn serialize_value_to_code(&self, var_name: &str, value: &DreamberdValue) -> Result<String, DreamberdError> {
        self.serialize_value_to_expression(value, 0)
            .map(|expr| format!("const {} = {}!", var_name, expr))
    }

    /// Serialize a value to an expression string (helper for nested values)
    fn serialize_value_to_expression(&self, value: &DreamberdValue, depth: usize) -> Result<String, DreamberdError> {
        // Prevent infinite recursion
        if depth > 10 {
            return Ok("undefined".to_string());
        }

        match value {
            DreamberdValue::Number(num) => {
                Ok(num.value.to_string())
            }
            DreamberdValue::String(s) => {
                Ok(format!("\"{}\"", s.value.replace('"', "\\\"")))
            }
            DreamberdValue::Boolean(b) => {
                let bool_val = match b.value {
                    Some(true) => "true",
                    Some(false) => "false", 
                    None => "maybe",
                };
                Ok(bool_val.to_string())
            }
            DreamberdValue::List(arr) => {
                // Recursively serialize array elements
                let elements: Result<Vec<String>, DreamberdError> = arr.values.iter()
                    .map(|v| self.serialize_value_to_expression(v, depth + 1))
                    .collect();
                Ok(format!("[{}]", elements?.join(", ")))
            }
            DreamberdValue::Map(map) => {
                // Export as object literal with properties
                let mut props: Vec<String> = Vec::new();
                for (key, val) in &map.values {
                    let val_expr = self.serialize_value_to_expression(val, depth + 1)?;
                    props.push(format!("\"{}\": {}", key, val_expr));
                }
                Ok(format!("{{{}}}", props.join(", ")))
            }
            DreamberdValue::Function(func) => {
                // Export function definition
                let args = func.args.join(", ");
                let body = if func.code.is_empty() {
                    String::new()
                } else {
                    // For now, just indicate it's a function - proper code serialization
                    // would require converting CodeStatements back to source text
                    format!("/* {} statements */", func.code.len())
                };
                
                if func.is_async {
                    Ok(format!("async function({}) => {{\n  {}\n}}", args, body))
                } else {
                    Ok(format!("function({}) => {{\n  {}\n}}", args, body))
                }
            }
            DreamberdValue::Object(obj) => {
                // Export object with properties
                let mut props: Vec<String> = Vec::new();
                for (key, val) in &obj.namespace {
                    let val_expr = self.serialize_value_to_expression(val, depth + 1)?;
                    props.push(format!("\"{}\": {}", key, val_expr));
                }
                // Include class name and properties
                if props.is_empty() {
                    Ok(format!("new {}()", obj.class_name))
                } else {
                    Ok(format!("new {}() /* with properties: {} */", obj.class_name, props.join(", ")))
                }
            }
            DreamberdValue::Undefined(_) => {
                Ok("undefined".to_string())
            }
            _ => {
                // For unsupported types (BuiltinFunction, SpecialBlank, etc.)
                Ok("undefined".to_string())
            }
        }
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
    
    /// Trigger when statements that depend on a specific variable
    fn trigger_when_statements_for_variable(&mut self, var_name: &str) -> Result<(), DreamberdError> {
        // Clone the when statements to avoid borrow checker issues
        let when_statements = self.when_statements.clone();
        
        for when_stmt in when_statements {
            // Check if this when statement depends on the changed variable
            if when_stmt.dependent_variables.contains(&var_name.to_string()) {
                // Re-evaluate the condition
                let condition_result = self.evaluate_expression(&when_stmt.condition)?;
                let condition_bool = db_to_boolean(&condition_result);

                // If condition is true or maybe, execute the body
                if let DreamberdBoolean { value: Some(true), .. } = condition_bool {
                    self.interpret_code_statements(when_stmt.body)?;
                }
            }
        }
        
        Ok(())
    }

    /// Block and wait for input events, executing matching after-statements until interrupted.
    pub fn wait_for_events(&mut self) -> Result<(), DreamberdError> {
        use std::time::Duration;
        println!("Waiting for input events (after-statements). Press Ctrl+C to exit.");
        loop {
            // Poll input events and execute bodies
            self.poll_input_events()?;
            // Also run any scheduled (time-based) after statements if they were deferred
            self.execute_scheduled_after_statements()?;
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
