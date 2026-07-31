use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_selection_control_geometry::{
    frame_is_within, selection_label_gap, toggle_thumb_rect, toggle_track_rect,
    workbench_selection_control_metrics,
};
use super::labels::push_selection_label;
use super::layers::{toggle_label_order, toggle_thumb_order};
use super::style::{
    control_border_color, selection_text_color, toggle_thumb_color, toggle_track_color,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_toggle(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let track = toggle_track_rect(node, rect);
    let metrics = workbench_selection_control_metrics();
    let label_rect = FrameRect {
        x: rect.x + metrics.mark_inset_x,
        y: rect.y + metrics.text_inset_y,
        width: (track.x - rect.x - metrics.mark_inset_x - selection_label_gap(node)).max(0.0),
        height: (rect.height - metrics.text_inset_y * 2.0).max(0.0),
    };
    if frame_is_within(&label_rect, rect) {
        push_selection_label(
            commands,
            node,
            label_rect,
            clip,
            toggle_label_order(order),
            selection_text_color(node),
            opacity,
        );
    }

    if !frame_is_within(&track, rect) {
        return;
    }
    commands.push(HostPaintCommand::quad(
        track.clone(),
        Some(clip.clone()),
        order,
        Some(toggle_track_color(node)),
        Some(control_border_color(node)),
        1.0,
        track.height * 0.5,
        opacity,
    ));
    let thumb = toggle_thumb_rect(node, &track);
    if frame_is_within(&thumb, &track) {
        commands.push(HostPaintCommand::quad(
            thumb.clone(),
            Some(clip.clone()),
            toggle_thumb_order(order),
            Some(toggle_thumb_color(node)),
            None,
            0.0,
            thumb.height * 0.5,
            opacity,
        ));
    }
}
