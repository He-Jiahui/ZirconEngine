//! 拖动事件把调用方提供的增量解释为步数；普通与大步拖动共用当前值、动态范围和类型归一入口，保持数值事件的约束一致。

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentState, UiValueKind,
};

// 上层先验证 DragDelta/LargeDragDelta 支持；属性必须声明为数值，动态状态设置优先于描述符默认及 schema 边界。
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
