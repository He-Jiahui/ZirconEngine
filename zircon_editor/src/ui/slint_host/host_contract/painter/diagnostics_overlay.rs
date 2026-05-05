use super::super::data::FrameRect;
use super::geometry::is_visible_frame;

const MARKER_HORIZONTAL_PADDING: f32 = 14.0;
const MARKER_RIGHT_INSET: f32 = 8.0;
const MARKER_TOP_INSET: f32 = 6.0;
const MARKER_VERTICAL_INSET: f32 = 12.0;
const MARKER_MIN_HEIGHT: f32 = 14.0;
const APPROX_GLYPH_WIDTH: f32 = 8.0;

pub(super) fn debug_refresh_overlay_frame(top_bar: &FrameRect, label: &str) -> Option<FrameRect> {
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

pub(super) fn top_bar_frame(width: u32, height: u32) -> FrameRect {
    let top_bar_height = 38.0_f32.min(height as f32 * 0.25);
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: top_bar_height,
    }
}

pub(super) fn union_frames(left: &FrameRect, right: &FrameRect) -> FrameRect {
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
}
