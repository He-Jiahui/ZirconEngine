use std::collections::BTreeMap;

use toml::Value;

use super::attributes::{first_non_empty_string_attribute, string_attribute};

pub(super) struct ProjectedDragPayload {
    pub(super) kind: String,
    pub(super) label: String,
    pub(super) reference: String,
}

pub(super) fn projected_drag_payload(attributes: &BTreeMap<String, Value>) -> ProjectedDragPayload {
    ProjectedDragPayload {
        kind: string_attribute(attributes, "payload_kind").unwrap_or_else(|| "unknown".to_string()),
        label: first_non_empty_string_attribute(
            attributes,
            &["payload_label", "label", "text", "payload_reference"],
        )
        .unwrap_or_default(),
        reference: string_attribute(attributes, "payload_reference").unwrap_or_default(),
    }
}
