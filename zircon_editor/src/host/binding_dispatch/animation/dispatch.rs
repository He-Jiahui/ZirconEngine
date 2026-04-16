use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload};

use super::super::error::EditorBindingDispatchError;
use super::animation_host_event::AnimationHostEvent;

pub fn dispatch_animation_binding(
    binding: &EditorUiBinding,
) -> Result<AnimationHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::PositionOfTrackAndFrame { track_path, frame } => {
            Ok(AnimationHostEvent::AddFrame {
                track_path: track_path.clone(),
                frame: *frame,
            })
        }
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}
