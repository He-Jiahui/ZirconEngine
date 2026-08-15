mod close_prompt;
mod snapshot;
mod template_hover_state;

use crate::ui::retained_host::console_output::console_output_viewport_size;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::data::SceneViewportChromeData;
use super::super::data::{
    FrameRect, HostDockPresentationPatch, HostMenuStateData, HostPaneInteractionStateData,
    HostPresentationGeneration, HostWindowLayoutData, HostWindowPresentationData,
    HostWindowShellData, TemplatePaneNodeData,
};
use super::UiHostWindow;
use crate::ui::retained_host::primitives::ModelRc;

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
        state.replace_host_presentation(presentation);
    }

    pub(crate) fn get_host_presentation(&self) -> HostWindowPresentationData {
        let state = self.state.borrow();
        host_presentation_from_state(&state)
    }

    pub(crate) fn update_host_presentation<R>(
        &self,
        update: impl FnOnce(&mut HostWindowPresentationData) -> R,
    ) -> R {
        let mut state = self.state.borrow_mut();
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        state.update_host_presentation(update)
    }

    pub(crate) fn update_host_presentation_if<R>(
        &self,
        predicate: impl FnOnce(&HostWindowPresentationData) -> bool,
        update: impl FnOnce(&mut HostWindowPresentationData) -> R,
    ) -> Option<R> {
        let mut state = self.state.borrow_mut();
        if !predicate(state.host_presentation.as_ref()) {
            return None;
        }
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        Some(state.update_host_presentation(update))
    }

    pub(crate) fn patch_workbench_window_nodes(
        &self,
        nodes: ModelRc<TemplatePaneNodeData>,
        changed_rows: &[usize],
    ) -> bool {
        self.state
            .borrow_mut()
            .patch_workbench_window_nodes(nodes, changed_rows)
    }

    pub(crate) fn patch_host_presentation_dock(
        &self,
        expected_structure_generation: u64,
        next_shell: HostWindowShellData,
        next_layout: HostWindowLayoutData,
        patch: HostDockPresentationPatch,
        replacements: &[(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)],
    ) -> bool {
        self.state.borrow_mut().patch_host_presentation_dock(
            expected_structure_generation,
            next_shell,
            next_layout,
            patch,
            replacements,
        )
    }

    pub(crate) fn patch_scene_viewport_chrome(
        &self,
        viewport: SceneViewportChromeData,
        status_grid_text: &str,
        status_snap_text: &str,
    ) -> bool {
        self.state.borrow_mut().patch_scene_viewport_chrome(
            viewport,
            status_grid_text,
            status_snap_text,
        )
    }

    pub(crate) fn get_host_presentation_generation(&self) -> HostPresentationGeneration {
        record_current_ui_perf_counter(UiPerfCounter::PresentationGenerationReadCount, 1.0);
        self.state.borrow().presentation_generation()
    }

    pub(crate) fn sync_host_paint_theme(&self) -> bool {
        self.state.borrow_mut().sync_host_paint_theme()
    }

    pub(crate) fn get_menu_state(&self) -> HostMenuStateData {
        self.state.borrow().menu_state.as_ref().clone()
    }

    pub(crate) fn get_pane_interaction_state(&self) -> HostPaneInteractionStateData {
        self.state.borrow().pane_interaction_state.as_ref().clone()
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
