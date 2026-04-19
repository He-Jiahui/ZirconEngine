use crate::ui::{EditorUiBinding, EditorUiBindingPayload};
use zircon_runtime::core::framework::animation::AnimationTrackPath;

use super::super::error::EditorBindingDispatchError;
use super::animation_host_event::AnimationHostEvent;

pub fn dispatch_animation_binding(
    binding: &EditorUiBinding,
) -> Result<AnimationHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::PositionOfTrackAndFrame { track_path, frame } => {
            let track_path = AnimationTrackPath::parse(track_path).map_err(|error| {
                EditorBindingDispatchError::InvalidAnimationTrackPath(error.to_string())
            })?;
            Ok(AnimationHostEvent::AddFrame {
                track_path,
                frame: *frame,
            })
        }
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}
