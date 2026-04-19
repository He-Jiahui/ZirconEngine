use zircon_runtime::ui::{binding::UiBindingValue, binding::UiEventBinding, binding::UiEventPath};

use super::{EditorUiBinding, EditorUiBindingError, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::binding::{inspector_field_control_id, DraftCommand};

impl EditorUiBinding {
    pub fn new(
        view_id: impl Into<String>,
        control_id: impl Into<String>,
        event_kind: EditorUiEventKind,
        payload: EditorUiBindingPayload,
    ) -> Self {
        Self {
            path: UiEventPath::new(view_id, control_id, event_kind),
            payload,
        }
    }

    pub fn path(&self) -> &UiEventPath {
        &self.path
    }

    pub fn payload(&self) -> &EditorUiBindingPayload {
        &self.payload
    }

    pub fn as_ui_binding(&self) -> UiEventBinding {
        UiEventBinding::new(self.path.clone(), self.payload.to_call())
    }

    pub fn from_ui_binding(binding: UiEventBinding) -> Result<Self, EditorUiBindingError> {
        let payload = binding
            .action
            .ok_or_else(|| EditorUiBindingError::InvalidPayload("missing binding action".into()))
            .and_then(EditorUiBindingPayload::from_call)?;
        Ok(Self {
            path: binding.path,
            payload,
        })
    }

    pub fn with_arguments(
        &self,
        arguments: Vec<UiBindingValue>,
    ) -> Result<Self, EditorUiBindingError> {
        let mut binding = self.as_ui_binding();
        let action = binding
            .action
            .as_mut()
            .ok_or_else(|| EditorUiBindingError::InvalidPayload("missing binding action".into()))?;
        action.arguments = arguments;
        let mut rebound = Self::from_ui_binding(binding)?;
        if let EditorUiBindingPayload::DraftCommand(DraftCommand::SetInspectorField {
            field_id,
            ..
        }) = rebound.payload()
        {
            if let Some(control_id) = inspector_field_control_id(field_id) {
                rebound.path.control_id = control_id.to_string();
            }
        }
        Ok(rebound)
    }

    pub fn native_binding(&self) -> String {
        self.as_ui_binding().native_binding()
    }

    pub fn parse_native_binding(input: &str) -> Result<Self, EditorUiBindingError> {
        Self::from_ui_binding(UiEventBinding::parse_native_binding(input)?)
    }
}
