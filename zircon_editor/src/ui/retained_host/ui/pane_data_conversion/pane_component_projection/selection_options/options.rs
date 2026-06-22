use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_options;
use super::super::command_palette::projected_command_palette_options;
use super::super::notification_center::projected_notification_center_options;

pub(super) fn projected_options(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Vec<String> {
    projected_command_palette_options(component_role, attributes)
        .or_else(|| projected_notification_center_options(component_role, attributes))
        .or_else(|| attributes.get("options").and_then(value_as_options))
        .unwrap_or_default()
}
