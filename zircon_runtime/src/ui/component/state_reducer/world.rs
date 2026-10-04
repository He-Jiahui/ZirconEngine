//! 世界表面事件把变换和表面参数保存为节点状态，供宿主世界 UI 投影消费；事件支持准入由公共归约入口负责。

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValidationState, UiValue,
};

pub(super) fn apply_world_transform(
    state: &mut UiComponentState,
    position: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
) -> Result<(), UiComponentEventError> {
    // Reject any non-finite value in any vector before writing state. This covers
    // NaN and Inf in all three inputs, not just a positive-only check on scale.
    let finite = |v: &[f64; 3]| v.iter().all(|x| x.is_finite());
    if !finite(&position) || !finite(&rotation) || !finite(&scale) {
        state.validation =
            UiValidationState::error("world transform values must be finite".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_transform".to_string(),
            value: format!("position={position:?} rotation={rotation:?} scale={scale:?}"),
        });
    }
    if scale.iter().any(|value| *value <= 0.0) {
        state.validation = UiValidationState::error("world scale must be positive".to_string());
        return Err(UiComponentEventError::InvalidComplexValue {
            property: "world_scale".to_string(),
            value: format!("{scale:?}"),
        });
    }
    set_world_value(state, "world_position", UiValue::Vec3(position));
    set_world_value(state, "world_rotation", UiValue::Vec3(rotation));
    set_world_value(state, "world_scale", UiValue::Vec3(scale));
    Ok(())
}

// 尺寸单位为世界表面参数，pixels_per_meter 使用描述符边界；宿主消费保存后的相机和渲染配置生成世界 UI。
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
    set_world_value(state, "world_size", UiValue::Vec2(size));
    set_world_value(state, "pixels_per_meter", UiValue::Float(pixels_per_meter));
    set_world_value(state, "billboard", UiValue::Bool(billboard));
    set_world_value(state, "depth_test", UiValue::Bool(depth_test));
    set_world_value(state, "render_order", UiValue::Int(render_order));
    set_world_value(state, "camera_target", UiValue::String(camera_target));
    Ok(())
}

fn set_world_value(state: &mut UiComponentState, property: &'static str, value: UiValue) {
    super::clear_reference_source(state, property);
    if let Some(existing) = state.values.get_mut(property) {
        *existing = value;
    } else {
        state.values.insert(property.to_owned(), value);
    }
}

#[cfg(test)]
#[path = "tests/world.rs"]
mod tests;
