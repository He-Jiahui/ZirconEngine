use std::collections::BTreeMap;

use toml::Value;

use super::super::pane_value_conversion::{value_as_bool, value_as_f64, value_as_string};

const MUI_TRANSITION_ENTERING_SCREEN_MS: i32 = 225;
const MUI_TRANSITION_LEAVING_SCREEN_MS: i32 = 195;
const MUI_TRANSITION_STANDARD_MS: i32 = 300;
const MUI_EASING_EASE_IN_OUT: &str = "cubic-bezier(0.4, 0, 0.2, 1)";
const MUI_EASING_EASE_OUT: &str = "cubic-bezier(0.0, 0, 0.2, 1)";
const MUI_EASING_SHARP: &str = "cubic-bezier(0.4, 0, 0.6, 1)";

pub(super) struct ProjectedTransitionMetadata {
    pub kind: String,
    pub active: bool,
    pub entered: bool,
    pub progress: f32,
    pub duration_ms: i32,
    pub easing: String,
    pub direction: String,
}

pub(super) fn projected_transition_metadata(
    attributes: &BTreeMap<String, Value>,
    component_role: &str,
    popup_open: bool,
) -> ProjectedTransitionMetadata {
    let kind = attributes
        .get("transition_kind")
        .and_then(value_as_string)
        .or_else(|| attributes.get("transition").and_then(value_as_string))
        .or_else(|| transition_kind_from_role(component_role).map(str::to_string))
        .unwrap_or_default();
    let transition_in = attributes
        .get("transition_in")
        .or_else(|| attributes.get("in"))
        .and_then(value_as_bool)
        .unwrap_or_else(|| {
            if kind.is_empty() {
                true
            } else {
                popup_open
                    || attributes
                        .get("open")
                        .and_then(value_as_bool)
                        .unwrap_or(true)
            }
        });
    let status = attributes
        .get("transition_status")
        .and_then(value_as_string)
        .unwrap_or_else(|| {
            if transition_in {
                "entered".to_string()
            } else {
                "exited".to_string()
            }
        });
    let progress = attributes
        .get("transition_progress")
        .or_else(|| attributes.get("animation_progress"))
        .and_then(value_as_f64)
        .map(|value| value.clamp(0.0, 1.0) as f32)
        .unwrap_or_else(|| default_progress(status.as_str(), transition_in));
    let entered = attributes
        .get("transition_entered")
        .or_else(|| attributes.get("entered"))
        .and_then(value_as_bool)
        .unwrap_or_else(|| transition_in && status == "entered" && progress >= 1.0);
    let duration_ms = attributes
        .get("transition_duration_ms")
        .or_else(|| attributes.get("timeout_ms"))
        .or_else(|| attributes.get("duration_ms"))
        .and_then(value_as_i32)
        .unwrap_or_else(|| default_duration_ms(kind.as_str(), transition_in));
    let easing = attributes
        .get("transition_easing")
        .or_else(|| attributes.get("easing"))
        .and_then(value_as_string)
        .unwrap_or_else(|| default_easing(kind.as_str(), transition_in).to_string());
    let direction = attributes
        .get("transition_direction")
        .or_else(|| attributes.get("direction"))
        .and_then(value_as_string)
        .unwrap_or_else(|| {
            if kind == "slide" {
                "down".to_string()
            } else {
                String::new()
            }
        });

    ProjectedTransitionMetadata {
        kind,
        active: transition_in,
        entered,
        progress,
        duration_ms,
        easing,
        direction,
    }
}

fn transition_kind_from_role(component_role: &str) -> Option<&'static str> {
    match component_role {
        "collapse" => Some("collapse"),
        "fade" => Some("fade"),
        "grow" => Some("grow"),
        "slide" => Some("slide"),
        "zoom" => Some("zoom"),
        _ => None,
    }
}

fn default_progress(status: &str, transition_in: bool) -> f32 {
    match status {
        "entering" | "exiting" => 0.5,
        "entered" => 1.0,
        "exited" => 0.0,
        _ if transition_in => 1.0,
        _ => 0.0,
    }
}

fn default_duration_ms(kind: &str, transition_in: bool) -> i32 {
    match kind {
        "collapse" => MUI_TRANSITION_STANDARD_MS,
        "fade" | "grow" | "slide" | "zoom" if transition_in => MUI_TRANSITION_ENTERING_SCREEN_MS,
        "fade" | "grow" | "slide" | "zoom" => MUI_TRANSITION_LEAVING_SCREEN_MS,
        _ => 0,
    }
}

fn default_easing(kind: &str, transition_in: bool) -> &'static str {
    match (kind, transition_in) {
        ("slide", true) => MUI_EASING_EASE_OUT,
        ("slide", false) => MUI_EASING_SHARP,
        _ => MUI_EASING_EASE_IN_OUT,
    }
}

fn value_as_i32(value: &Value) -> Option<i32> {
    match value {
        Value::Integer(value) => i32::try_from(*value).ok(),
        Value::Float(value) => value.is_finite().then_some(value.round() as i32),
        Value::String(value) => value.trim().parse::<i32>().ok(),
        _ => None,
    }
}
