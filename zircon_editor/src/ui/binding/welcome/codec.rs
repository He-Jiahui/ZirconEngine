use zircon_runtime_interface::ui::{binding::UiBindingCall, binding::UiBindingValue};

use super::WelcomeCommand;
use crate::ui::binding::core::{required_string_argument, EditorUiBindingError};

impl WelcomeCommand {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::SetProjectName { value } => UiBindingCall::new("WelcomeCommand.SetProjectName")
                .with_argument(UiBindingValue::string(value)),
            Self::SetLocation { value } => UiBindingCall::new("WelcomeCommand.SetLocation")
                .with_argument(UiBindingValue::string(value)),
            Self::CreateProject => UiBindingCall::new("WelcomeCommand.CreateProject"),
            Self::OpenExistingProject => UiBindingCall::new("WelcomeCommand.OpenExistingProject"),
            Self::OpenRecentProject { path } => {
                UiBindingCall::new("WelcomeCommand.OpenRecentProject")
                    .with_argument(UiBindingValue::string(path))
            }
            Self::RemoveRecentProject { path } => {
                UiBindingCall::new("WelcomeCommand.RemoveRecentProject")
                    .with_argument(UiBindingValue::string(path))
            }
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "WelcomeCommand.SetProjectName" => Self::SetProjectName {
                value: required_string_argument(&call, 0, "WelcomeCommand.SetProjectName")?,
            },
            "WelcomeCommand.SetLocation" => Self::SetLocation {
                value: required_string_argument(&call, 0, "WelcomeCommand.SetLocation")?,
            },
            "WelcomeCommand.CreateProject" => Self::CreateProject,
            "WelcomeCommand.OpenExistingProject" => Self::OpenExistingProject,
            "WelcomeCommand.OpenRecentProject" => Self::OpenRecentProject {
                path: required_string_argument(&call, 0, "WelcomeCommand.OpenRecentProject")?,
            },
            "WelcomeCommand.RemoveRecentProject" => Self::RemoveRecentProject {
                path: required_string_argument(&call, 0, "WelcomeCommand.RemoveRecentProject")?,
            },
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}
