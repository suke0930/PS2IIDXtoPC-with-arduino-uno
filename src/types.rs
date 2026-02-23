use serde::Deserialize;
use std::collections::HashMap;

// --- Button Event ---

#[derive(Debug, Clone)]
pub struct ButtonEvent {
    pub id: u8,
    pub pressed: bool,
}

// --- Output Type ---

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    Keyboard,
    X360,
}

// --- Keyboard Mapping ---

#[derive(Debug, Clone, Deserialize)]
pub struct KeyboardButtonEntry {
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardSpecialConfig {
    pub ignore_key: Option<String>,
    pub tap_keys: Option<Vec<String>>,
    pub tap_duration_ms: Option<u64>,
    pub release_on_ignore: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeyboardMapping {
    pub name: Option<String>,
    pub output: OutputType,
    pub buttons: HashMap<String, KeyboardButtonEntry>,
    pub special: Option<KeyboardSpecialConfig>,
}

// --- X360 Mapping ---

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum X360ButtonName {
    Start,
    Back,
    LeftThumb,
    RightThumb,
    LeftShoulder,
    RightShoulder,
    Guide,
    A,
    B,
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DpadDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerName {
    Left,
    Right,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum X360ButtonEntry {
    Button { name: X360ButtonName },
    Dpad { direction: DpadDirection },
    Trigger { trigger: TriggerName },
}

#[derive(Debug, Clone, Deserialize)]
pub struct X360Mapping {
    pub name: Option<String>,
    pub output: OutputType,
    pub buttons: HashMap<String, X360ButtonEntry>,
}

// --- Unified Mapping Config ---

#[derive(Debug, Clone)]
pub enum MappingConfig {
    Keyboard(KeyboardMapping),
    X360(X360Mapping),
}

impl MappingConfig {
    pub fn name(&self) -> Option<&str> {
        match self {
            MappingConfig::Keyboard(m) => m.name.as_deref(),
            MappingConfig::X360(m) => m.name.as_deref(),
        }
    }

    pub fn output_type(&self) -> &OutputType {
        match self {
            MappingConfig::Keyboard(m) => &m.output,
            MappingConfig::X360(m) => &m.output,
        }
    }
}

// --- Output Adapter Trait ---

pub trait OutputAdapter {
    fn handle_button(&mut self, event: &ButtonEvent);
    fn shutdown(&mut self);
}
