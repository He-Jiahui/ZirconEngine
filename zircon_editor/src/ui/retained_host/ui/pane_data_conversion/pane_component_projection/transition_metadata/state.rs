use std::collections::BTreeMap;

use toml::Value;

use super::super::super::pane_value_conversion::{value_as_bool, value_as_f64, value_as_string};

pub(super) fn projected_transition_in(
    attributes: &BTreeMap<String, Value>,
    kind: &str,
    popup_open: bool,
) -> bool {
    attributes
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
        })
}

pub(super) fn projected_transition_status(
    attributes: &BTreeMap<String, Value>,
    transition_in: bool,
) -> String {
    attributes
        .get("transition_status")
        .and_then(value_as_string)
        .unwrap_or_else(|| {
            if transition_in {
                "entered".to_string()
            } else {
                "exited".to_string()
            }
        })
}

pub(super) fn projected_transition_progress(
    attributes: &BTreeMap<String, Value>,
    status: &str,
    transition_in: bool,
) -> f32 {
    attributes
        .get("transition_progress")
        .or_else(|| attributes.get("animation_progress"))
        .and_then(value_as_f64)
        .map(|value| value.clamp(0.0, 1.0) as f32)
        .unwrap_or_else(|| default_progress(status, transition_in))
}

pub(super) fn projected_transition_entered(
    attributes: &BTreeMap<String, Value>,
    transition_in: bool,
    status: &str,
    progress: f32,
) -> bool {
    attributes
        .get("transition_entered")
        .or_else(|| attributes.get("entered"))
        .and_then(value_as_bool)
        .unwrap_or_else(|| transition_in && status == "entered" && progress >= 1.0)
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
