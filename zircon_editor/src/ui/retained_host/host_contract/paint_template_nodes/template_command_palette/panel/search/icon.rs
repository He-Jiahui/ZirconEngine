use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::super::layers::search_icon_order;
use super::super::super::layout::search_icon_rect;
use super::super::super::palette::WorkbenchCommandPalettePalette;
use super::super::super::text::command_palette_search_icon;

mod style;

use style::command_palette_search_icon_style;

pub(super) fn push_command_palette_search_icon(
    commands: &mut Vec<HostPaintCommand>,
    search_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: &WorkbenchCommandPalettePalette,
) {
    let icon_rect = search_icon_rect(search_rect);
    let style = command_palette_search_icon_style(palette);
    push_icon_asset_pixels(
        commands,
        command_palette_search_icon(),
        &icon_rect,
        clip,
        search_icon_order(order),
        style.tint,
        opacity,
    );
}
