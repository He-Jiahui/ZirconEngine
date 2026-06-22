use serde_json::Value;
use zircon_runtime_interface::ui::event_ui::UiValueType;

pub(super) fn infer_value_type(value: &Value) -> UiValueType {
    match value {
        Value::Null => UiValueType::Null,
        Value::Bool(_) => UiValueType::Bool,
        Value::Number(number) if number.is_u64() => UiValueType::Unsigned,
        Value::Number(number) if number.is_i64() => UiValueType::Signed,
        Value::Number(_) => UiValueType::Float,
        Value::String(_) => UiValueType::String,
        Value::Array(_) => UiValueType::Array,
        Value::Object(_) => UiValueType::Object,
    }
}
