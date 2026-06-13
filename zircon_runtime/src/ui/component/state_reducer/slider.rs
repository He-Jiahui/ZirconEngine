use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiComponentState, UiValue};

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
    super::set_value(
        state,
        value_property.to_string(),
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
    super::set_value(state, percent_property.to_string(), UiValue::Float(percent));
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
