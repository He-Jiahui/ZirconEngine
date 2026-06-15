use std::fmt;
use std::str::FromStr;

use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorKeyChord {
    key: String,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

impl EditorKeyChord {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: normalize_key(key.into()),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_meta(mut self) -> Self {
        self.meta = true;
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn ctrl(&self) -> bool {
        self.ctrl
    }

    pub fn shift(&self) -> bool {
        self.shift
    }

    pub fn alt(&self) -> bool {
        self.alt
    }

    pub fn meta(&self) -> bool {
        self.meta
    }

    pub fn from_keyboard_input(keyboard: &UiKeyboardInputEvent) -> Option<Self> {
        if keyboard.state != UiKeyboardInputState::Pressed {
            return None;
        }
        let mut chord = Self::new(key_name_for_keyboard_input(keyboard)?);
        let modifiers = keyboard.metadata.modifiers;
        chord.ctrl = modifiers.control;
        chord.shift = modifiers.shift;
        chord.alt = modifiers.alt;
        chord.meta = modifiers.super_key;
        Some(chord)
    }
}

impl fmt::Display for EditorKeyChord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.meta {
            parts.push("Meta".to_string());
        }
        parts.push(self.key.clone());
        formatter.write_str(&parts.join("+"))
    }
}

impl FromStr for EditorKeyChord {
    type Err = EditorKeyChordParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut chord = EditorKeyChord::new("");
        let mut key = None;
        for part in value.split('+') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => chord.ctrl = true,
                "shift" => chord.shift = true,
                "alt" | "option" => chord.alt = true,
                "meta" | "cmd" | "command" | "super" => chord.meta = true,
                _ => {
                    if key.replace(normalize_key(part)).is_some() {
                        return Err(EditorKeyChordParseError::MultipleKeys(value.to_string()));
                    }
                }
            }
        }

        let key = key.ok_or_else(|| EditorKeyChordParseError::MissingKey(value.to_string()))?;
        chord.key = key;
        Ok(chord)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorKeyChordParseError {
    MissingKey(String),
    MultipleKeys(String),
}

impl fmt::Display for EditorKeyChordParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(value) => write!(formatter, "key chord `{value}` has no key"),
            Self::MultipleKeys(value) => write!(formatter, "key chord `{value}` has multiple keys"),
        }
    }
}

impl std::error::Error for EditorKeyChordParseError {}

fn normalize_key(value: impl AsRef<str>) -> String {
    let value = value.as_ref().trim();
    match value.to_ascii_lowercase().as_str() {
        "del" | "delete" => "Delete".to_string(),
        "esc" | "escape" => "Escape".to_string(),
        "space" => "Space".to_string(),
        "enter" | "return" => "Enter".to_string(),
        value if value.len() == 1 => value.to_ascii_uppercase(),
        value
            if value.starts_with('f')
                && value[1..]
                    .chars()
                    .all(|character| character.is_ascii_digit()) =>
        {
            value.to_ascii_uppercase()
        }
        _ => value.to_string(),
    }
}

fn key_name_for_keyboard_input(keyboard: &UiKeyboardInputEvent) -> Option<String> {
    let logical_key = keyboard.logical_key.trim();
    let key = match logical_key.to_ascii_lowercase().as_str() {
        "" => fallback_key_name_for_code(keyboard.key_code)?,
        " " | "spacebar" => "Space".to_string(),
        "control" | "ctrl" | "shift" | "alt" | "meta" | "super" | "capslock" | "numlock" => {
            return None;
        }
        value if value.starts_with("dead") => return None,
        value if value.starts_with("unidentified") => {
            fallback_key_name_for_code(keyboard.key_code)?
        }
        _ => logical_key.to_string(),
    };
    Some(key)
}

fn fallback_key_name_for_code(key_code: u32) -> Option<String> {
    match key_code {
        8 => Some("Backspace".to_string()),
        9 => Some("Tab".to_string()),
        13 => Some("Enter".to_string()),
        27 => Some("Escape".to_string()),
        32 => Some("Space".to_string()),
        46 => Some("Delete".to_string()),
        48..=57 | 65..=90 => char::from_u32(key_code).map(|character| character.to_string()),
        112..=123 => Some(format!("F{}", key_code - 111)),
        _ => None,
    }
}
