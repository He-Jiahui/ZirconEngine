use std::collections::BTreeMap;

use toml::Value;

use super::super::super::pane_value_conversion::value_as_string;
use super::super::attribute_values::value_as_i32;

const MUI_TRANSITION_ENTERING_SCREEN_MS: i32 = 225;
const MUI_TRANSITION_LEAVING_SCREEN_MS: i32 = 195;
const MUI_TRANSITION_STANDARD_MS: i32 = 300;
const MUI_EASING_EASE_IN_OUT: &str = "cubic-bezier(0.4, 0, 0.2, 1)";
const MUI_EASING_EASE_OUT: &str = "cubic-bezier(0.0, 0, 0.2, 1)";
const MUI_EASING_SHARP: &str = "cubic-bezier(0.4, 0, 0.6, 1)";

pub(super) fn projected_transition_duration_ms(
    attributes: &BTreeMap<String, Value>,
    kind: &str,
    transition_in: bool,
) -> i32 {
    attributes
        .get("transition_duration_ms")
        .or_else(|| attributes.get("timeout_ms"))
        .or_else(|| attributes.get("duration_ms"))
        .and_then(value_as_i32)
        .unwrap_or_else(|| default_duration_ms(kind, transition_in))
}

pub(super) fn projected_transition_easing(
    attributes: &BTreeMap<String, Value>,
    kind: &str,
    transition_in: bool,
) -> String {
    attributes
        .get("transition_easing")
        .or_else(|| attributes.get("easing"))
        .and_then(value_as_string)
        .unwrap_or_else(|| default_easing(kind, transition_in).to_string())
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
