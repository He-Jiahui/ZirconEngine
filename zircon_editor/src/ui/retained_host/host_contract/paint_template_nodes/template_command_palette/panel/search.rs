use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::search_rect;
use super::super::palette::command_palette_palette;

mod icon;
mod surface;
mod text;

use icon::push_command_palette_search_icon;
use surface::push_command_palette_search_surface;
use text::push_command_palette_search_text;

pub(super) fn push_command_palette_search_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let palette = command_palette_palette();
    let search_rect = search_rect(rect);
    push_command_palette_search_surface(
        commands,
        node,
        &search_rect,
        clip,
        order,
        opacity,
        &palette,
    );
    push_command_palette_search_icon(commands, &search_rect, clip, order, opacity, &palette);

    push_command_palette_search_text(commands, node, &search_rect, clip, order, opacity, &palette);
}
