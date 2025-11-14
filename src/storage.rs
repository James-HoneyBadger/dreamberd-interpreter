use crate::base::DreamberdError;
use crate::builtin::{DreamberdValue, DreamberdNumber, DreamberdString, DreamberdBoolean, DreamberdList, DreamberdUndefined};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Get the path to the immutable globals storage directory
fn get_storage_dir() -> Result<PathBuf, DreamberdError> {
    let home = std::env::var("HOME").map_err(|_| {
        DreamberdError::InterpretationError("Unable to determine home directory".to_string())
    })?;
    
    let storage_path = PathBuf::from(home).join(".dreamberd_runtime");
    
    // Create directory if it doesn't exist
    if !storage_path.exists() {
        fs::create_dir_all(&storage_path).map_err(|e| {
            DreamberdError::InterpretationError(format!("Failed to create storage directory: {}", e))
        })?;
    }
    
    Ok(storage_path)
}

/// Get the path to the immutable globals file
fn get_immutable_globals_file() -> Result<PathBuf, DreamberdError> {
    Ok(get_storage_dir()?.join("immutable_globals.json"))
}

/// Store an immutable global variable to persistent storage
pub fn store_immutable_global(
    name: &str,
    value: &DreamberdValue,
    confidence: Option<f64>,
) -> Result<(), DreamberdError> {
    let file_path = get_immutable_globals_file()?;
    
    // Load existing globals
    let mut globals: HashMap<String, serde_json::Value> = if file_path.exists() {
        let content = fs::read_to_string(&file_path).map_err(|e| {
            DreamberdError::InterpretationError(format!("Failed to read immutable globals: {}", e))
        })?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };
    
    // Serialize the value with confidence
    let entry = serde_json::json!({
        "value": serialize_value(value),
        "confidence": confidence
    });
    
    globals.insert(name.to_string(), entry);
    
    // Write back to file
    let content = serde_json::to_string_pretty(&globals).map_err(|e| {
        DreamberdError::InterpretationError(format!("Failed to serialize immutable globals: {}", e))
    })?;
    
    fs::write(&file_path, content).map_err(|e| {
        DreamberdError::InterpretationError(format!("Failed to write immutable globals: {}", e))
    })?;
    
    Ok(())
}

/// Load all immutable globals from persistent storage
pub fn load_immutable_globals() -> Result<HashMap<String, (DreamberdValue, Option<f64>)>, DreamberdError> {
    let file_path = get_immutable_globals_file()?;
    
    if !file_path.exists() {
        return Ok(HashMap::new());
    }
    
    let content = fs::read_to_string(&file_path).map_err(|e| {
        DreamberdError::InterpretationError(format!("Failed to read immutable globals: {}", e))
    })?;
    
    let globals: HashMap<String, serde_json::Value> = serde_json::from_str(&content)
        .unwrap_or_default();
    
    let mut result = HashMap::new();
    
    for (name, entry) in globals {
        if let Some(obj) = entry.as_object() {
            let value = if let Some(v) = obj.get("value") {
                deserialize_value(v)
            } else {
                DreamberdValue::Undefined(crate::builtin::DreamberdUndefined)
            };
            
            let confidence = obj.get("confidence").and_then(|c| c.as_f64());
            
            result.insert(name, (value, confidence));
        }
    }
    
    Ok(result)
}

/// Serialize a DreamberdValue to JSON
fn serialize_value(value: &DreamberdValue) -> serde_json::Value {
    match value {
        DreamberdValue::Number(n) => serde_json::json!({"type": "number", "value": n.value}),
        DreamberdValue::String(s) => serde_json::json!({"type": "string", "value": s.value}),
        DreamberdValue::Boolean(b) => serde_json::json!({"type": "boolean", "value": b.value}),
        DreamberdValue::Undefined(_) => serde_json::json!({"type": "undefined"}),
        DreamberdValue::Function { .. } => serde_json::json!({"type": "function"}),
        DreamberdValue::Object { .. } => serde_json::json!({"type": "object"}),
        DreamberdValue::List(arr) => {
            let values: Vec<serde_json::Value> = arr.values.iter().map(serialize_value).collect();
            serde_json::json!({"type": "list", "value": values})
        }
        _ => serde_json::json!({"type": "unknown"}),
    }
}

/// Deserialize a DreamberdValue from JSON
fn deserialize_value(json: &serde_json::Value) -> DreamberdValue {
    if let Some(obj) = json.as_object() {
        match obj.get("type").and_then(|t| t.as_str()) {
            Some("number") => {
                let n = obj.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                DreamberdValue::Number(DreamberdNumber::new(n))
            }
            Some("string") => {
                let s = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
                DreamberdValue::String(DreamberdString::new(s.to_string()))
            }
            Some("boolean") => {
                let b = obj.get("value").and_then(|v| v.as_bool()).unwrap_or(false);
                DreamberdValue::Boolean(DreamberdBoolean::new(Some(b)))
            }
            Some("list") => {
                let values = obj.get("value")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(deserialize_value).collect())
                    .unwrap_or_default();
                DreamberdValue::List(DreamberdList::new(values))
            }
            _ => DreamberdValue::Undefined(DreamberdUndefined),
        }
    } else {
        DreamberdValue::Undefined(DreamberdUndefined)
    }
}
