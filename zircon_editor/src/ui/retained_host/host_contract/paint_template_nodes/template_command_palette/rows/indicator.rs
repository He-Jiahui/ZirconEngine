use super::super::super::super::data::{FrameRect, TemplatePaneOptionData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{command_palette_metrics, match_indicator_radius, match_indicator_rect};

mod color;
mod style;

use style::command_row_match_indicator_style;

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
    let rect = match_indicator_rect(row_rect, &metrics);
    let style = command_row_match_indicator_style(option, match_indicator_radius(&metrics));
    commands.push(HostPaintCommand::quad(
        rect,
        Some(clip.clone()),
        order,
        Some(style.fill),
        style.border,
        style.border_width,
        style.radius,
        opacity,
    ));
}
