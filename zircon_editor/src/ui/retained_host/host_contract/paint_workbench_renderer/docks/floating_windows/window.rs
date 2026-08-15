use super::super::super::super::data::{
    FloatingWindowData, FrameRect, HostPaneInteractionStateData, HostTextInputFocusData,
    HostViewportImageData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::{is_visible_frame, translated};
use super::super::super::super::paint_primitives::{
    draw_rounded_border_clipped, draw_rounded_rect_clipped,
};
use super::super::super::super::paint_template_nodes::draw_template_nodes;
use super::super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};
use super::super::pane;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FloatingWindowPalette {
    shadow: [u8; 4],
    panel: [u8; 4],
    border: [u8; 4],
    header: [u8; 4],
}

fn floating_window_palette(palette: HostMaterialPalette) -> FloatingWindowPalette {
    FloatingWindowPalette {
        shadow: palette.shadow,
        panel: palette.surface,
        border: palette.focus_ring,
        header: palette.popup,
    }
}

pub(super) fn draw_floating_window(
    frame: &mut HostRgbaFrame,
    window: &FloatingWindowData,
    interaction: &HostPaneInteractionStateData,
    viewport_image: Option<&HostViewportImageData>,
    text_input_focus: Option<&HostTextInputFocusData>,
) {
    if !is_visible_frame(&window.frame) {
        return;
    }
    let metrics = current_host_metrics();
    let palette = floating_window_palette(current_host_palette());
    let shadow = floating_window_shadow_frame(&window.frame, metrics);
    draw_rounded_rect_clipped(frame, shadow, None, palette.shadow, metrics.radius_control);
    draw_rounded_rect_clipped(
        frame,
        window.frame.clone(),
        Some(&window.frame),
        palette.panel,
        metrics.radius_control,
    );

    let header = translated(&window.header_frame, window.frame.x, window.frame.y);
    if is_visible_frame(&header) {
        draw_rounded_rect_clipped(
            frame,
            header.clone(),
            Some(&window.frame),
            palette.header,
            metrics.radius_control,
        );
        draw_template_nodes(frame, &window.header_nodes, &window.frame, &header, None);
    }

    let body = floating_window_body_frame(&window.frame, &header, metrics.border_width);
    pane::draw_pane(
        frame,
        &window.active_pane,
        &body,
        interaction,
        viewport_image,
        text_input_focus,
    );
    draw_rounded_border_clipped(
        frame,
        window.frame.clone(),
        Some(&window.frame),
        palette.border,
        metrics.border_width,
        metrics.radius_control,
    );
}

pub(super) fn floating_window_paint_bounds(window: &FrameRect) -> FrameRect {
    let shadow = floating_window_shadow_frame(window, current_host_metrics());
    let right = window.right().max(shadow.right());
    let bottom = window.bottom().max(shadow.bottom());
    FrameRect {
        x: window.x.min(shadow.x),
        y: window.y.min(shadow.y),
        width: (right - window.x.min(shadow.x)).max(0.0),
        height: (bottom - window.y.min(shadow.y)).max(0.0),
    }
}

fn floating_window_shadow_frame(window: &FrameRect, metrics: HostControlMetrics) -> FrameRect {
    FrameRect {
        x: window.x + metrics.gap_s,
        y: window.y + metrics.gap_s + metrics.border_width,
        width: window.width,
        height: window.height,
    }
}

fn floating_window_body_frame(
    window: &FrameRect,
    header: &FrameRect,
    border_width: f32,
) -> FrameRect {
    let inset = border_width.max(0.0).min(window.width.max(0.0) * 0.5);
    let window_bottom = window.y + window.height.max(0.0);
    let body_top =
        (header.y.max(window.y) + header.height.max(0.0) + inset).clamp(window.y, window_bottom);
    let body_bottom = (window_bottom - inset).max(body_top);
    FrameRect {
        x: window.x + inset,
        y: body_top,
        width: (window.width - inset * 2.0).max(0.0),
        height: (body_bottom - body_top).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

    #[test]
    fn floating_window_palette_projects_all_window_roles_from_the_current_theme() {
        let mut palette = PALETTE;
        palette.shadow = [1, 2, 3, 255];
        palette.surface = [4, 5, 6, 255];
        palette.focus_ring = [7, 8, 9, 255];
        palette.popup = [10, 11, 12, 255];

        assert_eq!(
            floating_window_palette(palette),
            FloatingWindowPalette {
                shadow: [1, 2, 3, 255],
                panel: [4, 5, 6, 255],
                border: [7, 8, 9, 255],
                header: [10, 11, 12, 255],
            }
        );
    }

    #[test]
    fn floating_window_shadow_uses_the_shared_slate_gap_and_border_metrics() {
        let shadow = floating_window_shadow_frame(
            &FrameRect {
                x: 20.0,
                y: 30.0,
                width: 200.0,
                height: 120.0,
            },
            METRICS,
        );

        assert_eq!(shadow.x, 24.0);
        assert_eq!(shadow.y, 35.0);
        assert_eq!(shadow.width, 200.0);
        assert_eq!(shadow.height, 120.0);
    }

    #[test]
    fn floating_window_paint_bounds_include_the_shadow_only_region() {
        let window = FrameRect {
            x: 20.0,
            y: 30.0,
            width: 200.0,
            height: 120.0,
        };

        let bounds = floating_window_paint_bounds(&window);

        assert_eq!(bounds.x, window.x);
        assert_eq!(bounds.y, window.y);
        assert!(bounds.right() > window.right());
        assert!(bounds.bottom() > window.bottom());
    }

    #[test]
    fn floating_window_body_frame_stays_inside_the_border_and_below_the_header() {
        let body = floating_window_body_frame(
            &FrameRect {
                x: 20.0,
                y: 30.0,
                width: 200.0,
                height: 120.0,
            },
            &FrameRect {
                x: 20.0,
                y: 30.0,
                width: 200.0,
                height: 28.0,
            },
            1.0,
        );

        assert_eq!(body.x, 21.0);
        assert_eq!(body.y, 59.0);
        assert_eq!(body.width, 198.0);
        assert_eq!(body.height, 90.0);
    }
}
