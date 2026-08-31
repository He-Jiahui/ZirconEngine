use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

use crate::ui::dispatch::UiTextHistoryDirection;

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
mod tests {
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
}
