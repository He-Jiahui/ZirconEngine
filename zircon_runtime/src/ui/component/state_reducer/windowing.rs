use zircon_runtime_interface::ui::component::{UiComponentEventError, UiComponentState, UiValue};

pub(super) fn apply_visible_range(
    state: &mut UiComponentState,
    start: i64,
    count: i64,
) -> Result<(), UiComponentEventError> {
    let total_count = int_value_any(
        state,
        &[
            "total_count",
            "row_count",
            "rowCount",
            "item_count",
            "itemCount",
            "rows",
            "items",
        ],
        0,
    )
    .max(0);
    let virtualization_disabled = bool_value_any(
        state,
        &["disable_virtualization", "disableVirtualization"],
        false,
    );
    let viewport_start = if virtualization_disabled {
        0
    } else {
        start.clamp(0, total_count)
    };
    let available_count = total_count.saturating_sub(viewport_start);
    let viewport_count = if virtualization_disabled {
        total_count
    } else {
        count.max(0).min(available_count)
    };
    let visible_end = viewport_start
        .saturating_add(viewport_count)
        .min(total_count);
    let overscan = if virtualization_disabled {
        0
    } else {
        int_value_any(state, &["overscan", "overscan_count", "overscanCount"], 0).max(0)
    };
    let requested_start = viewport_start.saturating_sub(overscan);
    let requested_end = visible_end.saturating_add(overscan).min(total_count);
    let requested_count = requested_end.saturating_sub(requested_start);
    let item_extent = float_value_any(
        state,
        &["item_extent", "itemSize", "row_height", "rowHeight"],
        0.0,
    )
    .max(0.0);

    for property in [
        "total_count",
        "row_count",
        "rowCount",
        "item_count",
        "itemCount",
    ] {
        super::set_value(state, property.to_string(), UiValue::Int(total_count));
    }
    super::set_value(
        state,
        "viewport_start".to_string(),
        UiValue::Int(viewport_start),
    );
    super::set_value(
        state,
        "viewport_count".to_string(),
        UiValue::Int(viewport_count),
    );
    super::set_value(state, "visible_end".to_string(), UiValue::Int(visible_end));
    super::set_value(state, "visibleEnd".to_string(), UiValue::Int(visible_end));
    super::set_value(
        state,
        "requested_start".to_string(),
        UiValue::Int(requested_start),
    );
    super::set_value(
        state,
        "requestedStart".to_string(),
        UiValue::Int(requested_start),
    );
    super::set_value(
        state,
        "requested_count".to_string(),
        UiValue::Int(requested_count),
    );
    super::set_value(
        state,
        "requestedCount".to_string(),
        UiValue::Int(requested_count),
    );
    super::set_value(state, "overscan".to_string(), UiValue::Int(overscan));
    super::set_value(state, "overscanCount".to_string(), UiValue::Int(overscan));
    super::set_value(
        state,
        "scroll_offset".to_string(),
        UiValue::Float(viewport_start as f64 * item_extent),
    );
    super::set_value(
        state,
        "scrollTop".to_string(),
        UiValue::Float(viewport_start as f64 * item_extent),
    );
    Ok(())
}

pub(super) fn apply_page_window(
    state: &mut UiComponentState,
    page_index: i64,
    page_size: i64,
) -> Result<(), UiComponentEventError> {
    let total_count = int_value(state, "total_count", 0).max(0);
    let page_size = page_size.max(1);
    let page_count = if total_count == 0 {
        0
    } else {
        ((total_count - 1) / page_size) + 1
    };
    let max_page_index = page_count.saturating_sub(1);
    let page_index = if page_count == 0 {
        0
    } else {
        page_index.clamp(0, max_page_index)
    };
    let page_start = page_index.saturating_mul(page_size).min(total_count);
    let page_end = page_start.saturating_add(page_size).min(total_count);

    super::set_value(state, "page_size".to_string(), UiValue::Int(page_size));
    super::set_value(state, "page_count".to_string(), UiValue::Int(page_count));
    super::set_value(state, "page_index".to_string(), UiValue::Int(page_index));
    super::set_value(state, "page_start".to_string(), UiValue::Int(page_start));
    super::set_value(state, "page_end".to_string(), UiValue::Int(page_end));
    super::set_value(state, "empty".to_string(), UiValue::Bool(total_count == 0));
    Ok(())
}

fn int_value(state: &UiComponentState, property: &str, default: i64) -> i64 {
    match state.values.get(property) {
        Some(UiValue::Int(value)) => *value,
        Some(UiValue::Array(value)) => value.len() as i64,
        Some(value) => value
            .as_f64()
            .map(|value| value.round() as i64)
            .unwrap_or(default),
        None => default,
    }
}

fn int_value_any(state: &UiComponentState, properties: &[&str], default: i64) -> i64 {
    properties
        .iter()
        .find_map(|property| match state.values.get(*property) {
            Some(UiValue::Int(value)) => Some(*value),
            Some(UiValue::Array(value)) => Some(value.len() as i64),
            Some(value) => value.as_f64().map(|value| value.round() as i64),
            None => None,
        })
        .unwrap_or(default)
}

fn float_value_any(state: &UiComponentState, properties: &[&str], default: f64) -> f64 {
    properties
        .iter()
        .find_map(|property| state.values.get(*property).and_then(UiValue::as_f64))
        .unwrap_or(default)
}

fn bool_value_any(state: &UiComponentState, properties: &[&str], default: bool) -> bool {
    properties
        .iter()
        .find_map(|property| match state.values.get(*property) {
            Some(UiValue::Bool(value)) => Some(*value),
            _ => None,
        })
        .unwrap_or(default)
}
