use std::collections::BTreeMap;

use super::super::super::pane_value_conversion::value_as_f64;
use super::attributes::f32_attribute;

pub(super) struct ProjectedLayoutOffsets {
    pub(super) layout_offset_x: f32,
    pub(super) layout_offset_y: f32,
    pub(super) layout_icon_size: f32,
    pub(super) layout_content_offset_x: f32,
    pub(super) layout_content_offset_y: f32,
    pub(super) layout_padding_left: f32,
    pub(super) layout_padding_right: f32,
    pub(super) layout_padding_top: f32,
    pub(super) layout_padding_bottom: f32,
    pub(super) layout_spacing: f32,
    pub(super) layout_first_cell_offset_x: f32,
    pub(super) layout_second_cell_offset_x: f32,
    pub(super) layout_third_cell_offset_x: f32,
    pub(super) layout_fourth_cell_offset_x: f32,
}

pub(super) fn projected_layout_offsets(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedLayoutOffsets {
    ProjectedLayoutOffsets {
        layout_offset_x: f32_attribute(attributes, "layout_offset_x", 0.0),
        layout_offset_y: f32_attribute(attributes, "layout_offset_y", 0.0),
        layout_icon_size: attributes
            .get("layout_icon_size")
            .or_else(|| attributes.get("thumb_size"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        layout_content_offset_x: attributes
            .get("layout_content_offset_x")
            .or_else(|| attributes.get("layout_gap"))
            .or_else(|| attributes.get("layout_spacing"))
            .or_else(|| attributes.get("track_offset_x"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        layout_content_offset_y: attributes
            .get("layout_content_offset_y")
            .or_else(|| attributes.get("icon_offset_y"))
            .or_else(|| attributes.get("track_height"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        layout_padding_left: f32_attribute(attributes, "layout_padding_left", 0.0),
        layout_padding_right: f32_attribute(attributes, "layout_padding_right", 0.0),
        layout_padding_top: f32_attribute(attributes, "layout_padding_top", 0.0),
        layout_padding_bottom: f32_attribute(attributes, "layout_padding_bottom", 0.0),
        layout_spacing: f32_attribute(attributes, "layout_spacing", 0.0),
        layout_first_cell_offset_x: attributes
            .get("layout_first_cell_offset_x")
            .or_else(|| attributes.get("track_width_delta"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        layout_second_cell_offset_x: attributes
            .get("layout_second_cell_offset_x")
            .or_else(|| attributes.get("range_min"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        layout_third_cell_offset_x: attributes
            .get("layout_third_cell_offset_x")
            .or_else(|| attributes.get("step_tick_count"))
            .and_then(value_as_f64)
            .unwrap_or(0.0) as f32,
        layout_fourth_cell_offset_x: f32_attribute(attributes, "layout_fourth_cell_offset_x", 0.0),
    }
}
