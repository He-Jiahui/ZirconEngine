use std::collections::BTreeMap;

use toml::Value;

mod attributes;
mod cursor;
mod drop_target;
mod indicator;
mod model;
mod payload;

pub(super) use self::model::ProjectedDragOverlayData;

pub(super) fn projected_drag_overlay_data(
    component_role: &str,
    attributes: &BTreeMap<String, Value>,
) -> ProjectedDragOverlayData {
    let drop_source_summary =
        attributes::string_attribute(attributes, "drop_source_summary").unwrap_or_default();

    if component_role != "drag-overlay" {
        return ProjectedDragOverlayData {
            drop_source_summary,
            ..ProjectedDragOverlayData::default()
        };
    }

    let open = attributes::bool_attribute(attributes, "open").unwrap_or(false)
        || attributes::bool_attribute(attributes, "dragging").unwrap_or(false);
    let payload = payload::projected_drag_payload(attributes);
    let cursor = cursor::projected_drag_cursor(attributes);
    let drop_target = drop_target::projected_drop_target(attributes);
    let indicator = indicator::projected_drop_indicator(attributes);

    ProjectedDragOverlayData {
        popup_open: Some(open),
        text: (!payload.label.is_empty()).then(|| payload.label.clone()),
        value_text: (!payload.reference.is_empty()).then(|| payload.reference.clone()),
        payload_kind: payload.kind,
        payload_label: payload.label,
        payload_reference: payload.reference,
        has_cursor: cursor.has_cursor,
        cursor_x: cursor.x,
        cursor_y: cursor.y,
        offset_x: cursor.offset_x,
        offset_y: cursor.offset_y,
        preview_width: cursor.preview_width,
        preview_height: cursor.preview_height,
        drop_allowed: drop_target.allowed,
        has_drop_target: drop_target.has_target,
        drop_target_x: drop_target.x,
        drop_target_y: drop_target.y,
        drop_target_width: drop_target.width,
        drop_target_height: drop_target.height,
        drop_indicator_edge: indicator.edge,
        drop_indicator_text: indicator.text,
        drop_source_summary,
    }
}
