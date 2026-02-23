use std::collections::HashSet;
use std::time::{Duration, Instant};

use enigo::{Enigo, Key, KeyboardControllable};

use crate::types::{ButtonEvent, KeyboardMapping, OutputAdapter};

/// Resolve a key name from JSON mapping to enigo::Key.
/// Supports F1-F24, letter keys, and special keys.
fn resolve_key(name: &str) -> Option<Key> {
    let upper = name.to_uppercase();
    match upper.as_str() {
        // Function keys
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        // F13-F24: use raw keycodes (platform-specific)
        "F13" => Some(Key::Raw(0x7C)),
        "F14" => Some(Key::Raw(0x7D)),
        "F15" => Some(Key::Raw(0x7E)),
        "F16" => Some(Key::Raw(0x7F)),
        "F17" => Some(Key::Raw(0x80)),
        "F18" => Some(Key::Raw(0x81)),
        "F19" => Some(Key::Raw(0x82)),
        "F20" => Some(Key::Raw(0x83)),
        "F21" => Some(Key::Raw(0x84)),
        "F22" => Some(Key::Raw(0x85)),
        "F23" => Some(Key::Raw(0x86)),
        "F24" => Some(Key::Raw(0x87)),
        // Modifier keys
        "SHIFT" | "LEFTSHIFT" => Some(Key::Shift),
        "RIGHTSHIFT" => Some(Key::Raw(0xA1)), // VK_RSHIFT
        "CONTROL" | "LEFTCONTROL" | "CTRL" => Some(Key::Control),
        "RIGHTCONTROL" | "RIGHTCTRL" => Some(Key::Raw(0xA3)), // VK_RCONTROL
        "ALT" | "LEFTALT" => Some(Key::Alt),
        "RIGHTALT" => Some(Key::Raw(0xA5)), // VK_RMENU
        // Navigation
        "ESCAPE" | "ESC" => Some(Key::Escape),
        "RETURN" | "ENTER" => Some(Key::Return),
        "TAB" => Some(Key::Tab),
        "SPACE" => Some(Key::Space),
        "BACKSPACE" => Some(Key::Backspace),
        "DELETE" => Some(Key::Delete),
        "HOME" => Some(Key::Home),
        "END" => Some(Key::End),
        "PAGEUP" => Some(Key::PageUp),
        "PAGEDOWN" => Some(Key::PageDown),
        "UP" | "UPARROW" => Some(Key::UpArrow),
        "DOWN" | "DOWNARROW" => Some(Key::DownArrow),
        "LEFT" | "LEFTARROW" => Some(Key::LeftArrow),
        "RIGHT" | "RIGHTARROW" => Some(Key::RightArrow),
        "CAPSLOCK" => Some(Key::CapsLock),
        // Single character keys (letters and digits)
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap().to_ascii_lowercase();
            Some(Key::Layout(ch))
        }
        _ => None,
    }
}

/// Scheduled tap releases for tap keys (non-blocking design from existing Rust version).
pub struct TapReleases {
    deadlines: Vec<(Key, String, Instant)>,
}

impl TapReleases {
    pub fn new() -> Self {
        Self {
            deadlines: Vec::new(),
        }
    }

    pub fn schedule(&mut self, key: Key, key_name: String, now: Instant, duration: Duration) {
        // Remove any existing deadline for this key name
        self.deadlines.retain(|(_, name, _)| name != &key_name);
        self.deadlines.push((key, key_name, now + duration));
    }

    pub fn clear(&mut self, key_name: &str) {
        self.deadlines.retain(|(_, name, _)| name != key_name);
    }

    pub fn clear_all(&mut self) {
        self.deadlines.clear();
    }

    pub fn take_due_keys(&mut self, now: Instant) -> Vec<Key> {
        let mut due = Vec::new();
        self.deadlines.retain(|(key, _, deadline)| {
            if *deadline <= now {
                due.push(*key);
                false
            } else {
                true
            }
        });
        due
    }
}

pub struct KeyboardOutput {
    enigo: Enigo,
    mapping: KeyboardMapping,
    tap_keys: HashSet<String>,
    release_on_ignore: HashSet<String>,
    ignore_key: Option<String>,
    tap_duration: Duration,
    offset_ms: u64,
    debug: bool,
    ignore: bool,
    pub tap_releases: TapReleases,
}

impl KeyboardOutput {
    pub fn new(mapping: KeyboardMapping, offset_ms: u64, debug: bool) -> Self {
        let special = mapping.special.clone().unwrap_or_default();
        let tap_keys: HashSet<String> = special
            .tap_keys
            .unwrap_or_default()
            .into_iter()
            .collect();
        let release_on_ignore: HashSet<String> = special
            .release_on_ignore
            .unwrap_or_default()
            .into_iter()
            .collect();
        let ignore_key = special.ignore_key;
        let tap_duration = Duration::from_millis(special.tap_duration_ms.unwrap_or(13));

        Self {
            enigo: Enigo::new(),
            mapping,
            tap_keys,
            release_on_ignore,
            ignore_key,
            tap_duration,
            offset_ms,
            debug,
            ignore: false,
            tap_releases: TapReleases::new(),
        }
    }

