use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::metrics::{dropdown_chevron_right, dropdown_chevron_size};
use super::segments::{push_segments, DropdownGlyphSegmentSpec};

const DROPDOWN_CHEVRON_ICON: &str = "dropdown";

const DROPDOWN_CHEVRON_SEGMENTS: &[DropdownGlyphSegmentSpec] = &[
    DropdownGlyphSegmentSpec::new(3, 5, 2, 2),
    DropdownGlyphSegmentSpec::new(5, 7, 2, 2),
    DropdownGlyphSegmentSpec::new(7, 5, 2, 2),
];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dropdown_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
) {
    let chevron_size = dropdown_chevron_size();
    let chevron = FrameRect {
        x: rect.x + rect.width - dropdown_chevron_right() - chevron_size,
        y: rect.y + (rect.height - chevron_size).max(0.0) * 0.5,
        width: chevron_size,
        height: chevron_size,
    };
    if push_icon_asset_pixels(
        commands,
        DROPDOWN_CHEVRON_ICON,
        &chevron,
        clip,
        order,
        Some(style.chevron),
        opacity,
    ) {
        return;
    }

    push_segments(
        commands,
        &chevron,
        clip,
        order,
        style.chevron,
        opacity,
        DROPDOWN_CHEVRON_SEGMENTS,
    );
}
