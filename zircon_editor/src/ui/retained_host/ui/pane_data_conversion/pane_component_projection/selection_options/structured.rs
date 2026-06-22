use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

use super::super::super::pane_option_projection::structured_options_for_node;
use super::super::command_palette::projected_command_palette_structured_options;
use super::super::notification_center::projected_notification_center_structured_options;

pub(super) fn projected_structured_options(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    options: &[String],
) -> Vec<host_contract::TemplatePaneOptionData> {
    projected_command_palette_structured_options(component_role, attributes)
        .or_else(|| projected_notification_center_structured_options(component_role, attributes))
        .unwrap_or_else(|| structured_options_for_node(options, attributes))
}
