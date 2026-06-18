mod actions;
mod assets;
mod chrome;
mod files;
mod tools;
mod visibility;

use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;
use super::template_icon_button_glyph_kind::IconButtonGlyphKind;

pub(super) fn push_icon_button_glyph_shape(
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
            files::push_menu_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::File => {
            files::push_file_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Folder => {
            files::push_folder_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Save => {
            files::push_save_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Cursor => {
            tools::push_cursor_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Move => {
            tools::push_move_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Rotate => {
            tools::push_rotate_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Scale => {
            tools::push_scale_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Snap => {
            tools::push_snap_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Play => {
            chrome::push_play_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::ChevronDown => {
            chrome::push_chevron_down_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Grid => {
            chrome::push_grid_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Sun => {
            chrome::push_sun_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::More => {
            chrome::push_more_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Plus => {
            actions::push_plus_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Trash => {
            actions::push_trash_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Filter => {
            actions::push_filter_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Cube => {
            assets::push_cube_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Graph => {
            assets::push_graph_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Image => {
            assets::push_image_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Audio => {
            assets::push_audio_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Code => {
            assets::push_code_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Eye => {
            visibility::push_eye_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::EyeOff => {
            visibility::push_eye_off_icon(commands, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Lock => {
            visibility::push_lock_icon(commands, rect, clip, order, color, opacity)
        }
    }
}
