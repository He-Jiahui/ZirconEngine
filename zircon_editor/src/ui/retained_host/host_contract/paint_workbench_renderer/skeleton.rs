mod markers;

use self::markers::{draw_debug_refresh_rate_marker, draw_project_marker};
use super::super::data::HostWindowPresentationData;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_primitives::{
    draw_border, draw_label_marker, draw_rect, draw_separator_line,
};
use super::root_frames::RootFrames;
use super::{
    ACCENT, CENTER_BAND, DOCUMENT_PANEL, MUTED_TEXT, SEPARATOR, SIDE_PANEL, STATUS_BAR, TOP_BAR,
    VIEWPORT_PANEL,
};

pub(in crate::ui::retained_host::host_contract) fn draw_root_skeleton(
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
