use std::collections::BTreeMap;

use crate::ui::retained_host as host_contract;

use super::super::super::pane_option_projection::structured_options_for_node;
pub(super) fn projected_structured_options(
    attributes: &BTreeMap<String, toml::Value>,
    options: &[String],
) -> Vec<host_contract::TemplatePaneOptionData> {
    structured_options_for_node(options, attributes)
}
