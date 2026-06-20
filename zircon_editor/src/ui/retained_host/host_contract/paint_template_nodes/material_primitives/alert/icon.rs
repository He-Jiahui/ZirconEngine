use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::{alert_icon_frame, alert_icon_mark_frame};
use super::style::{alert_icon_color, alert_icon_cutout_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = alert_icon_frame(rect);
    let color = alert_icon_color(node);
    let mark = alert_icon_mark_frame(&frame);
    let mark_radius = mark.height * 0.5;
    commands.push(HostPaintCommand::quad(
        mark.clone(),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        mark_radius,
        opacity,
    ));

    let center_x = mark.x + mark.width * 0.5;
    let center_y = mark.y + mark.height * 0.5;
    let cutout = alert_icon_cutout_color(node);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: center_x - 1.0,
            y: center_y - 4.0,
            width: 2.0,
            height: 6.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(cutout),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: center_x - 1.0,
            y: center_y + 4.0,
            width: 2.0,
            height: 2.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(cutout),
        None,
        0.0,
        1.0,
        opacity,
    ));
}
