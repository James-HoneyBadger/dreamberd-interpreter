// Built-in types and values for GulfOfMexico
// Rust port of dreamberd/builtin.py

use std::collections::HashMap;
// use std::fmt;

use crate::processor::syntax_tree::CodeStatement;

pub const FLOAT_TO_INT_PREC: f64 = 0.00000001;

/// Check if a float is effectively an integer
pub fn is_int(x: f64) -> bool {
    let remainder = x % 1.0;
    remainder.abs() < FLOAT_TO_INT_PREC || (1.0 - remainder).abs() < FLOAT_TO_INT_PREC
}

/// Gulf of Mexico value types
#[derive(Debug, Clone)]
pub enum DreamberdValue {
    Number(DreamberdNumber),
    String(DreamberdString),
    Boolean(DreamberdBoolean),
    List(DreamberdList),
    Map(DreamberdMap),
    Function(DreamberdFunction),
    BuiltinFunction(BuiltinFunction),
    Object(DreamberdObject),
    Undefined(DreamberdUndefined),
    SpecialBlank(DreamberdSpecialBlankValue),
    Keyword(DreamberdKeyword),
    Promise(DreamberdPromise),
}

impl PartialEq for DreamberdValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DreamberdValue::Number(a), DreamberdValue::Number(b)) => a == b,
            (DreamberdValue::String(a), DreamberdValue::String(b)) => a == b,
            (DreamberdValue::Boolean(a), DreamberdValue::Boolean(b)) => a == b,
            (DreamberdValue::List(a), DreamberdValue::List(b)) => a == b,
            (DreamberdValue::Map(a), DreamberdValue::Map(b)) => a == b,
            (DreamberdValue::Function(a), DreamberdValue::Function(b)) => a == b,
            (DreamberdValue::BuiltinFunction(a), DreamberdValue::BuiltinFunction(b)) => a == b,
            (DreamberdValue::Object(a), DreamberdValue::Object(b)) => a == b,
            (DreamberdValue::Undefined(a), DreamberdValue::Undefined(b)) => a == b,
            (DreamberdValue::SpecialBlank(a), DreamberdValue::SpecialBlank(b)) => a == b,
            (DreamberdValue::Keyword(a), DreamberdValue::Keyword(b)) => a == b,
            (DreamberdValue::Promise(a), DreamberdValue::Promise(b)) => a == b,
            _ => false,
        }
    }
}

impl DreamberdValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            DreamberdValue::Number(_) => "Number",
            DreamberdValue::String(_) => "String",
            DreamberdValue::Boolean(_) => "Boolean",
            DreamberdValue::List(_) => "List",
            DreamberdValue::Map(_) => "Map",
            DreamberdValue::Function(_) => "Function",
            DreamberdValue::BuiltinFunction(_) => "BuiltinFunction",
            DreamberdValue::Object(_) => "Object",
            DreamberdValue::Undefined(_) => "Undefined",
            DreamberdValue::SpecialBlank(_) => "SpecialBlank",
            DreamberdValue::Keyword(_) => "Keyword",
            DreamberdValue::Promise(_) => "Promise",
        }
    }
}

/// Number type
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdNumber {
    pub value: f64,
}

impl DreamberdNumber {
    pub fn new(value: f64) -> Self {
        DreamberdNumber { value }
    }
}

/// String type with indexing support
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdString {
    pub value: String,
    pub indexer: HashMap<i64, (usize, String)>, // Maps user index to (position, remaining)
}

impl DreamberdString {
    pub fn new(value: String) -> Self {
        let mut indexer = HashMap::new();
        // Initialize indexer for -1 based indexing
        for (i, _) in value.chars().enumerate() {
            indexer.insert(i as i64 - 1, (i, String::new()));
        }
        DreamberdString { value, indexer }
    }
}

impl std::fmt::Display for DreamberdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Boolean type (can be true, false, or maybe)
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdBoolean {
    pub value: Option<bool>, // None represents "maybe"
}

impl DreamberdBoolean {
    pub fn new(value: Option<bool>) -> Self {
        DreamberdBoolean { value }
    }

    pub fn true_val() -> Self {
        DreamberdBoolean::new(Some(true))
    }

    pub fn false_val() -> Self {
        DreamberdBoolean::new(Some(false))
    }

    pub fn maybe() -> Self {
        DreamberdBoolean::new(None)
    }
}

/// List type with custom indexing
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdList {
    pub values: Vec<DreamberdValue>,
    pub indexer: HashMap<String, usize>, // Maps user index (as string) to actual index
}

impl DreamberdList {
    pub fn new(values: Vec<DreamberdValue>) -> Self {
        let mut indexer = HashMap::new();
        // Initialize with -1 based indexing
        for (i, _) in values.iter().enumerate() {
            indexer.insert((i as i64 - 1).to_string(), i);
        }
        DreamberdList { values, indexer }
    }

