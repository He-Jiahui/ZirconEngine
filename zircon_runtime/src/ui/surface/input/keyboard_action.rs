use zircon_runtime_interface::ui::{
    component::UiComponentKeyboardAction,
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
};

/// 将按键压下映射为通用组件语义动作，供默认交互复用；文本输入走独立路径。
pub(super) fn keyboard_component_action(
    keyboard: &UiKeyboardInputEvent,
) -> Option<UiComponentKeyboardAction> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }

    if is_activation_key(keyboard) {
        return Some(UiComponentKeyboardAction::Activate);
    }
    if is_cancel_key(keyboard) {
        return Some(UiComponentKeyboardAction::Cancel);
    }

    let logical_key = keyboard.logical_key.as_str();
    if normalized_key_matches(
        logical_key,
        &[
            "arrowright",
            "right",
            "arrowdown",
            "down",
            "gamepaddpadright",
            "gamepaddpaddown",
        ],
    ) {
        Some(UiComponentKeyboardAction::Next)
    } else if normalized_key_matches(
        logical_key,
        &[
            "arrowleft",
            "left",
            "arrowup",
            "up",
            "gamepaddpadleft",
            "gamepaddpadup",
        ],
    ) {
        Some(UiComponentKeyboardAction::Previous)
    } else if normalized_key_matches(logical_key, &["home"]) {
        Some(UiComponentKeyboardAction::First)
    } else if normalized_key_matches(logical_key, &["end"]) {
        Some(UiComponentKeyboardAction::Last)
    } else if normalized_key_matches(logical_key, &["pageup"]) {
        Some(UiComponentKeyboardAction::LargeIncrement)
    } else if normalized_key_matches(logical_key, &["pagedown"]) {
        Some(UiComponentKeyboardAction::LargeDecrement)
    } else {
        match keyboard.key_code {
            39 | 40 => Some(UiComponentKeyboardAction::Next),
            37 | 38 => Some(UiComponentKeyboardAction::Previous),
            36 => Some(UiComponentKeyboardAction::First),
            35 => Some(UiComponentKeyboardAction::Last),
            33 => Some(UiComponentKeyboardAction::LargeIncrement),
            34 => Some(UiComponentKeyboardAction::LargeDecrement),
            _ => None,
        }
    }
}

/// TreeView 的左右键改变层级展开，而上下键移动行；与通用组件动作的轴含义不同。
pub(super) fn tree_view_keyboard_component_action(
    keyboard: &UiKeyboardInputEvent,
) -> Option<UiComponentKeyboardAction> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }

    if is_begin_edit_key(keyboard) {
        return Some(UiComponentKeyboardAction::BeginEdit);
    }

    let logical_key = keyboard.logical_key.as_str();
    if normalized_key_matches(logical_key, &["arrowright", "right", "gamepaddpadright"]) {
        Some(UiComponentKeyboardAction::Increment)
    } else if normalized_key_matches(logical_key, &["arrowleft", "left", "gamepaddpadleft"]) {
        Some(UiComponentKeyboardAction::Decrement)
    } else if normalized_key_matches(logical_key, &["arrowdown", "down", "gamepaddpaddown"]) {
        Some(UiComponentKeyboardAction::Next)
    } else if normalized_key_matches(logical_key, &["arrowup", "up", "gamepaddpadup"]) {
        Some(UiComponentKeyboardAction::Previous)
    } else {
        match keyboard.key_code {
            39 => Some(UiComponentKeyboardAction::Increment),
            37 => Some(UiComponentKeyboardAction::Decrement),
            40 => Some(UiComponentKeyboardAction::Next),
            38 => Some(UiComponentKeyboardAction::Previous),
            _ => keyboard_component_action(keyboard),
        }
    }
}

/// 仅把无命令修饰键且可显示的键盘文本交给组件 typeahead/编辑入口。
/// 控制字符及全空白不触发组件文本动作，IME/文本事件另由其专属入口处理。
pub(super) fn keyboard_component_text(keyboard: &UiKeyboardInputEvent) -> Option<&str> {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return None;
    }
    if keyboard.metadata.modifiers.alt
        || keyboard.metadata.modifiers.control
        || keyboard.metadata.modifiers.super_key
    {
        return None;
    }

    let text = keyboard.text.as_deref()?;
    if !keyboard_text_is_usable(text) {
        return None;
    }
    Some(text)
}

fn keyboard_text_is_usable(text: &str) -> bool {
    let mut has_non_whitespace = false;
    for character in text.chars() {
        if character.is_control() {
            return false;
        }
        has_non_whitespace |= !character.is_whitespace();
    }
    has_non_whitespace
}

pub(super) fn keyboard_requests_default_activation(keyboard: &UiKeyboardInputEvent) -> bool {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return false;
    }

    is_activation_key(keyboard)
}

pub(super) fn keyboard_requests_popup_dismissal(keyboard: &UiKeyboardInputEvent) -> bool {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return false;
    }

    is_cancel_key(keyboard)
}

fn is_activation_key(keyboard: &UiKeyboardInputEvent) -> bool {
    normalized_key_matches(
        keyboard.logical_key.as_str(),
        &["enter", "space", "spacebar", "virtualaccept"],
    ) || keyboard.logical_key == " "
        || matches!(keyboard.key_code, 13 | 32)
}

fn is_cancel_key(keyboard: &UiKeyboardInputEvent) -> bool {
    normalized_key_matches(
        keyboard.logical_key.as_str(),
        &["escape", "esc", "virtualback"],
    ) || keyboard.key_code == 27
}

fn is_begin_edit_key(keyboard: &UiKeyboardInputEvent) -> bool {
    normalized_key_matches(keyboard.logical_key.as_str(), &["f2"]) || keyboard.key_code == 113
}

fn normalized_key_matches(key: &str, expected: &[&str]) -> bool {
    expected.iter().any(|expected| {
        key.bytes()
            .filter(u8::is_ascii_alphanumeric)
            .map(|byte| byte.to_ascii_lowercase())
            .eq(expected.bytes())
    })
}

#[cfg(test)]
#[path = "keyboard_action/tests/single_scan_text_tests.rs"]
mod single_scan_text_tests;
