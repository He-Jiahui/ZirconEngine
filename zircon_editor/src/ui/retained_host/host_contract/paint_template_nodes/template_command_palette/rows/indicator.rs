use super::super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::command_palette_metrics;
use super::super::palette::command_palette_palette;

pub(super) fn push_command_row_match_indicator(
    commands: &mut Vec<HostPaintCommand>,
    option: &TemplatePaneOptionData,
    row_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !option.matched || intersect(row_rect, clip).is_none() {
        return;
    }

    let metrics = command_palette_metrics();
    let palette = command_palette_palette();
    let height = metrics
        .match_indicator_height
        .min((row_rect.height - metrics.match_indicator_width * 2.0).max(1.0));
    let rect = FrameRect {
        x: row_rect.x + metrics.match_indicator_left,
        y: row_rect.y + (row_rect.height - height).max(0.0) * 0.5,
        width: metrics.match_indicator_width,
        height,
    };
    let color = if option.disabled {
        palette.match_indicator_disabled
    } else {
        palette.match_indicator
    };
    commands.push(HostPaintCommand::quad(
        rect,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        metrics.match_indicator_width * 0.5,
        opacity,
    ));
}
