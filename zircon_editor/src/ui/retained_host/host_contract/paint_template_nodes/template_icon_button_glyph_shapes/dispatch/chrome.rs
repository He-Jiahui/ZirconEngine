use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_kind::IconButtonGlyphKind;
use super::super::chrome as chrome_shapes;

pub(super) fn push_chrome_button_glyph_shape(
    commands: &mut Vec<HostPaintCommand>,
    kind: IconButtonGlyphKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        IconButtonGlyphKind::Play => {
            chrome_shapes::push_play_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::ChevronDown => {
            chrome_shapes::push_chevron_down_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Grid => {
            chrome_shapes::push_grid_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Sun => {
            chrome_shapes::push_sun_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::More => {
            chrome_shapes::push_more_icon(commands, rect, clip, order, color, opacity)
        }
        _ => unreachable!("non-chrome icon button glyph routed to chrome dispatch"),
    }
}
