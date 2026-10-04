//! 区间滑块以实际数值和归一化比例两套字段服务绑定与交互；通用数值写入后在这里维护它们的一致投影。

use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiComponentState, UiValue};

// 仅 range-slider 角色采用成对同步；写入已由公共值入口归一，派生字段直接更新以避免再次触发递归同步。
pub(super) fn sync_after_value_change(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) {
    if descriptor.role != "range-slider" {
        return;
    }

    match property {
        "value" => sync_percent_from_value(state, descriptor, "value", "value_percent"),
        "range_min" => sync_percent_from_value(state, descriptor, "range_min", "range_min_percent"),
        "value_percent" => sync_value_from_percent(state, descriptor, "value_percent", "value"),
        "range_min_percent" => {
            sync_value_from_percent(state, descriptor, "range_min_percent", "range_min")
        }
        "min" | "max" => {
            sync_percent_from_value(state, descriptor, "range_min", "range_min_percent");
            sync_percent_from_value(state, descriptor, "value", "value_percent");
        }
        _ => {}
    }
}

fn sync_value_from_percent(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    percent_property: &str,
    value_property: &str,
) {
    if descriptor.prop(percent_property).is_none() || descriptor.prop(value_property).is_none() {
        return;
    }

    let Some(percent) = super::numeric_component_value(state, descriptor, percent_property) else {
        return;
    };
    let (min, max) = range_bounds(state, descriptor);
    let raw_value = if (max - min).abs() <= f64::EPSILON {
        min
    } else {
        min + (max - min) * percent.clamp(0.0, 1.0)
    };
    let Some(schema) = descriptor.prop(value_property) else {
        return;
    };
    let value = super::clamp_component_numeric_value(
        state,
        descriptor,
        value_property,
        schema.min,
        schema.max,
        raw_value,
    );
    set_slider_value(
        state,
        value_property,
        super::numeric_value(schema.value_kind, value),
    );
    sync_percent_from_value(state, descriptor, value_property, percent_property);
}

fn sync_percent_from_value(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    value_property: &str,
    percent_property: &str,
) {
    if descriptor.prop(percent_property).is_none() || descriptor.prop(value_property).is_none() {
        return;
    }

    let Some(value) = super::numeric_component_value(state, descriptor, value_property) else {
        return;
    };
    let (min, max) = range_bounds(state, descriptor);
    let percent = if (max - min).abs() <= f64::EPSILON {
        0.0
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    };
    set_slider_value(state, percent_property, UiValue::Float(percent));
}

// 派生比例或端点属于本次计算结果，继承的引用来源须清除；复用已有键保持高频拖动的分配契约。
fn set_slider_value(state: &mut UiComponentState, property: &str, value: UiValue) {
    state.reference_sources.remove(property);
    if let Some(current) = state.values.get_mut(property) {
        *current = value;
    } else {
        state.values.insert(property.to_owned(), value);
    }
}

fn range_bounds(state: &UiComponentState, descriptor: &UiComponentDescriptor) -> (f64, f64) {
    let min = super::optional_numeric_setting(state, descriptor, "min", None).unwrap_or(0.0);
    let max = super::optional_numeric_setting(state, descriptor, "max", None).unwrap_or(1.0);
    if min <= max {
        (min, max)
    } else {
        (max, min)
    }
}

#[cfg(test)]
#[path = "tests/slider.rs"]
mod tests;
