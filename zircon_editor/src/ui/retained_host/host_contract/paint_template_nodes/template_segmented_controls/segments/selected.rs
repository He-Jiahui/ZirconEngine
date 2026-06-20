use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_segmented_control_geometry::{
    selected_segment_rect, selected_segment_underline_rect, SEGMENT_RADIUS,
};
use super::super::style::segmented_control_style;

pub(super) fn push_selected_segment(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let selected_rect = selected_segment_rect(segment);
    let style = segmented_control_style(node);
    let border_width = style.selected_border_width;
    commands.push(HostPaintCommand::quad(
        selected_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.selected_surface),
        (border_width > 0.0).then_some(style.selected_border),
        border_width,
        (SEGMENT_RADIUS - 1.0).max(0.0),
        opacity,
    ));

    let underline_height = style.selected_underline_height;
    if underline_height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        selected_segment_underline_rect(&selected_rect, underline_height),
        Some(clip.clone()),
        order + 1,
        Some(style.selected_underline),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
