use zircon_runtime::ui::binding::UiBindingValue;

use super::{EditorUiBindingError, InspectorFieldChange};

impl InspectorFieldChange {
    pub fn new(field_id: impl Into<String>, value: UiBindingValue) -> Self {
        Self {
            field_id: field_id.into(),
            value,
        }
    }

    pub(crate) fn as_binding_value(&self) -> UiBindingValue {
        UiBindingValue::Array(vec![
            UiBindingValue::string(self.field_id.clone()),
            self.value.clone(),
        ])
    }

    pub(crate) fn from_binding_value(value: &UiBindingValue) -> Result<Self, EditorUiBindingError> {
        let UiBindingValue::Array(parts) = value else {
            return Err(EditorUiBindingError::InvalidPayload(
                "InspectorFieldBatch expects [field_id,value] pairs".to_string(),
            ));
        };
        if parts.len() != 2 {
            return Err(EditorUiBindingError::InvalidPayload(
                "InspectorFieldBatch expects pairs with 2 elements".to_string(),
            ));
        }
        let field_id = parts[0]
            .as_str()
            .ok_or(EditorUiBindingError::InvalidPayload(
                "InspectorFieldBatch field ids must be strings".to_string(),
            ))?
            .to_string();
        Ok(Self {
            field_id,
            value: parts[1].clone(),
        })
    }
}
