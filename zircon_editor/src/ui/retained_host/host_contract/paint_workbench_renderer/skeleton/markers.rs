use super::super::super::data::FrameRect;
use super::super::super::paint_diagnostics::debug_refresh_overlay_frame;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{draw_border_clipped, draw_rect, draw_rect_clipped};
use super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::paint_theme::HostControlMetrics;
use super::RootSkeletonPalette;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn draw_project_marker(
    frame: &mut HostRgbaFrame,
    project_path: &str,
    top_bar: &FrameRect,
    palette: RootSkeletonPalette,
    metrics: HostControlMetrics,
) {
    let icon_width = metrics.button_icon_gap + metrics.gap_l - metrics.border_width;
    let icon_height = (metrics.font_small + metrics.gap_s).min(top_bar.height.max(1.0));
    draw_rect(
        frame,
        FrameRect {
            x: top_bar.x + metrics.gap_l,
            y: top_bar.y + ((top_bar.height - icon_height).max(0.0) * 0.5),
            width: icon_width,
            height: icon_height,
        },
        palette.accent,
    );
    let left = top_bar.x + metrics.gap_l + icon_width + metrics.gap_m;
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil())
        .min(top_bar.height.max(1.0));
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: left,
            y: top_bar.y + ((top_bar.height - line_height).max(0.0) * 0.5),
            width: (top_bar.x + top_bar.width - left - metrics.gap_l).max(1.0),
            height: line_height,
        },
        project_path,
        Some(top_bar),
        palette.text_muted,
        metrics.font_body,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

pub(super) fn draw_debug_refresh_rate_marker(
    frame: &mut HostRgbaFrame,
    top_bar: &FrameRect,
    label: &str,
    palette: RootSkeletonPalette,
    metrics: HostControlMetrics,
) {
    let Some(marker) = debug_refresh_overlay_frame(top_bar, label) else {
        return;
    };
    draw_rect_clipped(frame, marker.clone(), Some(top_bar), palette.marker_surface);
    draw_border_clipped(frame, marker.clone(), Some(top_bar), palette.accent);
    let line_height = metrics
        .line_height(metrics.font_small)
        .round()
        .max(metrics.font_small.ceil())
        .min(marker.height.max(1.0));
    let inset = (metrics.gap_m - metrics.border_width).max(0.0);
    draw_text_with_size_and_style(
        frame,
        FrameRect {
            x: marker.x + inset,
            y: marker.y + ((marker.height - line_height).max(0.0) * 0.5),
            width: (marker.width - inset * 2.0).max(1.0),
            height: line_height,
        },
        label,
        Some(&marker),
        palette.accent,
        metrics.font_small,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

#[cfg(test)]
#[path = "tests/markers.rs"]
mod tests;
