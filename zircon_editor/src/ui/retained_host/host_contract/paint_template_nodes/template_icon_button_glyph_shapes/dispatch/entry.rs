use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_kind::IconButtonGlyphKind;
use super::{actions, assets, chrome, files, tools, visibility};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_glyph_shape(
    commands: &mut Vec<HostPaintCommand>,
    kind: IconButtonGlyphKind,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        IconButtonGlyphKind::Menu
        | IconButtonGlyphKind::File
        | IconButtonGlyphKind::Folder
        | IconButtonGlyphKind::Save => {
            files::push_file_button_glyph_shape(commands, kind, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Cursor
        | IconButtonGlyphKind::Move
        | IconButtonGlyphKind::Rotate
        | IconButtonGlyphKind::Scale
        | IconButtonGlyphKind::Snap => {
            tools::push_tool_button_glyph_shape(commands, kind, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Play
        | IconButtonGlyphKind::ChevronDown
        | IconButtonGlyphKind::Grid
        | IconButtonGlyphKind::Sun
        | IconButtonGlyphKind::More => chrome::push_chrome_button_glyph_shape(
            commands, kind, rect, clip, order, color, opacity,
        ),
        IconButtonGlyphKind::Plus | IconButtonGlyphKind::Trash | IconButtonGlyphKind::Filter => {
            actions::push_action_button_glyph_shape(
                commands, kind, rect, clip, order, color, opacity,
            )
        }
        IconButtonGlyphKind::Cube
        | IconButtonGlyphKind::Graph
        | IconButtonGlyphKind::Image
        | IconButtonGlyphKind::Audio
        | IconButtonGlyphKind::Code => {
            assets::push_asset_button_glyph_shape(commands, kind, rect, clip, order, color, opacity)
        }
        IconButtonGlyphKind::Eye | IconButtonGlyphKind::EyeOff | IconButtonGlyphKind::Lock => {
            visibility::push_visibility_button_glyph_shape(
                commands, kind, rect, clip, order, color, opacity,
            )
        }
    }
}
