use super::super::super::data::FrameRect;
use super::super::super::paint_diagnostics::debug_refresh_overlay_frame;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect, draw_rect_clipped, draw_text_bars, draw_text_bars_clipped,
};
use super::super::{ACCENT, MUTED_TEXT};

pub(super) fn draw_project_marker(
    frame: &mut HostRgbaFrame,
    project_path: &str,
    top_bar_height: f32,
) {
    draw_rect(
        frame,
        FrameRect {
            x: 12.0,
            y: (top_bar_height * 0.5 - 6.0).max(4.0),
            width: 18.0,
            height: 12.0,
        },
        ACCENT,
    );
    draw_text_bars(
        frame,
        40.0,
        (top_bar_height * 0.5 - 5.0).max(5.0),
        project_path,
        MUTED_TEXT,
    );
}

pub(super) fn draw_debug_refresh_rate_marker(
    frame: &mut HostRgbaFrame,
    top_bar: &FrameRect,
    label: &str,
) {
    let Some(marker) = debug_refresh_overlay_frame(top_bar, label) else {
        return;
    };
    draw_rect_clipped(frame, marker.clone(), Some(top_bar), [18, 24, 34, 210]);
    draw_border_clipped(frame, marker.clone(), Some(top_bar), ACCENT);
    draw_text_bars_clipped(
        frame,
        marker.x + 7.0,
        marker.y + 5.0,
        label,
        Some(&marker),
        ACCENT,
    );
}
