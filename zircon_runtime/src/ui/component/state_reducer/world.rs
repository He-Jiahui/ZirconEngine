use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValidationState, UiValue,
};

pub(super) fn apply_world_transform(
    state: &mut UiComponentState,
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
) -> Result<(), UiComponentEventError> {
    if scale.iter().any(|value| *value <= 0.0) {
        state.validation = UiValidationState::error("world scale must be positive".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_scale".to_string(),
            value: format!("{scale:?}"),
        });
    }
    super::set_value(state, "world_position".to_string(), UiValue::Vec3(position));
    super::set_value(state, "world_rotation".to_string(), UiValue::Vec3(rotation));
    super::set_value(state, "world_scale".to_string(), UiValue::Vec3(scale));
    Ok(())
}

pub(super) fn apply_world_surface(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    size: [f64; 2],
    pixels_per_meter: f64,
    billboard: bool,
    depth_test: bool,
    render_order: i64,
    camera_target: String,
) -> Result<(), UiComponentEventError> {
    if size.iter().any(|value| *value <= 0.0) {
        state.validation = UiValidationState::error("world size must be positive".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_size".to_string(),
            value: format!("{size:?}"),
        });
    }
    let pixels_per_meter = descriptor
        .prop("pixels_per_meter")
        .map(|schema| super::clamp_numeric(pixels_per_meter, schema.min, schema.max))
        .unwrap_or(pixels_per_meter);
    super::set_value(state, "world_size".to_string(), UiValue::Vec2(size));
    super::set_value(
        state,
        "pixels_per_meter".to_string(),
        UiValue::Float(pixels_per_meter),
    );
    super::set_value(state, "billboard".to_string(), UiValue::Bool(billboard));
    super::set_value(state, "depth_test".to_string(), UiValue::Bool(depth_test));
    super::set_value(
        state,
        "render_order".to_string(),
        UiValue::Int(render_order),
    );
    super::set_value(
        state,
        "camera_target".to_string(),
        UiValue::String(camera_target),
    );
    Ok(())
}
