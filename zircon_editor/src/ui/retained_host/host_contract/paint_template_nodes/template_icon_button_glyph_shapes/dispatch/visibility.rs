use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_kind::IconButtonGlyphKind;
use super::super::visibility as visibility_shapes;

pub(super) fn push_visibility_button_glyph_shape(
    commands: &mut Vec<HostPaintCommand>,
    kind: IconButtonGlyphKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        IconButtonGlyphKind::Eye => {
            visibility_shapes::push_eye_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::EyeOff => {
            visibility_shapes::push_eye_off_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Lock => {
            visibility_shapes::push_lock_icon(commands, rect, clip, order, color, opacity)
        }
        _ => unreachable!("non-visibility icon button glyph routed to visibility dispatch"),
    }
}
