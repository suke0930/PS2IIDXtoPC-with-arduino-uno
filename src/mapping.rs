use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::types::{
    KeyboardButtonEntry, KeyboardMapping, KeyboardSpecialConfig, MappingConfig, OutputType,
    X360ButtonEntry, X360Mapping,
};

pub fn default_maps() -> HashMap<&'static str, &'static str> {
    let mut maps = HashMap::new();
    maps.insert("iidx", "mapping/iidx.keyboard.json");
    maps.insert("popn", "mapping/popn.keyboard.json");
    maps.insert("x360", "mapping/x360.pad.json");
    maps
}

pub fn load_mapping(file_path: &str) -> Result<MappingConfig, String> {
    let resolved = if Path::new(file_path).is_absolute() {
        PathBuf::from(file_path)
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join(file_path)
    };

    let raw_text = fs::read_to_string(&resolved)
        .map_err(|e| format!("Failed to read mapping file {}: {}", resolved.display(), e))?;

    let parsed: Value = serde_json::from_str(&raw_text)
        .map_err(|e| format!("Failed to parse mapping JSON at {}: {}", resolved.display(), e))?;

    let obj = parsed
        .as_object()
        .ok_or_else(|| format!("Mapping JSON must be an object ({})", resolved.display()))?;

    let output = obj
        .get("output")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Mapping must have \"output\" field ({})", resolved.display()))?;

    let buttons_value = obj
        .get("buttons")
        .ok_or_else(|| format!("Mapping must have \"buttons\" field ({})", resolved.display()))?;

    let buttons_obj = buttons_value
        .as_object()
        .ok_or_else(|| format!("\"buttons\" must be an object ({})", resolved.display()))?;

    let name = obj.get("name").and_then(|v| v.as_str()).map(String::from);

    match output {
        "keyboard" => {
            let mut buttons = HashMap::new();
            for (key_id, value) in buttons_obj {
                let entry = normalize_keyboard_entry(value, key_id)?;
                buttons.insert(key_id.clone(), entry);
            }

            let special = if let Some(special_val) = obj.get("special") {
                Some(parse_keyboard_special(special_val)?)
            } else {
                None
            };

            Ok(MappingConfig::Keyboard(KeyboardMapping {
                name,
                output: OutputType::Keyboard,
                buttons,
                special,
            }))
        }
        "x360" => {
            let mut buttons = HashMap::new();
            for (key_id, value) in buttons_obj {
                let entry: X360ButtonEntry = serde_json::from_value(value.clone())
                    .map_err(|e| format!("Invalid x360 mapping entry for button {}: {}", key_id, e))?;
                buttons.insert(key_id.clone(), entry);
            }

            Ok(MappingConfig::X360(X360Mapping {
                name,
                output: OutputType::X360,
                buttons,
            }))
        }
        other => Err(format!(
            "Mapping output must be \"keyboard\" or \"x360\", got \"{}\" ({})",
            other,
            resolved.display()
        )),
    }
}

fn normalize_keyboard_entry(value: &Value, key_id: &str) -> Result<KeyboardButtonEntry, String> {
    if let Some(s) = value.as_str() {
        return Ok(KeyboardButtonEntry { key: s.to_string() });
    }
    if let Some(obj) = value.as_object() {
        if let Some(key) = obj.get("key").and_then(|v| v.as_str()) {
            return Ok(KeyboardButtonEntry {
                key: key.to_string(),
            });
        }
    }
    Err(format!(
        "Invalid keyboard mapping entry for button {}",
        key_id
    ))
}

fn parse_keyboard_special(value: &Value) -> Result<KeyboardSpecialConfig, String> {
    let obj = value
        .as_object()
        .ok_or("\"special\" must be an object")?;

    let ignore_key = obj.get("ignoreKey").and_then(|v| v.as_str()).map(String::from);

    let tap_keys = obj.get("tapKeys").and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
    });

    let tap_duration_ms = obj
        .get("tapDurationMs")
        .and_then(|v| v.as_u64());

    let release_on_ignore = obj.get("releaseOnIgnore").and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
    });

    Ok(KeyboardSpecialConfig {
        ignore_key,
        tap_keys,
        tap_duration_ms,
        release_on_ignore,
    })
}