    /// Process due tap releases. Call this every tick.
    pub fn process_tap_releases(&mut self) {
        let now = Instant::now();
        for key in self.tap_releases.take_due_keys(now) {
            self.enigo.key_up(key);
        }
    }

    fn release_key_by_name(&mut self, key_name: &str) {
        if let Some(key) = resolve_key(key_name) {
            self.enigo.key_up(key);
        }
    }
}

impl OutputAdapter for KeyboardOutput {
    fn handle_button(&mut self, event: &ButtonEvent) {
        let entry = match self.mapping.buttons.get(&event.id.to_string()) {
            Some(e) => e.clone(),
            None => return,
        };

        let key_name = &entry.key;
        let key = match resolve_key(key_name) {
            Some(k) => k,
            None => {
                if self.debug {
                    eprintln!(
                        "[keyboard] Unknown key \"{}\" for button {}",
                        key_name, event.id
                    );
                }
                return;
            }
        };

        if self.debug {
            let action = if event.pressed { "press" } else { "release" };
            println!("[keyboard] {} {} (id {})", action, key_name, event.id);
        }

        if event.pressed {
            // Release keys on ignore
            if self.ignore && !self.release_on_ignore.is_empty() {
                let keys_to_release: Vec<String> =
                    self.release_on_ignore.iter().cloned().collect();
                for release_key in &keys_to_release {
                    self.release_key_by_name(release_key);
                }
            }

            // Set ignore flag
            if let Some(ref ik) = self.ignore_key {
                if key_name == ik {
                    self.ignore = true;
                }
            }

            let is_tap = self.tap_keys.contains(key_name.as_str());

            if is_tap && !self.ignore {
                // Tap key without ignore: press only (no scheduled release)
                // Offset is not applied to tap keys when not ignoring (matches TS behavior)
                self.enigo.key_down(key);
                return;
            }

            if !is_tap {
                // Regular key: press with optional offset
                // Note: offset via thread::sleep would block. For offset > 0,
                // we use a simple sleep. This matches the deadline-based approach.
                if self.offset_ms > 0 {
                    std::thread::sleep(Duration::from_millis(self.offset_ms));
                }
                self.enigo.key_down(key);
                return;
            }

            // Tap key with ignore: press and schedule release
            if is_tap && self.ignore {
                if self.offset_ms > 0 {
                    std::thread::sleep(Duration::from_millis(self.offset_ms));
                }
                self.enigo.key_down(key);
                self.tap_releases.schedule(
                    key,
                    key_name.clone(),
                    Instant::now(),
                    self.tap_duration,
                );
            }
        } else {
            // Release event
            if let Some(ref ik) = self.ignore_key {
                if key_name == ik {
                    self.ignore = false;
                }
            }

            self.tap_releases.clear(key_name);

            if self.offset_ms > 0 {
                std::thread::sleep(Duration::from_millis(self.offset_ms));
            }
            self.enigo.key_up(key);
        }
    }

    fn shutdown(&mut self) {
        // No special cleanup needed for keyboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn tap_releases_after_deadline() {
        let mut releases = TapReleases::new();
        let start = Instant::now();

        releases.schedule(
            Key::Layout('f'),
            "F15".to_string(),
            start,
            Duration::from_millis(100),
        );
        assert!(releases
            .take_due_keys(start + Duration::from_millis(99))
            .is_empty());

        let due = releases.take_due_keys(start + Duration::from_millis(100));
        assert_eq!(due, vec![Key::Layout('f')]);
    }

    #[test]
    fn clear_prevents_late_release() {
        let mut releases = TapReleases::new();
        let start = Instant::now();

        releases.schedule(
            Key::Layout('r'),
            "F13".to_string(),
            start,
            Duration::from_millis(100),
        );
        releases.clear("F13");
        assert!(releases
            .take_due_keys(start + Duration::from_millis(100))
            .is_empty());
    }

    #[test]
    fn clear_all_removes_everything() {
        let mut releases = TapReleases::new();
        let start = Instant::now();

        releases.schedule(
            Key::Layout('f'),
            "F15".to_string(),
            start,
            Duration::from_millis(100),
        );
        releases.schedule(
            Key::Layout('r'),
            "F13".to_string(),
            start,
            Duration::from_millis(100),
        );
        releases.clear_all();
        assert!(releases
            .take_due_keys(start + Duration::from_millis(200))
            .is_empty());
    }
}
