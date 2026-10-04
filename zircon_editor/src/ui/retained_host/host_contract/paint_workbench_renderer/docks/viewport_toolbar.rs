use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::super::super::paint_text::{draw_text_with_size_and_style, measure_runtime_text_width};
use super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ViewportToolbarPalette {
    surface: [u8; 4],
    border: [u8; 4],
    text: [u8; 4],
}

fn viewport_toolbar_palette(palette: HostMaterialPalette) -> ViewportToolbarPalette {
    ViewportToolbarPalette {
        surface: palette.surface,
        border: palette.border,
        text: palette.text_muted,
    }
}

// Scene/Game pane 的视口栏读取当前主题与文字度量，标签槽受可用宽度约束并交由统一文字绘制器截断。
pub(in crate::ui::retained_host::host_contract) fn draw_viewport_toolbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    toolbar: &FrameRect,
    clip: &FrameRect,
) {
    if !is_visible_frame(toolbar) {
        return;
    }
    if pane.viewport.toolbar_template_nodes.row_count() > 0 {
        let nodes = super::super::super::viewport_chrome_state::live_toolbar_nodes(
            &pane.viewport,
            frame.pane_interaction_state(),
        );
        super::super::super::paint_template_nodes::draw_template_nodes(
            frame, &nodes, toolbar, clip, None,
        );
        return;
    }
    let metrics = current_host_metrics();
    let palette = viewport_toolbar_palette(current_host_palette());
    draw_rect_clipped(frame, toolbar.clone(), Some(clip), palette.surface);
    draw_border_clipped(frame, toolbar.clone(), Some(clip), palette.border);
    draw_viewport_toolbar_labels(
        frame,
        [
            scene_mode_label(pane.viewport.mode.as_str()),
            pane.viewport.transform_space.as_str(),
            pane.viewport.pivot_mode.as_str(),
            pane.viewport.display_mode.as_str(),
            pane.viewport.grid_mode.as_str(),
        ],
        toolbar,
        clip,
        palette,
        metrics,
    );
}

fn scene_mode_label(mode: &str) -> &str {
    mode.strip_prefix("Transform.")
        .or_else(|| mode.strip_prefix("Custom:"))
        .unwrap_or(mode)
}

fn draw_viewport_toolbar_labels(
    frame: &mut HostRgbaFrame,
    labels: [&str; 5],
    toolbar: &FrameRect,
    clip: &FrameRect,
    palette: ViewportToolbarPalette,
    metrics: HostControlMetrics,
) {
    let slots = viewport_toolbar_label_slots(toolbar, labels, metrics);
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());
    for (label, slot) in labels.into_iter().zip(slots) {
        draw_text_with_size_and_style(
            frame,
            slot,
            label,
            Some(clip),
            palette.text,
            metrics.font_body,
            line_height,
            UiTextRunPaintStyle::default(),
        );
    }
}

fn viewport_toolbar_label_slots(
    toolbar: &FrameRect,
    labels: [&str; 5],
    metrics: HostControlMetrics,
) -> [FrameRect; 5] {
    let outer_inset =
        (metrics.gap_m + metrics.border_width * 2.0).min(toolbar.width.max(0.0) * 0.5);
    let content_width = (toolbar.width - outer_inset * 2.0).max(0.0);
    let gap = metrics.gap_s.min(content_width / 4.0);
    let label_padding = metrics.gap_m;
    let preferred_widths = labels.map(|label| {
        (measure_runtime_text_width(label, metrics.font_body) + label_padding * 2.0).max(0.0)
    });
    let preferred_total = preferred_widths.iter().sum::<f32>() + gap * 4.0;
    let compact_width = ((content_width - gap * 4.0).max(0.0)) / 5.0;
    let slot_widths = if preferred_total <= content_width {
        preferred_widths
    } else {
        [compact_width; 5]
    };
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil())
        .min(toolbar.height.max(1.0));
    let mut x = toolbar.x + outer_inset;

    std::array::from_fn(|index| {
        let slot = FrameRect {
            x,
            y: toolbar.y + ((toolbar.height - line_height).max(0.0) * 0.5),
            width: slot_widths[index],
            height: line_height,
        };
        x += slot_widths[index] + gap;
        slot
    })
}

#[cfg(test)]
#[path = "tests/viewport_toolbar.rs"]
mod tests;
