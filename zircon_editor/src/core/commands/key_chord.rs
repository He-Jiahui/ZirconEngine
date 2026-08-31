use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    pub fn is_valid(&self) -> bool {
        let key = self.key.trim();
        !key.is_empty()
            && !is_modifier_key_name(key)
            && !starts_with_ignore_ascii_case(key, "dead")
            && !starts_with_ignore_ascii_case(key, "unidentified")
    }

    pub fn from_keyboard_input(keyboard: &UiKeyboardInputEvent) -> Option<Self> {
        EditorKeyboardChordInput::from_keyboard_input(keyboard).map(|input| input.into_chord())
    }

    pub(crate) fn signature(&self) -> EditorKeyChordSignature {
        EditorKeyChordSignature::from_chord(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EditorKeyChordSignature(u64);

impl EditorKeyChordSignature {
    fn from_chord(chord: &EditorKeyChord) -> Self {
        let mut hasher = KeyChordSignatureHasher::new();
        hasher.write_bytes(chord.key.as_bytes());
        hasher.write_modifiers(chord.ctrl, chord.shift, chord.alt, chord.meta);
        Self(hasher.finish())
    }
}

/// A borrowed keyboard chord used only during event dispatch.
///
/// It mirrors `EditorKeyChord` normalization without constructing an owned key.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EditorKeyboardChordInput<'a> {
    key: KeyboardInputKey<'a>,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

impl<'a> EditorKeyboardChordInput<'a> {
    pub(crate) fn from_keyboard_input(keyboard: &'a UiKeyboardInputEvent) -> Option<Self> {
        if keyboard.state != UiKeyboardInputState::Pressed {
            return None;
        }
        let modifiers = keyboard.metadata.modifiers;
        Some(Self {
            key: keyboard_input_key(keyboard)?,
            ctrl: modifiers.control,
            shift: modifiers.shift,
            alt: modifiers.alt,
            meta: modifiers.super_key,
        })
    }

    pub(crate) fn signature(self) -> EditorKeyChordSignature {
        let mut hasher = KeyChordSignatureHasher::new();
        self.key.write_signature(&mut hasher);
        hasher.write_modifiers(self.ctrl, self.shift, self.alt, self.meta);
        EditorKeyChordSignature(hasher.finish())
    }

    pub(crate) fn matches(self, chord: &EditorKeyChord) -> bool {
        self.ctrl == chord.ctrl
            && self.shift == chord.shift
            && self.alt == chord.alt
            && self.meta == chord.meta
            && self.key.matches(chord.key())
    }

    fn into_chord(self) -> EditorKeyChord {
        let mut chord = EditorKeyChord::new(self.key.to_owned_key());
        chord.ctrl = self.ctrl;
        chord.shift = self.shift;
        chord.alt = self.alt;
        chord.meta = self.meta;
        chord
    }
}

#[derive(Clone, Copy, Debug)]
enum KeyboardInputKey<'a> {
    Fixed(&'static str),
    Raw(&'a str),
    AsciiUpper(&'a str),
    AsciiByte(u8),
    FunctionKey(u8),
}

impl KeyboardInputKey<'_> {
    fn write_signature(self, hasher: &mut KeyChordSignatureHasher) {
        match self {
            Self::Fixed(value) => hasher.write_bytes(value.as_bytes()),
            Self::Raw(value) => hasher.write_bytes(value.as_bytes()),
            Self::AsciiUpper(value) => {
                for byte in value.bytes() {
                    hasher.write_byte(byte.to_ascii_uppercase());
                }
            }
            Self::AsciiByte(byte) => hasher.write_byte(byte),
            Self::FunctionKey(number) => {
                hasher.write_byte(b'F');
                write_decimal_bytes(number, |byte| hasher.write_byte(byte));
            }
        }
    }

    fn matches(self, stored_key: &str) -> bool {
        match self {
            Self::Fixed(value) => stored_key == value,
            Self::Raw(value) => stored_key == value,
            Self::AsciiUpper(value) => {
                stored_key.len() == value.len()
                    && stored_key
                        .bytes()
                        .zip(value.bytes())
                        .all(|(stored, input)| stored == input.to_ascii_uppercase())
            }
            Self::AsciiByte(byte) => stored_key.as_bytes() == [byte],
            Self::FunctionKey(number) => stored_key
                .strip_prefix('F')
                .is_some_and(|suffix| decimal_bytes_match(suffix, number)),
        }
    }

    fn to_owned_key(self) -> String {
        match self {
            Self::Fixed(value) => value.to_string(),
            Self::Raw(value) => value.to_string(),
            Self::AsciiUpper(value) => value.to_ascii_uppercase(),
            Self::AsciiByte(byte) => char::from(byte).to_string(),
            Self::FunctionKey(number) => {
                let mut key = String::from("F");
                write_decimal_bytes(number, |byte| key.push(char::from(byte)));
                key
            }
        }
    }
}

fn keyboard_input_key(keyboard: &UiKeyboardInputEvent) -> Option<KeyboardInputKey<'_>> {
    let logical_key = keyboard.logical_key.trim();
    if logical_key.is_empty() {
        return fallback_keyboard_key_for_code(keyboard.key_code);
    }
    if logical_key == " " || logical_key.eq_ignore_ascii_case("spacebar") {
        return Some(KeyboardInputKey::Fixed("Space"));
    }
    if is_modifier_key_name(logical_key) || starts_with_ignore_ascii_case(logical_key, "dead") {
        return None;
    }
    if starts_with_ignore_ascii_case(logical_key, "unidentified") {
        return fallback_keyboard_key_for_code(keyboard.key_code);
    }
    Some(normalized_keyboard_key(logical_key))
}

fn is_modifier_key_name(key: &str) -> bool {
    [
        "control", "ctrl", "shift", "alt", "meta", "super", "capslock", "numlock",
    ]
    .iter()
    .any(|modifier| key.eq_ignore_ascii_case(modifier))
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn normalized_keyboard_key(value: &str) -> KeyboardInputKey<'_> {
    if value.eq_ignore_ascii_case("del") || value.eq_ignore_ascii_case("delete") {
        KeyboardInputKey::Fixed("Delete")
    } else if value.eq_ignore_ascii_case("esc") || value.eq_ignore_ascii_case("escape") {
        KeyboardInputKey::Fixed("Escape")
    } else if value.eq_ignore_ascii_case("space") {
        KeyboardInputKey::Fixed("Space")
    } else if value.eq_ignore_ascii_case("enter") || value.eq_ignore_ascii_case("return") {
        KeyboardInputKey::Fixed("Enter")
    } else if value.len() == 1 || is_function_key_name(value) {
        KeyboardInputKey::AsciiUpper(value)
    } else {
        KeyboardInputKey::Raw(value)
    }
}

fn is_function_key_name(value: &str) -> bool {
    value
        .as_bytes()
        .first()
        .is_some_and(|byte| matches!(byte, b'f' | b'F'))
        && value[1..]
            .chars()
            .all(|character| character.is_ascii_digit())
}

fn fallback_keyboard_key_for_code(key_code: u32) -> Option<KeyboardInputKey<'static>> {
    match key_code {
        8 => Some(KeyboardInputKey::Fixed("Backspace")),
        9 => Some(KeyboardInputKey::Fixed("Tab")),
        13 => Some(KeyboardInputKey::Fixed("Enter")),
        27 => Some(KeyboardInputKey::Fixed("Escape")),
        32 => Some(KeyboardInputKey::Fixed("Space")),
        46 => Some(KeyboardInputKey::Fixed("Delete")),
        48..=57 | 65..=90 => Some(KeyboardInputKey::AsciiByte(key_code as u8)),
        112..=123 => Some(KeyboardInputKey::FunctionKey((key_code - 111) as u8)),
        _ => None,
    }
}

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

struct KeyChordSignatureHasher(u64);

impl KeyChordSignatureHasher {
    const fn new() -> Self {
        Self(FNV_OFFSET_BASIS)
    }

    fn write_byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(FNV_PRIME);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_byte(*byte);
        }
    }

    fn write_modifiers(&mut self, ctrl: bool, shift: bool, alt: bool, meta: bool) {
        self.write_byte(0xff);
        self.write_byte(u8::from(ctrl));
        self.write_byte(u8::from(shift));
        self.write_byte(u8::from(alt));
        self.write_byte(u8::from(meta));
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

fn write_decimal_bytes(number: u8, mut write: impl FnMut(u8)) {
    if number >= 100 {
        write(b'0' + number / 100);
        write(b'0' + (number / 10) % 10);
        write(b'0' + number % 10);
    } else if number >= 10 {
        write(b'0' + number / 10);
        write(b'0' + number % 10);
    } else {
        write(b'0' + number);
    }
}

fn decimal_bytes_match(value: &str, number: u8) -> bool {
    let mut expected = [0; 3];
    let mut count = 0;
    write_decimal_bytes(number, |byte| {
        expected[count] = byte;
        count += 1;
    });
    value.as_bytes() == &expected[..count]
}

impl fmt::Display for EditorKeyChord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_modifier = false;
        for (enabled, label) in [
            (self.ctrl, "Ctrl"),
            (self.shift, "Shift"),
            (self.alt, "Alt"),
            (self.meta, "Meta"),
        ] {
            if !enabled {
                continue;
            }
            if wrote_modifier {
                formatter.write_str("+")?;
            }
            formatter.write_str(label)?;
            wrote_modifier = true;
        }
        if wrote_modifier {
            formatter.write_str("+")?;
        }
        formatter.write_str(&self.key)
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
            if part.eq_ignore_ascii_case("ctrl") || part.eq_ignore_ascii_case("control") {
                chord.ctrl = true;
            } else if part.eq_ignore_ascii_case("shift") {
                chord.shift = true;
            } else if part.eq_ignore_ascii_case("alt") || part.eq_ignore_ascii_case("option") {
                chord.alt = true;
            } else if ["meta", "cmd", "command", "super"]
                .iter()
                .any(|modifier| part.eq_ignore_ascii_case(modifier))
            {
                chord.meta = true;
            } else if key.replace(normalize_key(part)).is_some() {
                return Err(EditorKeyChordParseError::MultipleKeys(value.to_string()));
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
    if value.eq_ignore_ascii_case("del") || value.eq_ignore_ascii_case("delete") {
        "Delete".to_string()
    } else if value.eq_ignore_ascii_case("esc") || value.eq_ignore_ascii_case("escape") {
        "Escape".to_string()
    } else if value.eq_ignore_ascii_case("space") {
        "Space".to_string()
    } else if value.eq_ignore_ascii_case("enter") || value.eq_ignore_ascii_case("return") {
        "Enter".to_string()
    } else if value.len() == 1
        || (value
            .as_bytes()
            .first()
            .is_some_and(|byte| matches!(byte, b'f' | b'F'))
            && value[1..]
                .chars()
                .all(|character| character.is_ascii_digit()))
    {
        value.to_ascii_uppercase()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::EditorKeyChord;

    #[test]
    fn chord_format_and_alias_normalization_preserve_canonical_text() {
        assert_eq!(
            EditorKeyChord::from_str("command+option+del")
                .unwrap()
                .to_string(),
            "Alt+Meta+Delete"
        );
        assert_eq!(EditorKeyChord::new("escape").to_string(), "Escape");
        assert_eq!(EditorKeyChord::new("f12").to_string(), "F12");
    }

    #[test]
    fn chord_validity_requires_one_non_modifier_key() {
        assert!(EditorKeyChord::from_str("Ctrl+S").unwrap().is_valid());
        assert!(!EditorKeyChord::new("").is_valid());
        assert!(!EditorKeyChord::new("Ctrl").is_valid());
        assert!(!EditorKeyChord::new("DeadAcute").is_valid());
        assert!(!EditorKeyChord::new("Unidentified").is_valid());
    }

    #[test]
    fn hot_chord_normalization_and_display_do_not_build_temporary_lowercase_or_parts_lists() {
        let source = include_str!("key_chord.rs");
        let lowercase_temporary = ["to_ascii_lowercase()", ".as_str()"].concat();
        let parts_list = ["let mut parts = ", "Vec::new()"].concat();

        assert!(!source.contains(&lowercase_temporary));
        assert!(!source.contains(&parts_list));
    }
}
