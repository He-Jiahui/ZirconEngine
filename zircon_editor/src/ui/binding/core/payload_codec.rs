use zircon_runtime_interface::ui::{binding::UiBindingCall, binding::UiBindingValue};

use crate::core::editor_event::InspectorFieldChange;
use crate::ui::binding::{
    AnimationCommand, AssetCommand, DockCommand, DraftCommand, SelectionCommand, ViewportCommand,
    WelcomeCommand,
};

use super::{EditorUiBindingError, EditorUiBindingPayload};

impl EditorUiBindingPayload {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::AnimationCommand(command) => command.to_call(),
            Self::MenuAction { action_id } => {
                UiBindingCall::new("MenuAction").with_argument(UiBindingValue::string(action_id))
            }
            Self::EditorOperation {
                operation_id,
                arguments,
            } => {
                let mut call = UiBindingCall::new("EditorOperation")
                    .with_argument(UiBindingValue::string(operation_id));
                call.arguments.extend(arguments.iter().cloned());
                call
            }
            Self::DraftCommand(command) => command.to_call(),
            Self::InspectorFieldBatch {
                subject_path,
                changes,
            } => UiBindingCall::new("InspectorFieldBatch")
                .with_argument(UiBindingValue::string(subject_path))
                .with_argument(UiBindingValue::array(
                    changes
                        .iter()
                        .map(InspectorFieldChange::as_binding_value)
                        .collect::<Vec<_>>(),
                )),
            Self::SelectionCommand(command) => command.to_call(),
            Self::AssetCommand(command) => command.to_call(),
            Self::WelcomeCommand(command) => command.to_call(),
            Self::DockCommand(command) => command.to_call(),
            Self::ViewportCommand(command) => command.to_call(),
            Self::Custom(call) => call.clone(),
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Self, EditorUiBindingError> {
        if let Some(command) = AnimationCommand::from_call(call.clone())? {
            return Ok(Self::AnimationCommand(command));
        }
        if let Some(command) = SelectionCommand::from_call(call.clone())? {
            return Ok(Self::SelectionCommand(command));
        }
        if let Some(command) = AssetCommand::from_call(call.clone())? {
            return Ok(Self::AssetCommand(command));
        }
        if let Some(command) = WelcomeCommand::from_call(call.clone())? {
            return Ok(Self::WelcomeCommand(command));
        }
        if let Some(command) = DraftCommand::from_call(call.clone())? {
            return Ok(Self::DraftCommand(command));
        }
        if let Some(command) = DockCommand::from_call(call.clone())? {
            return Ok(Self::DockCommand(command));
        }
        if let Some(command) = ViewportCommand::from_call(call.clone())? {
            return Ok(Self::ViewportCommand(command));
        }
        match call.symbol.as_str() {
            "MenuAction" => Ok(Self::MenuAction {
                action_id: call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "MenuAction expects string action_id".to_string(),
                    ))?
                    .to_string(),
            }),
            "EditorOperation" => Ok(Self::EditorOperation {
                operation_id: call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "EditorOperation expects string operation_id".to_string(),
                    ))?
                    .to_string(),
                arguments: call.arguments.into_iter().skip(1).collect(),
            }),
            "InspectorFieldBatch" => {
                let subject_path = call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "InspectorFieldBatch expects string subject_path".to_string(),
                    ))?
                    .to_string();
                let changes = match call.argument(1) {
                    Some(UiBindingValue::Array(values)) => values
                        .iter()
                        .map(InspectorFieldChange::from_binding_value)
                        .collect::<Result<Vec<_>, _>>()?,
                    _ => {
                        return Err(EditorUiBindingError::InvalidPayload(
                            "InspectorFieldBatch expects [field_id,value] pairs".to_string(),
                        ));
                    }
                };
                Ok(Self::InspectorFieldBatch {
                    subject_path,
                    changes,
                })
            }
            _ => Ok(Self::Custom(call)),
        }
    }
}
