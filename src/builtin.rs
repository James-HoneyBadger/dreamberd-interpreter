// Built-in types and values for DreamBerd
// Rust port of dreamberd/builtin.py

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::processor::syntax_tree::CodeStatement;

pub const FLOAT_TO_INT_PREC: f64 = 0.00000001;

/// Check if a float is effectively an integer
pub fn is_int(x: f64) -> bool {
    let remainder = x % 1.0;
    remainder.abs() < FLOAT_TO_INT_PREC || (1.0 - remainder).abs() < FLOAT_TO_INT_PREC
}

/// DreamBerd value types
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdNumber {
    pub value: f64,
}

impl DreamberdNumber {
    pub fn new(value: f64) -> Self {
        DreamberdNumber { value }
    }
}

/// String type with indexing support
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Boolean type (can be true, false, or maybe)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdList {
    pub values: Vec<DreamberdValue>,
    pub indexer: HashMap<i64, usize>, // Maps user index to actual index
}

impl DreamberdList {
    pub fn new(values: Vec<DreamberdValue>) -> Self {
        let mut indexer = HashMap::new();
        // Initialize with -1 based indexing
        for (i, _) in values.iter().enumerate() {
            indexer.insert(i as i64 - 1, i);
        }
        DreamberdList { values, indexer }
    }
}

/// Map/dictionary type
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdFunction {
    pub name: String,
    pub args: Vec<String>,
    pub code: Vec<CodeStatement>,
    pub is_async: bool,
}

/// Built-in function type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinFunction {
    pub name: String,
    pub arg_count: i32, // -1 means variadic
}

/// Object type (class instances)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdObject {
    pub class_name: String,
    pub namespace: HashMap<String, DreamberdValue>,
}

/// Undefined type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdUndefined;

/// Special blank value (for optional parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdSpecialBlankValue;

/// Keyword type (for reassignable keywords)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdKeyword {
    pub value: String,
}

/// Promise type (for async/next operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamberdPromise {
    pub resolved: bool,
    pub value: Option<Box<DreamberdValue>>,
}

/// Variable lifetime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableLifetime {
    pub value: DreamberdValue,
    pub duration: u64, // Lines or time units
    pub confidence: i32,
    pub can_be_reset: bool,
    pub can_edit_value: bool,
}

/// Variable with multiple lifetimes
#[derive(Debug, Clone, Serialize, Deserialize)]
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
