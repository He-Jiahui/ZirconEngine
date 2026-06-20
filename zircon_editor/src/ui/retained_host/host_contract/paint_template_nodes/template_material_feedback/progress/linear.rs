use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_style::template_corner_radius;
use super::super::state::{
    progress_fill_color, progress_is_indeterminate, progress_percent, progress_track_color,
};

pub(super) fn push_linear_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = template_corner_radius(node)
        .max((rect.height * 0.5).min(2.0))
        .max(0.0);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(progress_track_color(node)),
        None,
        0.0,
        radius,
        opacity,
    ));

    let fill = progress_fill_color(node);
    if progress_is_indeterminate(node) {
        for (x_factor, width_factor) in [(0.12, 0.36), (0.62, 0.24)] {
            let bar = FrameRect {
                x: rect.x + rect.width * x_factor,
                y: rect.y,
                width: (rect.width * width_factor).max(1.0),
                height: rect.height,
            };
            commands.push(HostPaintCommand::quad(
                bar,
                Some(clip.clone()),
                order + 1,
                Some(fill),
                None,
                0.0,
                radius,
                opacity,
            ));
        }
        return;
    }

    let width = rect.width * progress_percent(node);
    if width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: width.max(1.0),
            height: rect.height,
        },
        Some(clip.clone()),
        order + 1,
        Some(fill),
        None,
        0.0,
        radius,
        opacity,
    ));
}
