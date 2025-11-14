use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};

lazy_static! {
    static ref ALIAS_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize)]
struct AliasPersistence {
    aliases: HashMap<String, String>,
}

const CANONICAL_KEYWORDS: &[&str] = &[
    "function","func","fun","fn","if","when","after","class","return","delete","export","import","reverse","var","const"
];

fn aliases_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut p = PathBuf::from(home);
    p.push(".dreamberd_runtime");
    if !p.exists() { let _ = fs::create_dir_all(&p); }
    p.push("aliases.json");
    p
}

pub fn load_aliases() {
    let path = aliases_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(data) = serde_json::from_str::<AliasPersistence>(&content) {
            let mut map = ALIAS_MAP.lock().unwrap();
            *map = data.aliases;
        }
    }
}

fn save_aliases() {
    let path = aliases_path();
    if let Ok(map) = ALIAS_MAP.lock() {
        let data = AliasPersistence { aliases: map.clone() };
        if let Ok(json) = serde_json::to_string_pretty(&data) { let _ = fs::write(path, json); }
    }
}

pub fn alias(original: &str, new_name: &str) -> bool {
    if !CANONICAL_KEYWORDS.contains(&original) { return false; }
    if new_name.is_empty() || new_name.len() > 32 || !new_name.chars().all(|c| c.is_ascii_alphabetic()) { return false; }
    let mut map = ALIAS_MAP.lock().unwrap();
    if CANONICAL_KEYWORDS.contains(&new_name) { return false; }
    if map.contains_key(new_name) { return false; }
    // Prevent canonical -> alias collision (no chaining)
    if map.values().any(|v| v == new_name) { return false; }
    map.insert(new_name.to_string(), original.to_string());
    drop(map);
    save_aliases();
    true
}

pub fn unalias(name: &str) -> bool {
    let mut map = ALIAS_MAP.lock().unwrap();
    let existed = map.remove(name).is_some();
    drop(map);
    if existed { save_aliases(); }
    existed
}

pub fn list_aliases() -> HashMap<String, String> {
    ALIAS_MAP.lock().unwrap().clone()
}

pub fn canonicalize_tokens(tokens: &mut [crate::base::Token]) {
    let map = ALIAS_MAP.lock().unwrap();
    for t in tokens.iter_mut() {
        if t.token_type == crate::base::TokenType::Name {
            if let Some(canon) = map.get(&t.value) {
                t.value = canon.clone();
            }
        }
    }
}

// Convenience boolean helpers
pub fn bool_value(b: bool) -> crate::builtin::DreamberdValue {
    use crate::builtin::{DreamberdValue, DreamberdBoolean};
    DreamberdValue::Boolean(if b { DreamberdBoolean::true_val() } else { DreamberdBoolean::false_val() })
}