    /// Get a value by index (supports both integer and float indices)
    pub fn get(&self, index: f64) -> Option<&DreamberdValue> {
        let index_str = if index.fract() == 0.0 {
            (index as i64).to_string()
        } else {
            index.to_string()
        };
        
        if let Some(&actual_index) = self.indexer.get(&index_str) {
            self.values.get(actual_index)
        } else {
            None
        }
    }

    /// Insert a value at a fractional index
    pub fn insert_at(&mut self, index: f64, value: DreamberdValue) {
        let index_str = if index.fract() == 0.0 {
            (index as i64).to_string()
        } else {
            index.to_string()
        };

        // If it's an existing integer index, replace the value
        if let Some(&actual_index) = self.indexer.get(&index_str) {
            if actual_index < self.values.len() {
                self.values[actual_index] = value;
                return;
            }
        }

        // For fractional indices, insert at the appropriate position
        if index.fract() != 0.0 {
            let base_index = index.floor() as i64;
            let base_index_str = base_index.to_string();
            
            // Find where to insert based on the base index
            let insert_pos = if let Some(&actual_base) = self.indexer.get(&base_index_str) {
                actual_base + 1
            } else {
                // If base index doesn't exist, append at the end
                self.values.len()
            };

            // Insert the value
            self.values.insert(insert_pos, value);
            
            // Update indexer - shift all indices after the insertion point
            let mut new_indexer = HashMap::new();
            for (key, &pos) in &self.indexer {
                if pos >= insert_pos {
                    new_indexer.insert(key.clone(), pos + 1);
                } else {
                    new_indexer.insert(key.clone(), pos);
                }
            }
            
            // Add the new fractional index
            new_indexer.insert(index_str, insert_pos);
            self.indexer = new_indexer;
        } else {
            // Integer index - add as new entry
            let actual_index = self.values.len();
            self.values.push(value);
            self.indexer.insert(index_str, actual_index);
        }
    }
}

/// Map/dictionary type
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdMap {
    pub values: HashMap<String, DreamberdValue>,
}

impl DreamberdMap {
    pub fn new() -> Self {
        DreamberdMap {
            values: HashMap::new(),
        }
    }
}

/// Function type
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdFunction {
    pub name: String,
    pub args: Vec<String>,
    pub code: Vec<CodeStatement>,
    pub is_async: bool,
}

/// Built-in function type
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub arg_count: i32, // -1 means variadic
    pub function: fn(&[DreamberdValue]) -> Option<DreamberdValue>,
}

impl PartialEq for BuiltinFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arg_count == other.arg_count
        // Note: We can't compare function pointers, so we just compare name and arg_count
    }
}

/// Object type (class instances)
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdObject {
    pub class_name: String,
    pub namespace: HashMap<String, DreamberdValue>,
}

/// Undefined type
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdUndefined;

/// Special blank value (for optional parameters)
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdSpecialBlankValue;

/// Keyword type (for reassignable keywords)
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdKeyword {
    pub value: String,
}

/// Promise type (for async/next operations)
#[derive(Debug, Clone, PartialEq)]
pub struct DreamberdPromise {
    pub resolved: bool,
    pub value: Option<Box<DreamberdValue>>,
}

/// Variable lifetime information
#[derive(Debug, Clone, PartialEq)]
pub struct VariableLifetime {
    pub value: DreamberdValue,
    pub duration: u64, // Lines or time units
    pub confidence: i32,
    pub can_be_reset: bool,
    pub can_edit_value: bool,
    pub created_at: std::time::Instant,
    pub created_line: u64, // Line number when created
    pub is_time_based: bool, // true for time-based (seconds), false for line-based
}

impl VariableLifetime {
    pub fn new(value: DreamberdValue, duration: u64, confidence: i32, can_be_reset: bool, can_edit_value: bool) -> Self {
        VariableLifetime {
            value,
            duration,
            confidence,
            can_be_reset,
            can_edit_value,
            created_at: std::time::Instant::now(),
            created_line: 0, // Will be set by interpreter
            is_time_based: false, // Default to line-based
        }
    }

    pub fn new_with_tracking(
        value: DreamberdValue, 
        duration: u64, 
        confidence: i32, 
        can_be_reset: bool, 
        can_edit_value: bool,
        current_line: u64,
        is_time_based: bool
    ) -> Self {
        VariableLifetime {
            value,
            duration,
            confidence,
            can_be_reset,
            can_edit_value,
            created_at: std::time::Instant::now(),
            created_line: current_line,
            is_time_based,
        }
    }

    /// Check if this lifetime has expired
    pub fn is_expired(&self, current_line: u64) -> bool {
        if self.is_time_based {
            // Time-based expiration (duration in seconds)
            self.created_at.elapsed().as_secs() >= self.duration
        } else {
            // Line-based expiration
            current_line >= self.created_line + self.duration
        }
    }
}

