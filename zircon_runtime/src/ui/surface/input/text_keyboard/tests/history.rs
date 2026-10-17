//! 固定 Control 与按下态，验证三种逻辑键组合对应的历史方向。
//! 测试通过父模块私有入口分类，不创建 Surface 或文档历史资源。

use super::*;
use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
};

// 逻辑键与物理键标签使用同一入参；Control 与按下态固定，Shift 独立选择。
fn keyboard(logical_key: &str, shift: bool) -> UiKeyboardInputEvent {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1));
    metadata.modifiers.control = true;
    metadata.modifiers.shift = shift;
    UiKeyboardInputEvent {
        metadata,
        state: UiKeyboardInputState::Pressed,
        key_code: 0,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: None,
    }
}

#[test]
fn primary_modifier_z_and_y_map_to_document_history() {
    assert_eq!(
        keyboard_text_history_direction(&keyboard("z", false)),
        Some(UiTextHistoryDirection::Undo)
    );
    assert_eq!(
        keyboard_text_history_direction(&keyboard("Z", true)),
        Some(UiTextHistoryDirection::Redo)
    );
    assert_eq!(
        keyboard_text_history_direction(&keyboard("y", false)),
        Some(UiTextHistoryDirection::Redo)
    );
}
