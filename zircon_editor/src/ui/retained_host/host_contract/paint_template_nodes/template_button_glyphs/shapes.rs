use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::identity::ButtonGlyph;
use super::segments;

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
        ButtonGlyph::Plus => segments::push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[(6.0, 2.0, 2.0, 10.0), (2.0, 6.0, 10.0, 2.0)],
        ),
        ButtonGlyph::Trash => segments::push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[
                (3.0, 4.0, 8.0, 1.2),
                (4.0, 2.0, 6.0, 1.2),
                (4.0, 5.0, 1.2, 7.0),
                (9.0, 5.0, 1.2, 7.0),
                (5.0, 12.0, 4.0, 1.2),
            ],
        ),
        ButtonGlyph::ChevronDown => segments::push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[
                (3.0, 5.0, 2.0, 2.0),
                (5.0, 7.0, 4.0, 2.0),
                (9.0, 5.0, 2.0, 2.0),
            ],
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
