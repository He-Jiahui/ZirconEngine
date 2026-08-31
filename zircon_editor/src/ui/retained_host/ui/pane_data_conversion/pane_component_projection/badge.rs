use std::collections::BTreeMap;

use super::super::pane_value_conversion::{value_as_f64, value_as_string};
use zircon_runtime_interface::ui::component::UiValue;

pub(super) fn projected_badge_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Option<String> {
    if component_role != "badge" {
        return None;
    }
    let variant = badge_variant(attributes);
    if variant == "dot" {
        return Some(String::new());
    }
    let content = attributes
        .get("badgeContent")
        .or_else(|| attributes.get("badge_content"))?;
    let max = attributes.get("max").and_then(value_as_f64).unwrap_or(99.0);
    if badge_content_number(content).is_some_and(|value| value > max) {
        return Some(format!("{}+", max.round() as i64));
    }
    Some(UiValue::from_toml(content).display_text())
}

fn badge_variant(attributes: &BTreeMap<String, toml::Value>) -> &str {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(toml::Value::as_str)
        .unwrap_or("standard")
}

fn badge_content_number(value: &toml::Value) -> Option<f64> {
    value_as_f64(value).or_else(|| value_as_string(value)?.trim().parse::<f64>().ok())
}

#[cfg(test)]
#[path = "badge/borrowed_variant_tests.rs"]
mod borrowed_variant_tests;
