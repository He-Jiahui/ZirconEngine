use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::segments::{push_segments, GlyphSegmentSpec, TREE_ACTION_GLYPH_GRID_UNITS};

const TREE_LOCK_ICON: &str = "zircon_editor_shell/scene/lock.svg";
const LOCK_SEGMENTS: [GlyphSegmentSpec; 4] = [
    GlyphSegmentSpec::new(4, 6, 7, 6),
    GlyphSegmentSpec::new(5, 3, 5, 1),
    GlyphSegmentSpec::new(4, 4, 1, 3),
    GlyphSegmentSpec::new(10, 4, 1, 3),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_lock_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if push_icon_asset_pixels(
        commands,
        TREE_LOCK_ICON,
        rect,
        clip,
        order,
        Some(color),
        opacity,
    ) {
        return;
    }

    push_segments(
        commands,
        rect,
        TREE_ACTION_GLYPH_GRID_UNITS,
        clip,
        order,
        color,
        opacity,
        &LOCK_SEGMENTS,
    );
}
