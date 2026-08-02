mod close_prompt;
mod snapshot;
mod template_hover_state;

use crate::ui::retained_host::console_output::console_output_viewport_size;
use crate::ui::retained_host::ui_perf::{UiPerfCounter, record_current_ui_perf_counter};
use zircon_runtime::diagnostic_log::{
    DiagnosticLogLevel, diagnostic_log_allows, write_diagnostic_log,
};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::data::{
    FrameRect, HostMenuStateData, HostPaneInteractionStateData, HostWindowPresentationData,
};
use super::UiHostWindow;

pub(in crate::ui::retained_host::host_contract) use self::snapshot::host_presentation_from_state;

impl UiHostWindow {
    pub(crate) fn set_host_presentation(&self, presentation: HostWindowPresentationData) {
        let mut state = self.state.borrow_mut();
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        if state.presentation_rebuild_count <= 8
            || state.presentation_rebuild_count.is_power_of_two()
        {
            if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
                write_diagnostic_log(
                    "editor_host_window",
                    format!(
                        "set_host_presentation count={} project_path={} viewport_label={} status={} center={} document={} viewport={}",
                        state.presentation_rebuild_count,
                        presentation.host_shell.project_path,
                        presentation.host_shell.viewport_label,
                        presentation.host_shell.status_secondary,
                        frame_summary(&presentation.host_layout.center_band_frame),
                        frame_summary(&presentation.host_layout.document_region_frame),
                        frame_summary(&presentation.host_layout.viewport_content_frame)
                    ),
                );
            }
        }
        state.host_presentation = presentation;
    }

    pub(crate) fn get_host_presentation(&self) -> HostWindowPresentationData {
        let state = self.state.borrow();
        host_presentation_from_state(&state)
    }

    pub(crate) fn get_menu_state(&self) -> HostMenuStateData {
        self.state.borrow().menu_state.clone()
    }

    pub(crate) fn get_pane_interaction_state(&self) -> HostPaneInteractionStateData {
        self.state.borrow().pane_interaction_state.clone()
    }

    pub(crate) fn console_output_viewport_size(
        &self,
        source_window_id: Option<&str>,
    ) -> Option<UiSize> {
        let state = self.state.borrow();
        console_output_viewport_size(&state.host_presentation, source_window_id)
    }
}

fn frame_summary(frame: &FrameRect) -> String {
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        frame.x, frame.y, frame.width, frame.height
    )
}
