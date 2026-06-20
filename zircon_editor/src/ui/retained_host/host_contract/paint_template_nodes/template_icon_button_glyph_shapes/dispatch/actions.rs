use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_kind::IconButtonGlyphKind;
use super::super::actions as action_shapes;

pub(super) fn push_action_button_glyph_shape(
    commands: &mut Vec<HostPaintCommand>,
    kind: IconButtonGlyphKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        IconButtonGlyphKind::Plus => {
            action_shapes::push_plus_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Trash => {
            action_shapes::push_trash_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Filter => {
            action_shapes::push_filter_icon(commands, rect, clip, order, color, opacity)
        }
        _ => unreachable!("non-action icon button glyph routed to action dispatch"),
    }
}
