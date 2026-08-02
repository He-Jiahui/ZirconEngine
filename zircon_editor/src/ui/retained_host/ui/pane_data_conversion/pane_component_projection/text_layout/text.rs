use std::collections::BTreeMap;

use super::super::attribute_values::{
    first_non_empty_string_attribute, humanize_control_id, should_humanize_control_label,
};
use super::super::drag_overlay::ProjectedDragOverlayData;

pub(super) fn projected_text(
    control_id: &str,
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    has_bindings: bool,
    drag_overlay: &ProjectedDragOverlayData,
) -> String {
    let text_names: &[&str] = match component_role {
        "card-header" => &["text", "label", "title", "subheader"],
        "dialog" | "confirm-dialog" | "alert-dialog" => &["title", "text", "label"],
        "notification-center" => &["title", "text", "label"],
        "snackbar" | "snackbar-content" => &["text", "label", "message"],
        _ => &["text", "label"],
    };
    drag_overlay
        .text
        .clone()
        .or_else(|| first_non_empty_string_attribute(attributes, text_names))
        .or_else(|| {
            (has_bindings || should_humanize_control_label(control_id))
                .then(|| humanize_control_id(control_id))
        })
        .unwrap_or_default()
}

pub(super) fn projected_icon_placement(attributes: &BTreeMap<String, toml::Value>) -> String {
    first_non_empty_string_attribute(attributes, &["icon_placement"]).unwrap_or_default()
}
