use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::segments::{GlyphSegmentSpec, TREE_DISCLOSURE_GLYPH_GRID_UNITS, push_segments};

const TREE_DISCLOSURE_DOWN_ICON: &str = "zircon_editor_shell/toolbar/dropdown.svg";
const TREE_DISCLOSURE_RIGHT_ICON: &str = "zircon_editor_shell/toolbar/chevron-right.svg";
const DOWN_CHEVRON_SEGMENTS: [GlyphSegmentSpec; 3] = [
    GlyphSegmentSpec::new(3, 4, 2, 2),
    GlyphSegmentSpec::new(5, 6, 2, 2),
    GlyphSegmentSpec::new(7, 4, 2, 2),
];
const RIGHT_CHEVRON_SEGMENTS: [GlyphSegmentSpec; 3] = [
    GlyphSegmentSpec::new(4, 3, 2, 3),
    GlyphSegmentSpec::new(6, 6, 2, 2),
    GlyphSegmentSpec::new(4, 8, 2, 3),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_disclosure_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if node.expanded {
        if push_icon_asset_pixels(
            commands,
            TREE_DISCLOSURE_DOWN_ICON,
            rect,
            clip,
            order,
            Some(color),
            opacity,
        ) {
            return;
        }
        push_down_chevron(commands, rect, clip, order, color, opacity);
    } else {
        if push_icon_asset_pixels(
            commands,
            TREE_DISCLOSURE_RIGHT_ICON,
            rect,
            clip,
            order,
            Some(color),
            opacity,
        ) {
            return;
        }
        push_right_chevron(commands, rect, clip, order, color, opacity);
    }
}

fn push_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        TREE_DISCLOSURE_GLYPH_GRID_UNITS,
        clip,
        order,
        color,
        opacity,
        &DOWN_CHEVRON_SEGMENTS,
    );
}

fn push_right_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        TREE_DISCLOSURE_GLYPH_GRID_UNITS,
        clip,
        order,
        color,
        opacity,
        &RIGHT_CHEVRON_SEGMENTS,
    );
}
