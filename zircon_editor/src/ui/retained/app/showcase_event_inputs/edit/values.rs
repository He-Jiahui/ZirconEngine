use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::action_matches;

pub(super) fn demo_value_edit_input(
    action_id: &str,
    value: &str,
) -> UiComponentShowcaseDemoEventInput {
    let value =
        if action_matches(action_id, "number_field") || action_matches(action_id, "range_field") {
            value
                .parse::<f64>()
                .map(UiValue::Float)
                .unwrap_or_else(|_| UiValue::String(value.to_string()))
        } else {
            UiValue::String(value.to_string())
        };
    UiComponentShowcaseDemoEventInput::Value(value)
}

pub(super) fn parse_collection_edit_value(value: &str) -> UiValue {
    if let Ok(value) = value.parse::<bool>() {
        return UiValue::Bool(value);
    }
    value
        .parse::<f64>()
        .map(UiValue::Float)
        .unwrap_or_else(|_| UiValue::String(value.to_string()))
}
