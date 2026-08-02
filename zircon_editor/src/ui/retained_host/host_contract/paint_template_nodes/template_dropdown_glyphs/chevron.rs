use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchDropdownStyle;
use super::super::template_dropdown_metrics::WorkbenchDropdownMetrics;
use super::super::template_icon_assets::push_icon_asset_pixels;
use super::segments::{DropdownGlyphSegmentSpec, push_segments};

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
    metrics: &WorkbenchDropdownMetrics,
) {
    if !has_paintable_rect(rect)
        || !metrics.chevron_size.is_finite()
        || !metrics.chevron_right.is_finite()
        || metrics.chevron_size <= 0.0
        || metrics.chevron_right < 0.0
    {
        return;
    }
    let size = metrics.chevron_size.min(rect.width).min(rect.height);
    if size <= 0.0 {
        return;
    }
    let right_inset = metrics.chevron_right.min((rect.width - size).max(0.0));
    let chevron = FrameRect {
        x: rect.x + rect.width - right_inset - size,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
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

fn has_paintable_rect(rect: &FrameRect) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}
