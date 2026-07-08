use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::identity::ButtonGlyph;
use super::segments::{self, ButtonGlyphSegmentSpec};

const PLUS_SEGMENTS: &[ButtonGlyphSegmentSpec] = &[
    ButtonGlyphSegmentSpec::new(30, 10, 10, 50),
    ButtonGlyphSegmentSpec::new(10, 30, 50, 10),
];
const TRASH_SEGMENTS: &[ButtonGlyphSegmentSpec] = &[
    ButtonGlyphSegmentSpec::new(15, 20, 40, 6),
    ButtonGlyphSegmentSpec::new(20, 10, 30, 6),
    ButtonGlyphSegmentSpec::new(20, 25, 6, 35),
    ButtonGlyphSegmentSpec::new(45, 25, 6, 35),
    ButtonGlyphSegmentSpec::new(25, 60, 20, 6),
];
const CHEVRON_DOWN_SEGMENTS: &[ButtonGlyphSegmentSpec] = &[
    ButtonGlyphSegmentSpec::new(15, 25, 10, 10),
    ButtonGlyphSegmentSpec::new(25, 35, 20, 10),
    ButtonGlyphSegmentSpec::new(45, 25, 10, 10),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    glyph: ButtonGlyph,
    color: [u8; 4],
    opacity: f32,
) {
    if let Some(asset_name) = button_glyph_asset_name(glyph) {
        if push_icon_asset_pixels(
            commands,
            asset_name,
            rect,
            clip,
            order,
            Some(color),
            opacity,
        ) {
            return;
        }
    }

    match glyph {
        ButtonGlyph::Plus => {
            segments::push_segments(commands, rect, clip, order, color, opacity, PLUS_SEGMENTS)
        }
        ButtonGlyph::Trash => {
            segments::push_segments(commands, rect, clip, order, color, opacity, TRASH_SEGMENTS)
        }
        ButtonGlyph::ChevronDown => segments::push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            CHEVRON_DOWN_SEGMENTS,
        ),
        ButtonGlyph::None => {}
    }
}

fn button_glyph_asset_name(glyph: ButtonGlyph) -> Option<&'static str> {
    match glyph {
        ButtonGlyph::Plus => Some("add"),
        ButtonGlyph::Trash => Some("trash"),
        ButtonGlyph::ChevronDown => Some("dropdown"),
        ButtonGlyph::None => None,
    }
}
