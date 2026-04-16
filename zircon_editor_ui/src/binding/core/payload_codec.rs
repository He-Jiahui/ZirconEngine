use zircon_ui::{UiBindingCall, UiBindingValue};

use crate::binding::{
    AssetCommand, DockCommand, DraftCommand, SelectionCommand, ViewportCommand, WelcomeCommand,
};

use super::{EditorUiBindingError, EditorUiBindingPayload, InspectorFieldChange};

impl EditorUiBindingPayload {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::PositionOfTrackAndFrame { track_path, frame } => {
                UiBindingCall::new("PositionOfTrackAndFrame")
                    .with_argument(UiBindingValue::string(track_path))
                    .with_argument(UiBindingValue::unsigned(*frame))
            }
            Self::MenuAction { action_id } => {
                UiBindingCall::new("MenuAction").with_argument(UiBindingValue::string(action_id))
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
            "PositionOfTrackAndFrame" => Ok(Self::PositionOfTrackAndFrame {
                track_path: call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "PositionOfTrackAndFrame expects string track_path".to_string(),
                    ))?
                    .to_string(),
                frame: call.argument(1).and_then(UiBindingValue::as_u32).ok_or(
                    EditorUiBindingError::InvalidPayload(
                        "PositionOfTrackAndFrame expects u32 frame".to_string(),
                    ),
                )?,
            }),
            "MenuAction" => Ok(Self::MenuAction {
                action_id: call
                    .argument(0)
                    .and_then(UiBindingValue::as_str)
                    .ok_or(EditorUiBindingError::InvalidPayload(
                        "MenuAction expects string action_id".to_string(),
                    ))?
                    .to_string(),
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
