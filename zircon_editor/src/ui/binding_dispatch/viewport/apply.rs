use crate::scene::viewport::ViewportFeedback;
use crate::ui::binding::{EditorUiBinding, ViewportCommand};

use super::super::error::EditorBindingDispatchError;
use super::dispatch::dispatch_viewport_binding;
use crate::EditorState;

pub fn apply_viewport_binding(
    state: &mut EditorState,
    binding: &EditorUiBinding,
) -> Result<ViewportFeedback, EditorBindingDispatchError> {
    let command = dispatch_viewport_binding(binding)?;
    apply_viewport_command_to_state(state, &command)
}

pub(crate) fn apply_viewport_command_to_state(
    state: &mut EditorState,
    command: &ViewportCommand,
) -> Result<ViewportFeedback, EditorBindingDispatchError> {
    Ok(state.apply_viewport_command(command))
}
