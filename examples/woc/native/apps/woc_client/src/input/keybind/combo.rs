pub const LEGACY_KEYBIND_STORAGE_KEY: &str = "woc_keybinds";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyBindingKind {
    Held,
    Edge,
}

pub fn make_key_combo(code: &str, modifiers: KeyModifiers) -> String {
    let mut parts = Vec::with_capacity(5);
    if modifiers.ctrl {
        parts.push("Ctrl");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    if modifiers.meta {
        parts.push("Meta");
    }
    parts.push(code);
    parts.join("+")
}

pub fn combo_code(combo: &str) -> &str {
    combo.rsplit_once('+').map_or(combo, |(_, code)| code)
}

pub fn combo_modifiers(combo: &str) -> KeyModifiers {
    let Some((prefix, _)) = combo.rsplit_once('+') else {
        return KeyModifiers::default();
    };
    let has = |modifier| prefix.split('+').any(|part| part == modifier);
    KeyModifiers {
        ctrl: has("Ctrl"),
        alt: has("Alt"),
        shift: has("Shift"),
        meta: has("Meta"),
    }
}

pub fn is_modifier_code(code: &str) -> bool {
    matches!(
        code,
        "ShiftLeft"
            | "ShiftRight"
            | "ControlLeft"
            | "ControlRight"
            | "AltLeft"
            | "AltRight"
            | "MetaLeft"
            | "MetaRight"
    )
}

pub fn is_reserved_combo(combo: &str) -> bool {
    combo_code(combo) == "Escape"
}

pub fn normalize_key_combo(kind: KeyBindingKind, combo: &str) -> String {
    match kind {
        KeyBindingKind::Held => combo_code(combo).to_string(),
        KeyBindingKind::Edge => combo.to_string(),
    }
}

pub fn keybind_storage_key(scope: &str) -> String {
    if scope.is_empty() {
        LEGACY_KEYBIND_STORAGE_KEY.to_string()
    } else {
        format!("{LEGACY_KEYBIND_STORAGE_KEY}:{scope}")
    }
}

pub fn key_label(combo: Option<&str>) -> String {
    let Some(combo) = combo else {
        return String::new();
    };
    let code = combo_code(combo);
    let modifier_prefix = &combo[..combo.len() - code.len()];
    format!("{modifier_prefix}{}", code_label(code))
}

pub fn key_cap_label(label: &str) -> String {
    label
        .to_lowercase()
        .replace("shift+", "s-")
        .replace("ctrl+", "c-")
        .replace("alt+", "a-")
        .replace("meta+", "m-")
}

fn code_label(code: &str) -> String {
    if code.len() == 6 && code.starts_with("Digit") && code.as_bytes()[5].is_ascii_digit() {
        return code[5..].to_string();
    }
    if code.len() == 4 && code.starts_with("Key") && code.as_bytes()[3].is_ascii_uppercase() {
        return code[3..].to_string();
    }
    if code.strip_prefix('F').is_some_and(|suffix| {
        (1..=2).contains(&suffix.len()) && suffix.bytes().all(|b| b.is_ascii_digit())
    }) {
        return code.to_string();
    }
    if code.len() == 7 && code.starts_with("Numpad") && code.as_bytes()[6].is_ascii_digit() {
        return format!("Num{}", &code[6..]);
    }
    match code {
        "Minus" => "-",
        "Equal" => "=",
        "Backquote" => "`",
        "BracketLeft" => "[",
        "BracketRight" => "]",
        "Backslash" => "\\",
        "Semicolon" => ";",
        "Quote" => "'",
        "Comma" => ",",
        "Period" => ".",
        "Slash" => "/",
        "Space" => "Space",
        "Tab" => "Tab",
        "Enter" => "Enter",
        "Escape" => "Esc",
        "ArrowUp" => "↑",
        "ArrowDown" => "↓",
        "ArrowLeft" => "←",
        "ArrowRight" => "→",
        "ShiftLeft" => "LShift",
        "ShiftRight" => "RShift",
        "ControlLeft" => "LCtrl",
        "ControlRight" => "RCtrl",
        "AltLeft" => "LAlt",
        "AltRight" => "RAlt",
        "CapsLock" => "Caps",
        "NumpadAdd" => "Num+",
        "NumpadSubtract" => "Num-",
        "NumpadMultiply" => "Num*",
        "NumpadDivide" => "Num/",
        "NumpadDecimal" => "Num.",
        "NumpadEnter" => "NumEnter",
        _ => code,
    }
    .to_string()
}
