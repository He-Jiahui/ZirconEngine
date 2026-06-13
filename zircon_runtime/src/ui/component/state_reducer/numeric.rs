use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValueKind,
};

pub(super) fn apply_numeric_drag(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: String,
    delta: f64,
    step_property: &str,
) -> Result<(), UiComponentEventError> {
    let Some(schema) = descriptor.prop(&property) else {
        return Err(UiComponentEventError::NonNumericProperty { property });
    };
    if !matches!(schema.value_kind, UiValueKind::Float | UiValueKind::Int) {
        return Err(UiComponentEventError::NonNumericProperty { property });
    }
    let current = state
        .values
        .get(&property)
        .or(schema.default_value.as_ref())
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    let step = numeric_setting(state, descriptor, step_property, schema.step, 1.0);
    let next = super::clamp_component_numeric_value(
        state,
        descriptor,
        &property,
        schema.min,
        schema.max,
        current + delta * step,
    );
    super::apply_value(
        state,
        descriptor,
        property,
        super::numeric_value(schema.value_kind, next),
    )
}

fn numeric_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    schema_value: Option<f64>,
    default_value: f64,
) -> f64 {
    super::optional_numeric_setting(state, descriptor, property, schema_value)
        .unwrap_or(default_value)
}
