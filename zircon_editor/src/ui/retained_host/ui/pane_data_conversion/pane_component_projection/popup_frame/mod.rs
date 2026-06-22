mod attributes;
mod origin;
mod overlay;
mod placement;

use crate::ui::retained_host as host_contract;

use self::attributes::float_attribute;
use self::origin::{
    default_anchor_origin_horizontal, default_anchor_origin_vertical,
    default_transform_origin_horizontal, default_transform_origin_vertical, origin_axis,
    origin_offset,
};
use self::overlay::{is_anchor_positioned_overlay, uses_popper_placement};
use self::placement::{default_popper_placement, popper_position};
use super::super::pane_value_conversion::value_as_string;

pub(super) fn projected_popup_frame(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    component_role: &str,
    popup_open: bool,
    popup_anchor_x: Option<f32>,
    popup_anchor_y: Option<f32>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> host_contract::TemplateNodeFrameData {
    let mut frame = host_contract::TemplateNodeFrameData {
        x,
        y,
        width,
        height,
    };
    if !popup_open || !is_anchor_positioned_overlay(component_role) {
        return frame;
    }
    let (Some(anchor_x), Some(anchor_y)) = (popup_anchor_x, popup_anchor_y) else {
        return frame;
    };

    let anchor_width = float_attribute(attributes, "popup_anchor_width")
        .or_else(|| float_attribute(attributes, "anchor_width"))
        .unwrap_or(0.0);
    let anchor_height = float_attribute(attributes, "popup_anchor_height")
        .or_else(|| float_attribute(attributes, "anchor_height"))
        .unwrap_or(0.0);
    let offset_x = float_attribute(attributes, "popup_offset_x")
        .or_else(|| float_attribute(attributes, "offset_x"))
        .unwrap_or(0.0);
    let offset_y = float_attribute(attributes, "popup_offset_y")
        .or_else(|| float_attribute(attributes, "offset_y"))
        .unwrap_or(0.0);

    if uses_popper_placement(component_role, attributes) {
        let placement = attributes
            .get("placement")
            .and_then(value_as_string)
            .unwrap_or_else(|| default_popper_placement(component_role).to_string());
        let (left, top) = popper_position(
            &placement,
            component_role,
            anchor_x,
            anchor_y,
            anchor_width,
            anchor_height,
            width,
            height,
        );
        frame.x = left + offset_x;
        frame.y = top + offset_y;
        return frame;
    }

    let anchor_vertical = origin_axis(
        attributes,
        "anchor_origin_vertical",
        default_anchor_origin_vertical(component_role),
    );
    let anchor_horizontal = origin_axis(
        attributes,
        "anchor_origin_horizontal",
        default_anchor_origin_horizontal(component_role),
    );
    let transform_vertical = origin_axis(
        attributes,
        "transform_origin_vertical",
        default_transform_origin_vertical(component_role),
    );
    let transform_horizontal = origin_axis(
        attributes,
        "transform_origin_horizontal",
        default_transform_origin_horizontal(component_role),
    );

    frame.x = anchor_x + origin_offset(anchor_width, &anchor_horizontal)
        - origin_offset(width, &transform_horizontal)
        + offset_x;
    frame.y = anchor_y + origin_offset(anchor_height, &anchor_vertical)
        - origin_offset(height, &transform_vertical)
        + offset_y;
    frame
}

#[cfg(test)]
mod tests;
