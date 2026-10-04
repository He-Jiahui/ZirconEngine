use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

use crate::ui::dispatch::UiTextHistoryDirection;

/// 识别文档历史命令，交给活动文档会话提交历史移动；这里只分类，不直接改变 surface 文本。
pub(in crate::ui::surface::input) fn keyboard_text_history_direction(
    keyboard: &UiKeyboardInputEvent,
) -> Option<UiTextHistoryDirection> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }
    let modifiers = &keyboard.metadata.modifiers;
    if (!modifiers.control && !modifiers.super_key) || modifiers.alt {
        return None;
    }
    let logical_key = keyboard.logical_key.as_str();
    let is_z = matches!(logical_key, "z" | "Z")
        || (logical_key.is_empty() && matches!(keyboard.key_code, 90 | 122));
    let is_y = matches!(logical_key, "y" | "Y")
        || (logical_key.is_empty() && matches!(keyboard.key_code, 89 | 121));
    if is_z {
        return Some(if modifiers.shift {
            UiTextHistoryDirection::Redo
        } else {
            UiTextHistoryDirection::Undo
        });
    }
    (is_y && !modifiers.shift).then_some(UiTextHistoryDirection::Redo)
}

#[cfg(test)]
#[path = "tests/history.rs"]
mod tests;
