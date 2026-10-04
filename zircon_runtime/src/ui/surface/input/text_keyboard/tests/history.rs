use super::*;
use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
};

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
