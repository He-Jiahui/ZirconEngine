use crate::ui::retained_host as host_contract;

use super::super::pane_value_conversion::{value_as_f64, value_as_string};

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

fn is_anchor_positioned_overlay(component_role: &str) -> bool {
    matches!(
        component_role,
        "popover" | "popper" | "tooltip" | "menu" | "context-menu"
    )
}

fn uses_popper_placement(
    component_role: &str,
    attributes: &std::collections::BTreeMap<String, toml::Value>,
) -> bool {
    matches!(component_role, "popper" | "tooltip")
        || attributes
            .get("placement")
            .and_then(value_as_string)
            .is_some_and(|placement| placement.contains('-'))
}

fn default_popper_placement(component_role: &str) -> &'static str {
    match component_role {
        "tooltip" => "top",
        "menu" => "bottom-start",
        "popper" => "bottom-start",
        _ => "bottom",
    }
}

fn popper_position(
    placement: &str,
    component_role: &str,
    anchor_x: f32,
    anchor_y: f32,
    anchor_width: f32,
    anchor_height: f32,
    width: f32,
    height: f32,
) -> (f32, f32) {
    let (side, align) = split_placement(placement);
    let gap = if component_role == "tooltip" {
        8.0
    } else {
        0.0
    };
    match side {
        "top" => (
            horizontal_aligned(anchor_x, anchor_width, width, align),
            anchor_y - height - gap,
        ),
        "left" => (
            anchor_x - width - gap,
            vertical_aligned(anchor_y, anchor_height, height, align),
        ),
        "right" => (
            anchor_x + anchor_width + gap,
            vertical_aligned(anchor_y, anchor_height, height, align),
        ),
        _ => (
            horizontal_aligned(anchor_x, anchor_width, width, align),
            anchor_y + anchor_height + gap,
        ),
    }
}

fn split_placement(placement: &str) -> (&str, &str) {
    placement.split_once('-').unwrap_or((placement, "center"))
}

fn horizontal_aligned(anchor_x: f32, anchor_width: f32, width: f32, align: &str) -> f32 {
    match align {
        "start" | "left" => anchor_x,
        "end" | "right" => anchor_x + anchor_width - width,
        _ => anchor_x + anchor_width * 0.5 - width * 0.5,
    }
}

fn vertical_aligned(anchor_y: f32, anchor_height: f32, height: f32, align: &str) -> f32 {
    match align {
        "start" | "top" => anchor_y,
        "end" | "bottom" => anchor_y + anchor_height - height,
        _ => anchor_y + anchor_height * 0.5 - height * 0.5,
    }
}

fn origin_axis(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
    default: &str,
) -> String {
    attributes
        .get(key)
        .and_then(value_as_string)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn default_anchor_origin_vertical(component_role: &str) -> &'static str {
    match component_role {
        "menu" | "context-menu" => "bottom",
        _ => "top",
    }
}

fn default_anchor_origin_horizontal(_component_role: &str) -> &'static str {
    "left"
}

fn default_transform_origin_vertical(_component_role: &str) -> &'static str {
    "top"
}

fn default_transform_origin_horizontal(_component_role: &str) -> &'static str {
    "left"
}

fn origin_offset(length: f32, axis: &str) -> f32 {
    match axis {
        "center" => length * 0.5,
        "bottom" | "right" | "end" => length,
        value => value.parse::<f32>().unwrap_or(0.0),
    }
}

fn float_attribute(
    attributes: &std::collections::BTreeMap<String, toml::Value>,
    key: &str,
) -> Option<f32> {
    attributes
        .get(key)
        .and_then(value_as_f64)
        .map(|value| value as f32)
}
