use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::first_non_empty;
use geometry::{
    divider_is_vertical, horizontal_divider_extent, horizontal_label_bounds,
    horizontal_label_text_frame, horizontal_line_frame, horizontal_line_y, vertical_divider_extent,
    vertical_label_bounds, vertical_label_text_frame, vertical_line_frame, vertical_line_x,
};
use style::{divider_color, divider_text_color};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

mod geometry;
mod style;

pub(super) fn push_divider_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_divider_node(node) {
        return false;
    }

    // MUI Divider is border/pseudo-element geometry, not a filled panel.
    // Emit explicit line segments so inset, middle, and label gaps match the web contract.
    if divider_is_vertical(node, rect) {
        push_vertical_divider(commands, node, rect, clip, order, opacity);
    } else {
        push_horizontal_divider(commands, node, rect, clip, order, opacity);
    }
    true
}

fn is_divider_node(node: &TemplatePaneNodeData) -> bool {
    matches_any_role(
        node.component_role.as_str(),
        node.role.as_str(),
        &["divider", "Divider"],
    ) || node.surface_variant.as_str() == "divider"
}

fn matches_any_role(component_role: &str, role: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| component_role == *candidate || role == *candidate)
}

fn push_horizontal_divider(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (line_start, line_end) = horizontal_divider_extent(node, rect);
    let line_y = horizontal_line_y(rect);
    let label = divider_label(node);
    if label.is_empty() {
        push_horizontal_line(
            commands, line_start, line_end, line_y, clip, order, node, opacity,
        );
        return;
    }

    let (label_left, label_right) = horizontal_label_bounds(node, line_start, line_end, &label);

    push_horizontal_line(
        commands, line_start, label_left, line_y, clip, order, node, opacity,
    );
    push_horizontal_line(
        commands,
        label_right,
        line_end,
        line_y,
        clip,
        order,
        node,
        opacity,
    );
    push_horizontal_divider_label(
        commands,
        node,
        &label,
        label_left,
        label_right,
        rect,
        clip,
        order + 1,
        opacity,
    );
}

fn push_vertical_divider(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (line_top, line_bottom) = vertical_divider_extent(node, rect);
    let line_x = vertical_line_x(rect);
    let label = divider_label(node);
    if label.is_empty() {
        push_vertical_line(
            commands,
            line_x,
            line_top,
            line_bottom,
            clip,
            order,
            node,
            opacity,
        );
        return;
    }

    let (label_top, label_bottom) = vertical_label_bounds(node, rect, line_bottom);

    push_vertical_line(
        commands, line_x, line_top, label_top, clip, order, node, opacity,
    );
    push_vertical_line(
        commands,
        line_x,
        label_bottom,
        line_bottom,
        clip,
        order,
        node,
        opacity,
    );
    push_vertical_divider_label(
        commands,
        node,
        &label,
        label_top,
        label_bottom,
        rect,
        clip,
        order + 1,
        opacity,
    );
}

fn push_horizontal_line(
    commands: &mut Vec<HostPaintCommand>,
    left: f32,
    right: f32,
    y: f32,
    clip: &FrameRect,
    order: i32,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    let Some(frame) = horizontal_line_frame(left, right, y) else {
        return;
    };
    push_quad(commands, frame, clip, order, divider_color(node), opacity);
}

fn push_vertical_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    top: f32,
    bottom: f32,
    clip: &FrameRect,
    order: i32,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    let Some(frame) = vertical_line_frame(x, top, bottom) else {
        return;
    };
    push_quad(commands, frame, clip, order, divider_color(node), opacity);
}

fn push_horizontal_divider_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    label: &str,
    label_left: f32,
    label_right: f32,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if label.trim().is_empty() || label_right <= label_left {
        return;
    }
    let Some((frame, font_size, line_height)) =
        horizontal_label_text_frame(node, label, label_left, label_right, rect)
    else {
        return;
    };
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        label.to_string(),
        divider_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_vertical_divider_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    label: &str,
    label_top: f32,
    label_bottom: f32,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if label.trim().is_empty() || label_bottom <= label_top {
        return;
    }
    let Some((frame, font_size, line_height)) =
        vertical_label_text_frame(node, label, label_top, label_bottom, rect)
    else {
        return;
    };
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        label.to_string(),
        divider_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_quad(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn divider_label(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.options_text.as_str(),
    ])
    .to_string()
}
