// Serialization support for persistent variables
// Rust port of dreamberd/serialize.py

// Note: Serialization is disabled because DreamberdValue contains function pointers
// which cannot be serialized. This could be re-enabled with a custom serialization
// implementation that handles BuiltinFunction specially.

// use serde::{Deserialize, Serialize};
// use crate::builtin::DreamberdValue;

// /// Serialize a Gulf of Mexico value to JSON
// pub fn serialize_obj(value: &DreamberdValue) -> Result<String, serde_json::Error> {
//     serde_json::to_string(value)
// }

// /// Deserialize a Gulf of Mexico value from JSON
// pub fn deserialize_obj(json: &str) -> Result<DreamberdValue, serde_json::Error> {
//     serde_json::from_str(json)
// }
