use std::collections::BTreeMap;

use toml::Value;

use super::attributes::string_attribute;

pub(super) struct ProjectedDropIndicator {
    pub(super) edge: String,
    pub(super) text: String,
}

pub(super) fn projected_drop_indicator(
    attributes: &BTreeMap<String, Value>,
) -> ProjectedDropIndicator {
    ProjectedDropIndicator {
        edge: string_attribute(attributes, "drop_indicator_edge")
            .unwrap_or_else(|| "none".to_string()),
        text: string_attribute(attributes, "drop_indicator_text").unwrap_or_default(),
    }
}
