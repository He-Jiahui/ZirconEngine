use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::metrics::{chip_glyph_chevron_right, chip_glyph_chevron_size};

const CHIP_CHEVRON_ICON: &str = "zircon_editor_shell/toolbar/chevron-right.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let Some(chevron) = chip_chevron_rect(rect) else {
        return;
    };
    if intersect(&chevron, clip).is_none() {
        return;
    }
    push_icon_asset_pixels(
        commands,
        CHIP_CHEVRON_ICON,
        &chevron,
        clip,
        order,
        Some(color),
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_can_paint_chevron(
    rect: &FrameRect,
) -> bool {
    chip_chevron_rect(rect).is_some()
}

fn chip_chevron_rect(rect: &FrameRect) -> Option<FrameRect> {
    if !rect.x.is_finite()
        || !rect.y.is_finite()
        || !rect.width.is_finite()
        || !rect.height.is_finite()
        || rect.width <= 0.0
        || rect.height <= 0.0
    {
        return None;
    }
    let chevron_size = chip_glyph_chevron_size();
    let chevron_right = chip_glyph_chevron_right();
    if !chevron_size.is_finite()
        || !chevron_right.is_finite()
        || chevron_size <= 0.0
        || chevron_right < 0.0
    {
        return None;
    }
    let chevron = FrameRect {
        x: rect.x + rect.width - chevron_right - chevron_size,
        y: rect.y + (rect.height - chevron_size).max(0.0) * 0.5,
        width: chevron_size,
        height: chevron_size,
    };
    if chevron.x >= rect.x
        && chevron.y >= rect.y
        && chevron.x + chevron.width <= rect.x + rect.width
        && chevron.y + chevron.height <= rect.y + rect.height
    {
        Some(chevron)
    } else {
        None
    }
}
