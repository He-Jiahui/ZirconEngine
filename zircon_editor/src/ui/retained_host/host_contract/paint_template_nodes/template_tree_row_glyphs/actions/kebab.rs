use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::segments::{GlyphSegmentSpec, TREE_ACTION_GLYPH_GRID_UNITS, push_segments};

const TREE_MORE_ICON: &str = "zircon_editor_shell/toolbar/more-vertical.svg";
const KEBAB_SEGMENTS: [GlyphSegmentSpec; 3] = [
    GlyphSegmentSpec::new(6, 2, 2, 2),
    GlyphSegmentSpec::new(6, 6, 2, 2),
    GlyphSegmentSpec::new(6, 10, 2, 2),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_kebab_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if push_icon_asset_pixels(
        commands,
        TREE_MORE_ICON,
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
        &KEBAB_SEGMENTS,
    );
}
