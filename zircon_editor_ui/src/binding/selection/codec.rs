use zircon_ui::{UiBindingCall, UiBindingValue};

use super::SelectionCommand;
use crate::binding::core::EditorUiBindingError;

impl SelectionCommand {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::SelectSceneNode { node_id } => {
                UiBindingCall::new("SelectionCommand.SelectSceneNode")
                    .with_argument(UiBindingValue::Unsigned(*node_id))
            }
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "SelectionCommand.SelectSceneNode" => Self::SelectSceneNode {
                node_id: call.argument(0).and_then(UiBindingValue::as_u32).ok_or(
                    EditorUiBindingError::InvalidPayload(
                        "SelectionCommand.SelectSceneNode expects unsigned node_id".to_string(),
                    ),
                )? as u64,
            },
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}
