// Serialization support for persistent variables
// Rust port of dreamberd/serialize.py

use serde::{Deserialize, Serialize};
use crate::builtin::DreamberdValue;

/// Serialize a DreamBerd value to JSON
pub fn serialize_obj(value: &DreamberdValue) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserialize a DreamBerd value from JSON
pub fn deserialize_obj(json: &str) -> Result<DreamberdValue, serde_json::Error> {
    serde_json::from_str(json)
}
