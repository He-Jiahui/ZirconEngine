use zircon_runtime_interface::ui::{binding::UiBindingCall, binding::UiBindingValue};

use super::DraftCommand;
use crate::ui::binding::core::{required_string_argument, EditorUiBindingError};

impl DraftCommand {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::SetInspectorField {
                subject_path,
                field_id,
                value,
            } => UiBindingCall::new("DraftCommand.SetInspectorField")
                .with_argument(UiBindingValue::string(subject_path))
                .with_argument(UiBindingValue::string(field_id))
                .with_argument(value.clone()),
            Self::SetMeshImportPath { value } => {
                UiBindingCall::new("DraftCommand.SetMeshImportPath")
                    .with_argument(UiBindingValue::string(value))
            }
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "DraftCommand.SetInspectorField" => Self::SetInspectorField {
                subject_path: required_string_argument(&call, 0, "DraftCommand.SetInspectorField")?,
                field_id: required_string_argument(&call, 1, "DraftCommand.SetInspectorField")?,
                value: call.argument(2).cloned().ok_or_else(|| {
                    EditorUiBindingError::InvalidPayload(
                        "DraftCommand.SetInspectorField expects value argument at index 2"
                            .to_string(),
                    )
                })?,
            },
            "DraftCommand.SetMeshImportPath" => Self::SetMeshImportPath {
                value: required_string_argument(&call, 0, "DraftCommand.SetMeshImportPath")?,
            },
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}
