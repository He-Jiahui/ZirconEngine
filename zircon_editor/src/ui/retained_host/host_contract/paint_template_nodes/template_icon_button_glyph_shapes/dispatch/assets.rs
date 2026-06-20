use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_kind::IconButtonGlyphKind;
use super::super::assets as asset_shapes;

pub(super) fn push_asset_button_glyph_shape(
    commands: &mut Vec<HostPaintCommand>,
    kind: IconButtonGlyphKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        IconButtonGlyphKind::Cube => {
            asset_shapes::push_cube_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Graph => {
            asset_shapes::push_graph_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Image => {
            asset_shapes::push_image_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Audio => {
            asset_shapes::push_audio_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Code => {
            asset_shapes::push_code_icon(commands, rect, clip, order, color, opacity)
        }
        _ => unreachable!("non-asset icon button glyph routed to asset dispatch"),
    }
}
