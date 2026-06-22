use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_f64;
use super::values::variant_contains;

pub(in super::super) fn projected_border_width(
    attributes: &BTreeMap<String, toml::Value>,
    component_role: &str,
    component_variant: &str,
) -> f32 {
    attributes
        .get("border_width")
        .and_then(value_as_f64)
        .map(|value| value as f32)
        .unwrap_or_else(|| {
            if component_role == "alert" && variant_contains(component_variant, "outlined") {
                1.0
            } else if matches!(component_role, "card" | "paper")
                && component_variant
                    .split_whitespace()
                    .any(|part| part == "outlined")
            {
                1.0
            } else if matches!(component_role, "dialog" | "confirm-dialog" | "alert-dialog") {
                1.0
            } else {
                0.0
            }
        })
}