/// Variable with multiple lifetimes
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub lifetimes: Vec<VariableLifetime>,
    pub prev_values: Vec<DreamberdValue>,
}

impl Variable {
    pub fn new(name: String, lifetime: VariableLifetime) -> Self {
        Variable {
            name,
            lifetimes: vec![lifetime],
            prev_values: Vec::new(),
        }
    }

    pub fn value(&self) -> Option<&DreamberdValue> {
        self.lifetimes.first().map(|lt| &lt.value)
    }

    pub fn set_value(&mut self, new_value: DreamberdValue) {
        if let Some(lifetime) = self.lifetimes.first_mut() {
            if lifetime.can_edit_value {
                // Store the current value as a previous value before updating
                self.prev_values.push(lifetime.value.clone());
                lifetime.value = new_value;
            }
        }
    }

    /// Get previous value at specified index (0 = most recent previous)
    pub fn get_previous(&self, index: usize) -> Option<&DreamberdValue> {
        self.prev_values.iter().rev().nth(index)
    }

    /// Get current value (alias for value())
    pub fn get_current(&self) -> Option<&DreamberdValue> {
        self.value()
    }
}

/// Name binding (immutable reference to a value)
#[derive(Debug, Clone)]
pub struct Name {
    pub name: String,
    pub value: DreamberdValue,
}

impl Name {
    pub fn new(name: String, value: DreamberdValue) -> Self {
        Name { name, value }
    }
}

/// Namespace entry (can be either a Variable or a Name)
#[derive(Debug, Clone)]
pub enum NamespaceEntry {
    Variable(Variable),
    Name(Name),
}

/// Type conversion functions
pub fn db_to_boolean(value: &DreamberdValue) -> DreamberdBoolean {
    match value {
        DreamberdValue::Boolean(b) => b.clone(),
        DreamberdValue::Number(n) => {
            if n.value == 0.0 {
                DreamberdBoolean::false_val()
            } else {
                DreamberdBoolean::true_val()
            }
        }
        DreamberdValue::String(s) => {
            if s.value.is_empty() {
                DreamberdBoolean::false_val()
            } else {
                DreamberdBoolean::true_val()
            }
        }
        DreamberdValue::Undefined(_) => DreamberdBoolean::false_val(),
        DreamberdValue::List(l) => {
            if l.values.is_empty() {
                DreamberdBoolean::false_val()
            } else {
                DreamberdBoolean::true_val()
            }
        }
        _ => DreamberdBoolean::maybe(),
    }
}

pub fn db_to_number(value: &DreamberdValue) -> DreamberdNumber {
    match value {
        DreamberdValue::Number(n) => n.clone(),
        DreamberdValue::Boolean(b) => match b.value {
            Some(true) => DreamberdNumber::new(1.0),
            Some(false) => DreamberdNumber::new(0.0),
            None => DreamberdNumber::new(0.5), // maybe = 0.5
        },
        DreamberdValue::String(s) => {
            s.value.parse::<f64>().map(DreamberdNumber::new).unwrap_or(DreamberdNumber::new(0.0))
        }
        _ => DreamberdNumber::new(0.0),
    }
}

pub fn db_to_string(value: &DreamberdValue) -> DreamberdString {
    match value {
        DreamberdValue::String(s) => s.clone(),
        DreamberdValue::Number(n) => {
            if is_int(n.value) {
                DreamberdString::new(format!("{}", n.value as i64))
            } else {
                DreamberdString::new(format!("{}", n.value))
            }
        }
        DreamberdValue::Boolean(b) => match b.value {
            Some(true) => DreamberdString::new("true".to_string()),
            Some(false) => DreamberdString::new("false".to_string()),
            None => DreamberdString::new("maybe".to_string()),
        },
        DreamberdValue::List(list) => {
            let elements: Vec<String> = list.values
                .iter()
                .map(|v| db_to_string(v).value)
                .collect();
            DreamberdString::new(format!("[{}]", elements.join(", ")))
        }
        DreamberdValue::Undefined(_) => DreamberdString::new("undefined".to_string()),
        _ => DreamberdString::new(format!("{:?}", value)),
    }
}

pub fn db_not(value: &DreamberdBoolean) -> DreamberdBoolean {
    match value.value {
        Some(true) => DreamberdBoolean::false_val(),
        Some(false) => DreamberdBoolean::true_val(),
        None => DreamberdBoolean::maybe(),
    }
}

/// Create a builtin function
pub fn create_builtin_function(name: &str, arg_count: i32, function: fn(&[DreamberdValue]) -> Option<DreamberdValue>) -> BuiltinFunction {
    BuiltinFunction {
        name: name.to_string(),
        arg_count,
        function,
    }
}
