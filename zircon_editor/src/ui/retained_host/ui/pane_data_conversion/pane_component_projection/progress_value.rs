use super::super::pane_value_conversion::normalized_value_percent;

pub(super) fn projected_value_percent(
    component_role: &str,
    value_number: f64,
    value_percent: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
) -> f32 {
    if let Some(value_percent) = value_percent {
        return normalize_percent_literal(value_percent);
    }
    match (min, max) {
        (Some(min), Some(max)) if max > min => {
            ((value_number - min) / (max - min)).clamp(0.0, 1.0) as f32
        }
        _ if is_progress_component_role(component_role) && value_number > 1.0 => {
            normalize_percent_literal(value_number)
        }
        _ => normalized_value_percent(value_number, min, max),
    }
}

fn normalize_percent_literal(value: f64) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0) as f32
    } else {
        value.clamp(0.0, 1.0) as f32
    }
}

fn is_progress_component_role(component_role: &str) -> bool {
    matches!(
        component_role,
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    )
}
