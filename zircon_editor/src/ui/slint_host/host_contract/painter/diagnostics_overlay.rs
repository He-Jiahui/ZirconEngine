use super::super::data::{FrameRect, HostWindowLayoutData, HostWindowPresentationData};
use super::geometry::is_visible_frame;

const MARKER_HORIZONTAL_PADDING: f32 = 14.0;
const MARKER_RIGHT_INSET: f32 = 8.0;
const MARKER_TOP_INSET: f32 = 6.0;
const MARKER_VERTICAL_INSET: f32 = 12.0;
const MARKER_MIN_HEIGHT: f32 = 14.0;
const APPROX_GLYPH_WIDTH: f32 = 8.0;

pub(in crate::ui::slint_host::host_contract) fn debug_refresh_overlay_frame(
    top_bar: &FrameRect,
    label: &str,
) -> Option<FrameRect> {
    if label.trim().is_empty() || !is_visible_frame(top_bar) {
        return None;
    }
    let marker_width = (label.chars().count() as f32 * APPROX_GLYPH_WIDTH
        + MARKER_HORIZONTAL_PADDING)
        .min((top_bar.width - MARKER_VERTICAL_INSET).max(1.0))
        .max(1.0);
    Some(FrameRect {
        x: (top_bar.x + top_bar.width - marker_width - MARKER_RIGHT_INSET).max(top_bar.x),
        y: top_bar.y + MARKER_TOP_INSET,
        width: marker_width,
        height: (top_bar.height - MARKER_VERTICAL_INSET).max(MARKER_MIN_HEIGHT),
    })
}

pub(in crate::ui::slint_host::host_contract) fn presentation_top_bar_frame(
    width: u32,
    height: u32,
    presentation: &HostWindowPresentationData,
) -> FrameRect {
    let layout = if has_visible_root_frame(&presentation.host_scene_data.layout) {
        &presentation.host_scene_data.layout
    } else {
        &presentation.host_layout
    };
    let top_bar_height =
        if layout.center_band_frame.y.is_finite() && layout.center_band_frame.y > 1.0 {
            layout.center_band_frame.y
        } else {
            fallback_top_bar_height(height)
        };
    top_bar_frame_with_height(width, top_bar_height)
}

fn top_bar_frame_with_height(width: u32, height: f32) -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height,
    }
}

fn fallback_top_bar_height(height: u32) -> f32 {
    38.0_f32.min(height as f32 * 0.25)
}

fn has_visible_root_frame(layout: &HostWindowLayoutData) -> bool {
    is_visible_frame(&layout.center_band_frame)
        || is_visible_frame(&layout.status_bar_frame)
        || is_visible_frame(&layout.document_region_frame)
        || is_visible_frame(&layout.viewport_content_frame)
}

pub(in crate::ui::slint_host::host_contract) fn union_frames(
    left: &FrameRect,
    right: &FrameRect,
) -> FrameRect {
    let x0 = left.x.min(right.x);
    let y0 = left.y.min(right.y);
    let x1 = (left.x + left.width).max(right.x + right.width);
    let y1 = (left.y + left.height).max(right.y + right.height);
    FrameRect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(0.0),
        height: (y1 - y0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_refresh_overlay_frame_uses_top_right_marker_geometry() {
        let top_bar = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 38.0,
        };

        let frame = debug_refresh_overlay_frame(&top_bar, "FPS 60").unwrap();

        assert_eq!(frame.y, 6.0);
        assert_eq!(frame.height, 26.0);
        assert!(frame.x > 0.0);
        assert!(frame.x + frame.width <= top_bar.width);
    }

    #[test]
    fn presentation_top_bar_frame_uses_scene_layout_height_before_fallback() {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.layout.center_band_frame = FrameRect {
            x: 0.0,
            y: 58.0,
            width: 200.0,
            height: 100.0,
        };

        let frame = presentation_top_bar_frame(200, 120, &presentation);

        assert_eq!(frame.height, 58.0);
    }

    #[test]
    fn presentation_top_bar_frame_falls_back_for_empty_layout() {
        let frame = presentation_top_bar_frame(200, 120, &HostWindowPresentationData::default());

        assert_eq!(frame.height, 30.0);
    }
}
