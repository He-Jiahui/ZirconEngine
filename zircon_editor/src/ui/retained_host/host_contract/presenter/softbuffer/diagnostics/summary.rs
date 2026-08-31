use std::fmt::{self, Write as _};

use super::super::super::super::data::{FrameRect, HostWindowPresentationData};

const PRESENTATION_SUMMARY_BASE_CAPACITY: usize = 2_048;

struct FrameSummary<'a>(&'a FrameRect);

impl fmt::Display for FrameSummary<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let frame = self.0;
        write!(
            formatter,
            "{:.1},{:.1},{:.1},{:.1}",
            frame.x, frame.y, frame.width, frame.height
        )
    }
}

pub(in crate::ui::retained_host::host_contract) fn presentation_summary(
    presentation: &HostWindowPresentationData,
) -> String {
    let layout = &presentation.host_layout;
    let scene = &presentation.host_scene_data;
    let dynamic_values = [
        presentation.host_shell.project_path.as_str(),
        presentation.host_shell.viewport_label.as_str(),
        presentation.host_shell.status_secondary.as_str(),
        scene.document_dock.pane.kind.as_str(),
        scene.left_dock.pane.kind.as_str(),
        scene.right_dock.pane.kind.as_str(),
        scene.bottom_dock.pane.kind.as_str(),
    ];
    let capacity = dynamic_values
        .iter()
        .fold(PRESENTATION_SUMMARY_BASE_CAPACITY, |capacity, value| {
            capacity.saturating_add(value.len())
        });
    let mut summary = String::with_capacity(capacity);
    write!(
        &mut summary,
        "project_path={} viewport_label={} status={} center={} status_bar={} document={} viewport={} left={} right={} bottom={} page_tabs={} document_tabs={} left_tabs={} right_tabs={} bottom_tabs={} floating_windows={} document_pane_kind={} left_pane_kind={} right_pane_kind={} bottom_pane_kind={}",
        presentation.host_shell.project_path,
        presentation.host_shell.viewport_label,
        presentation.host_shell.status_secondary,
        FrameSummary(&layout.center_band_frame),
        FrameSummary(&layout.status_bar_frame),
        FrameSummary(&layout.document_region_frame),
        FrameSummary(&layout.viewport_content_frame),
        FrameSummary(&layout.left_region_frame),
        FrameSummary(&layout.right_region_frame),
        FrameSummary(&layout.bottom_region_frame),
        scene.page_chrome.tabs.row_count(),
        scene.document_dock.tabs.row_count(),
        scene.left_dock.tabs.row_count(),
        scene.right_dock.tabs.row_count(),
        scene.bottom_dock.tabs.row_count(),
        scene.floating_layer.floating_windows.row_count(),
        scene.document_dock.pane.kind,
        scene.left_dock.pane.kind,
        scene.right_dock.pane.kind,
        scene.bottom_dock.pane.kind,
    )
    .expect("writing to a String cannot fail");
    summary
}

pub(in crate::ui::retained_host::host_contract) fn frame_summary(frame: &FrameRect) -> String {
    FrameSummary(frame).to_string()
}

#[cfg(test)]
#[path = "summary/single_buffer_presentation_summary_tests.rs"]
mod single_buffer_presentation_summary_tests;
