use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::paint_diagnostics::debug_refresh_overlay_frame;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{
    draw_border, draw_border_clipped, draw_label_marker, draw_rect, draw_rect_clipped,
    draw_separator_line, draw_text_bars, draw_text_bars_clipped,
};
use super::root_frames::RootFrames;
use super::{
    ACCENT, CENTER_BAND, DOCUMENT_PANEL, MUTED_TEXT, SEPARATOR, SIDE_PANEL, STATUS_BAR, TOP_BAR,
    VIEWPORT_PANEL,
};

pub(super) fn draw_root_skeleton(
    frame: &mut HostRgbaFrame,
    root: &RootFrames,
    presentation: &HostWindowPresentationData,
) {
    draw_rect(frame, root.top_bar.clone(), TOP_BAR);
    draw_rect(frame, root.center_band.clone(), CENTER_BAND);
    draw_rect(frame, root.left_region.clone(), SIDE_PANEL);
    draw_rect(frame, root.right_region.clone(), SIDE_PANEL);
    draw_rect(frame, root.document_region.clone(), DOCUMENT_PANEL);
    draw_rect(frame, root.bottom_region.clone(), SIDE_PANEL);
    draw_rect(frame, root.viewport_region.clone(), VIEWPORT_PANEL);
    draw_rect(frame, root.status_bar.clone(), STATUS_BAR);

    draw_border(frame, root.left_region.clone(), SEPARATOR);
    draw_border(frame, root.right_region.clone(), SEPARATOR);
    draw_border(frame, root.document_region.clone(), SEPARATOR);
    draw_border(frame, root.bottom_region.clone(), SEPARATOR);
    draw_border(frame, root.viewport_region.clone(), ACCENT);
    draw_separator_line(
        frame,
        0,
        root.top_bar.height.round() as u32,
        frame.width(),
        SEPARATOR,
    );

    draw_project_marker(
        frame,
        &presentation.host_shell.project_path,
        root.top_bar.height,
    );
    draw_debug_refresh_rate_marker(
        frame,
        &root.top_bar,
        &presentation.host_shell.debug_refresh_rate,
    );
    draw_label_marker(
        frame,
        &root.viewport_region,
        &presentation.host_shell.viewport_label,
        ACCENT,
    );
    draw_label_marker(
        frame,
        &root.status_bar,
        &presentation.host_shell.status_secondary,
        MUTED_TEXT,
    );
}

fn draw_project_marker(frame: &mut HostRgbaFrame, project_path: &str, top_bar_height: f32) {
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

fn draw_debug_refresh_rate_marker(frame: &mut HostRgbaFrame, top_bar: &FrameRect, label: &str) {
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
