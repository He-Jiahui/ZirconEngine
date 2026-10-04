use zircon_runtime_interface::ui::component::UiValue;

pub(super) fn toast_timer_from_queue_value(value: &UiValue) -> Option<(&str, u64)> {
    match value {
        UiValue::Array(values) => values.iter().find_map(toast_timer_from_queue_value),
        UiValue::Map(values) => {
            let toast_id =
                first_string_value(values, &["id", "toast_id", "toastId", "value", "key"])?;
            let timeout_ms = first_u64_value(
                values,
                &[
                    "duration",
                    "duration_ms",
                    "auto_hide_duration_ms",
                    "autoHideDuration",
                ],
            )?;
            (timeout_ms > 0).then_some((toast_id, timeout_ms))
        }
        UiValue::String(value) | UiValue::Enum(value) => toast_timer_from_queue_string(value),
        _ => None,
    }
}

fn toast_timer_from_queue_string(value: &str) -> Option<(&str, u64)> {
    let mut parts = value.split('|');
    let toast_id = parts.next()?.trim();
    if toast_id.is_empty() {
        return None;
    }

    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        if matches!(
            key.trim(),
            "duration" | "duration_ms" | "auto_hide_duration_ms" | "autoHideDuration"
        ) {
            let timeout_ms = value.trim().parse::<u64>().ok()?;
            return (timeout_ms > 0).then_some((toast_id, timeout_ms));
        }
    }
    None
}

fn first_string_value<'a>(
    values: &'a std::collections::BTreeMap<String, UiValue>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn first_u64_value(
    values: &std::collections::BTreeMap<String, UiValue>,
    keys: &[&str],
) -> Option<u64> {
    keys.iter()
        .find_map(|key| values.get(*key).and_then(u64_value))
        .filter(|value| *value > 0)
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.as_str()),
        _ => None,
    }
}

fn u64_value(value: &UiValue) -> Option<u64> {
    match value {
        UiValue::Int(value) => Some((*value).max(0) as u64),
        UiValue::Float(value) => Some((*value).round().max(0.0) as u64),
        UiValue::String(value) | UiValue::Enum(value) => value.parse::<u64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/toast_timer_queue.rs"]
mod tests;
