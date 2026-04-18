use zircon_ui::{UiBindingCall, UiBindingValue};

use super::EditorUiBindingError;

pub(crate) fn required_string_argument(
    call: &UiBindingCall,
    index: usize,
    symbol: &str,
) -> Result<String, EditorUiBindingError> {
    call.argument(index)
        .and_then(UiBindingValue::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            EditorUiBindingError::InvalidPayload(format!(
                "{symbol} expects string argument at index {index}"
            ))
        })
}

pub(crate) fn required_u32_argument(
    call: &UiBindingCall,
    index: usize,
    symbol: &str,
) -> Result<u32, EditorUiBindingError> {
    call.argument(index)
        .and_then(UiBindingValue::as_u32)
        .ok_or_else(|| {
            EditorUiBindingError::InvalidPayload(format!(
                "{symbol} expects unsigned argument at index {index}"
            ))
        })
}

pub(crate) fn required_f32_argument(
    call: &UiBindingCall,
    index: usize,
    symbol: &str,
) -> Result<f32, EditorUiBindingError> {
    let value = call.argument(index).ok_or_else(|| {
        EditorUiBindingError::InvalidPayload(format!(
            "{symbol} expects numeric argument at index {index}"
        ))
    })?;
    match value {
        UiBindingValue::Float(value) => Ok(*value as f32),
        UiBindingValue::Unsigned(value) => Ok(*value as f32),
        UiBindingValue::Signed(value) => Ok(*value as f32),
        _ => Err(EditorUiBindingError::InvalidPayload(format!(
            "{symbol} expects numeric argument at index {index}"
        ))),
    }
}

pub(crate) fn required_bool_argument(
    call: &UiBindingCall,
    index: usize,
    symbol: &str,
) -> Result<bool, EditorUiBindingError> {
    match call.argument(index) {
        Some(UiBindingValue::Bool(value)) => Ok(*value),
        _ => Err(EditorUiBindingError::InvalidPayload(format!(
            "{symbol} expects bool argument at index {index}"
        ))),
    }
}
