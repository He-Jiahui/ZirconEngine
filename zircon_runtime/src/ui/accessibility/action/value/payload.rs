use zircon_runtime_interface::ui::{
    accessibility::{UiA11yRole, UiAccessibilityActionRequest},
    component::UiValue,
};

pub(super) fn set_value_payload(
    request: &UiAccessibilityActionRequest,
    role: UiA11yRole,
) -> Option<UiValue> {
    match role {
        UiA11yRole::TextInput => request.value.clone().map(UiValue::String).or_else(|| {
            request
                .numeric_value
                .map(|value| UiValue::String(value.to_string()))
        }),
        UiA11yRole::Slider => request
            .numeric_value
            .filter(|value| value.is_finite())
            .map(UiValue::Float)
            .or_else(|| {
                request
                    .value
                    .as_deref()
                    .and_then(|value| value.parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                    .map(UiValue::Float)
            }),
        _ => None,
    }
}
