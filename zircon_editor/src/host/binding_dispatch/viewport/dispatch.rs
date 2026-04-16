use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload, ViewportCommand};

use super::super::error::EditorBindingDispatchError;

pub fn dispatch_viewport_binding(
    binding: &EditorUiBinding,
) -> Result<ViewportCommand, EditorBindingDispatchError> {
    let EditorUiBindingPayload::ViewportCommand(command) = binding.payload() else {
        return Err(EditorBindingDispatchError::UnsupportedPayload);
    };

    Ok(command.clone())
}
