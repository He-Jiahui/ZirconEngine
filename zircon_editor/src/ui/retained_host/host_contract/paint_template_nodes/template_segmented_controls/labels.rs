use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_segmented_control_geometry::{
    segment_group_label_line_height, segment_label_rect, segment_line_height,
    segmented_group_label_rect, SEGMENT_FONT_SIZE, SEGMENT_GROUP_LABEL_FONT_SIZE,
};
use super::options::segment_label;
use super::style::{segment_text_color, segmented_group_label_color};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segmented_group_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = node.label_text.trim();
    if label.is_empty() {
        return;
    }

    commands.push(HostPaintCommand::text(
        segmented_group_label_rect(rect),
        Some(clip.clone()),
        order,
        label.to_string(),
        segmented_group_label_color(node),
        SEGMENT_GROUP_LABEL_FONT_SIZE,
        segment_group_label_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segment_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    option: &str,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    selected: bool,
    opacity: f32,
) {
    let label = segment_label(option);
    commands.push(HostPaintCommand::text(
        segment_label_rect(segment),
        Some(clip.clone()),
        order,
        label,
        segment_text_color(node, selected),
        SEGMENT_FONT_SIZE,
        segment_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
