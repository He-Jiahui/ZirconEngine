use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::{draw_border_clipped, draw_rect_clipped};
use super::super::super::paint_text::{draw_text_with_size_and_style, measure_runtime_text_width};
use super::super::super::paint_theme::{
    HostControlMetrics, HostMaterialPalette, current_host_metrics, current_host_palette,
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

pub(in crate::ui::retained_host::host_contract) fn draw_viewport_toolbar(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    toolbar: &FrameRect,
    clip: &FrameRect,
) {
    if !is_visible_frame(toolbar) {
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
    labels: [&str; 4],
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
    labels: [&str; 4],
    metrics: HostControlMetrics,
) -> [FrameRect; 4] {
    let outer_inset =
        (metrics.gap_m + metrics.border_width * 2.0).min(toolbar.width.max(0.0) * 0.5);
    let content_width = (toolbar.width - outer_inset * 2.0).max(0.0);
    let gap = metrics.gap_s.min(content_width / 3.0);
    let label_padding = metrics.gap_m;
    let preferred_widths = labels.map(|label| {
        (measure_runtime_text_width(label, metrics.font_body) + label_padding * 2.0).max(0.0)
    });
    let preferred_total = preferred_widths.iter().sum::<f32>() + gap * 3.0;
    let compact_width = ((content_width - gap * 3.0).max(0.0)) / 4.0;
    let slot_widths = if preferred_total <= content_width {
        preferred_widths
    } else {
        [compact_width; 4]
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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
    use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

    #[test]
    fn toolbar_palette_uses_runtime_surface_border_and_text_roles() {
        let mut palette = PALETTE;
        palette.surface = [1, 2, 3, 255];
        palette.border = [4, 5, 6, 255];
        palette.text_muted = [7, 8, 9, 255];

        assert_eq!(
            viewport_toolbar_palette(palette),
            ViewportToolbarPalette {
                surface: [1, 2, 3, 255],
                border: [4, 5, 6, 255],
                text: [7, 8, 9, 255],
            }
        );
    }

    #[test]
    fn scene_mode_protocol_symbols_project_to_user_facing_labels() {
        assert_eq!(scene_mode_label("Select"), "Select");
        assert_eq!(scene_mode_label("Transform.Rotate"), "Rotate");
        assert_eq!(scene_mode_label("Custom:terrain.paint"), "terrain.paint");
    }

    #[test]
    fn toolbar_slots_follow_natural_text_widths_when_space_allows() {
        let toolbar = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 640.0,
            height: 30.0,
        };
        let slots =
            viewport_toolbar_label_slots(&toolbar, ["Move", "World", "Lit", "Grid"], METRICS);

        assert!(slots[0].width > 0.0);
        assert!(slots[0].width < slots[1].width);
        assert!(
            slots
                .windows(2)
                .all(|pair| pair[0].x + pair[0].width <= pair[1].x)
        );
        assert!(slots[3].x + slots[3].width <= toolbar.x + toolbar.width);
    }

    #[test]
    fn toolbar_slots_compact_evenly_inside_a_narrow_toolbar() {
        let toolbar = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 30.0,
        };
        let slots = viewport_toolbar_label_slots(
            &toolbar,
            [
                "Translate Long Tool Name",
                "World Coordinates",
                "Lit With Shadows",
                "Visible And Snap",
            ],
            METRICS,
        );

        assert!(
            slots
                .windows(2)
                .all(|pair| pair[0].x + pair[0].width <= pair[1].x)
        );
        assert!((slots[0].width - slots[1].width).abs() < f32::EPSILON);
        assert!(slots[3].x + slots[3].width <= toolbar.x + toolbar.width);
    }

    #[test]
    fn toolbar_labels_use_finite_runtime_text_slots_with_ellipsis() {
        let toolbar = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 120.0,
            height: 30.0,
        };
        let mut frame = HostRgbaFrame::recording_only(128, 40);

        draw_viewport_toolbar_labels(
            &mut frame,
            [
                "Translate Long Tool Name",
                "World Coordinates",
                "Lit With Shadows",
                "Visible And Snap",
            ],
            &toolbar,
            &toolbar,
            viewport_toolbar_palette(PALETTE),
            METRICS,
        );

        let texts = frame
            .into_recorded_commands()
            .into_iter()
            .filter_map(|command| match command.kind {
                HostRecordedPaintKind::Text { text, .. } => Some((text, command.frame)),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(texts.len(), 4);
        assert!(texts.iter().all(|(text, _)| text.ends_with('\u{2026}')));
        assert!(texts.iter().all(|(_, frame)| {
            frame.x >= toolbar.x && frame.x + frame.width <= toolbar.x + toolbar.width
        }));
    }
}
