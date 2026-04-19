use zircon_runtime::ui::binding::UiBindingValue;

use super::super::error::EditorBindingDispatchError;

pub(crate) fn binding_value_to_string(
    value: &UiBindingValue,
    field_id: &str,
) -> Result<String, EditorBindingDispatchError> {
    match value {
        UiBindingValue::String(value) => Ok(value.clone()),
        UiBindingValue::Unsigned(value) => Ok(value.to_string()),
        UiBindingValue::Signed(value) => Ok(value.to_string()),
        UiBindingValue::Float(value) => Ok(value.to_string()),
        UiBindingValue::Bool(value) => Ok(value.to_string()),
        UiBindingValue::Null => Ok(String::new()),
        UiBindingValue::Array(_) => Err(EditorBindingDispatchError::InvalidInspectorFieldValue {
            field_id: field_id.to_string(),
        }),
    }
}

pub(super) fn parent_binding_value_to_string(
    value: &UiBindingValue,
    field_id: &str,
) -> Result<String, EditorBindingDispatchError> {
    match value {
        UiBindingValue::Null => Ok(String::new()),
        _ => binding_value_to_string(value, field_id),
    }
}
