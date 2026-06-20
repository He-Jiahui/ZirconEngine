use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_kind::IconButtonGlyphKind;
use super::super::files as file_shapes;

pub(super) fn push_file_button_glyph_shape(
    commands: &mut Vec<HostPaintCommand>,
    kind: IconButtonGlyphKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        IconButtonGlyphKind::Menu => {
            file_shapes::push_menu_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::File => {
            file_shapes::push_file_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Folder => {
            file_shapes::push_folder_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Save => {
            file_shapes::push_save_icon(commands, rect, clip, order, color, opacity)
        }
        _ => unreachable!("non-file icon button glyph routed to file dispatch"),
    }
}
