use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, WelcomeCommand};

use super::super::error::EditorBindingDispatchError;
use super::welcome_host_event::WelcomeHostEvent;

pub fn dispatch_welcome_binding(
    binding: &EditorUiBinding,
) -> Result<WelcomeHostEvent, EditorBindingDispatchError> {
    let EditorUiBindingPayload::WelcomeCommand(command) = binding.payload() else {
        return Err(EditorBindingDispatchError::UnsupportedPayload);
    };

    match command {
        WelcomeCommand::SetProjectName { value } => Ok(WelcomeHostEvent::SetProjectName {
            value: value.clone(),
        }),
        WelcomeCommand::SetLocation { value } => Ok(WelcomeHostEvent::SetLocation {
            value: value.clone(),
        }),
        WelcomeCommand::CreateProject => Ok(WelcomeHostEvent::CreateProject),
        WelcomeCommand::OpenExistingProject => Ok(WelcomeHostEvent::OpenExistingProject),
        WelcomeCommand::OpenRecentProject { path } => {
            Ok(WelcomeHostEvent::OpenRecentProject { path: path.clone() })
        }
        WelcomeCommand::RemoveRecentProject { path } => {
            Ok(WelcomeHostEvent::RemoveRecentProject { path: path.clone() })
        }
    }
}
