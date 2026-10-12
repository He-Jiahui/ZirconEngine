use std::collections::BTreeMap;

use toml::Value;

use super::attributes::f32_attribute;

pub(super) struct ProjectedDragCursor {
    pub(super) has_cursor: bool,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) offset_x: f32,
    pub(super) offset_y: f32,
    pub(super) preview_width: f32,
    pub(super) preview_height: f32,
}

pub(super) fn projected_drag_cursor(attributes: &BTreeMap<String, Value>) -> ProjectedDragCursor {
    let cursor_x = f32_attribute(attributes, "cursor_x");
    let cursor_y = f32_attribute(attributes, "cursor_y");

    ProjectedDragCursor {
        has_cursor: cursor_x.is_some() && cursor_y.is_some(),
        x: cursor_x.unwrap_or(0.0),
        y: cursor_y.unwrap_or(0.0),
        offset_x: f32_attribute(attributes, "offset_x").unwrap_or(12.0),
        offset_y: f32_attribute(attributes, "offset_y").unwrap_or(12.0),
        preview_width: f32_attribute(attributes, "preview_width").unwrap_or(0.0),
        preview_height: f32_attribute(attributes, "preview_height").unwrap_or(0.0),
    }
}
