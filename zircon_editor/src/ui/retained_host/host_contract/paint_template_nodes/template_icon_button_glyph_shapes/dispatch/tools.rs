use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_kind::IconButtonGlyphKind;
use super::super::tools as tool_shapes;

pub(super) fn push_tool_button_glyph_shape(
    commands: &mut Vec<HostPaintCommand>,
    kind: IconButtonGlyphKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        IconButtonGlyphKind::Cursor => {
            tool_shapes::push_cursor_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Move => {
            tool_shapes::push_move_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Rotate => {
            tool_shapes::push_rotate_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Scale => {
            tool_shapes::push_scale_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Snap => {
            tool_shapes::push_snap_icon(commands, rect, clip, order, color, opacity)
        }
        _ => unreachable!("non-tool icon button glyph routed to tool dispatch"),
    }
}
