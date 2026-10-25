#[path = "../../../contract/window/presentation/asset_deletion_blocker.rs"]
mod asset_deletion_blocker;
#[path = "presentation/close_prompt.rs"]
mod close_prompt;
#[path = "presentation/snapshot.rs"]
mod snapshot;
#[path = "presentation/hover_state.rs"]
mod template_hover_state;
#[path = "presentation/press_state.rs"]
mod template_press_state;
#[path = "presentation/viewport_toolbar.rs"]
mod viewport_toolbar;

use std::sync::Arc;

use crate::ui::retained_host::console_output::console_output_viewport_size;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::data::SceneViewportChromeData;
use super::super::data::{
    FrameRect, HostDockPresentationPatch, HostMenuStateData, HostPaneInteractionStateData,
    HostPanePresentationPatch, HostPresentationGeneration, HostPresentationPatch,
    HostWindowGeometryPresentationData, HostWindowLayoutData, HostWindowPresentationData,
    HostWindowShellData, TemplatePaneNodeData,
};
use super::UiHostWindow;
use crate::ui::retained_host::primitives::ModelRc;

pub(in crate::ui::retained_host::host_contract) use self::snapshot::host_presentation_from_state;

impl UiHostWindow {
    pub(crate) fn set_normal_arranged_inventory(
        &self,
        inventory: Option<crate::ui::template_runtime::NormalPresentInventory>,
    ) {
        let mut state = self.state.borrow_mut();
        let cursor = state.presentation_generation().cursor();
        // Capture metadata does not advance any visual generation.
        Arc::make_mut(&mut state.host_presentation).normal_arranged_inventory =
            inventory.map(|inventory| (cursor, Arc::new(inventory)));
    }

    pub(crate) fn set_prepared_viewport_toolbar_publication(
        &self,
        snapshot: Option<(
            Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
            Arc<serde_json::Value>,
        )>,
        mount: Option<zircon_runtime_interface::ui::layout::UiFrame>,
        scale: f32,
    ) {
        viewport_toolbar::set_capture(&mut self.state.borrow_mut(), snapshot, mount, scale);
    }

    pub(crate) fn set_normal_workbench_business_snapshot(
        &self,
        snapshot: Option<crate::ui::retained_host::ui::NormalWorkbenchBusinessSnapshot>,
    ) {
        let mut state = self.state.borrow_mut();
        let cursor = state.presentation_generation().cursor();
        let snapshot = snapshot.and_then(|snapshot| {
            match snapshot.with_consumed_unreal_dock_headers(state.host_presentation.as_ref()) {
                Ok(snapshot) => Some(snapshot),
                Err(error) => {
                    write_diagnostic_log(
                        "unreal_dock_header_publication",
                        format!("actual Header Source publication unavailable: {error}"),
                    );
                    None
                }
            }
        });
        // Capture metadata is not visual state. Copy-on-write preserves all previously
        // captured immutable generations without advancing paint/layout counters.
        Arc::make_mut(&mut state.host_presentation).normal_workbench_business =
            snapshot.map(|snapshot| (cursor, Arc::new(snapshot)));
    }

    #[track_caller]
    pub(crate) fn set_host_presentation(&self, mut presentation: HostWindowPresentationData) {
        self.prepare_canvas_anchor_popup_publication(&mut presentation);
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

    #[track_caller]
    pub(crate) fn set_host_geometry_presentation(
        &self,
        geometry: HostWindowGeometryPresentationData,
        workbench_changed_rows: &[usize],
    ) -> bool {
        self.retire_current_canvas_anchor_popup();
        let committed = self
            .state
            .borrow_mut()
            .replace_host_geometry_presentation(geometry, workbench_changed_rows);
        if committed {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.geometry_commit_count",
                1_u8
            );
        }
        committed
    }

    pub(crate) fn get_host_presentation(&self) -> HostWindowPresentationData {
        let state = self.state.borrow();
        host_presentation_from_state(&state)
    }

    #[track_caller]
    pub(crate) fn update_host_presentation<R>(
        &self,
        update: impl FnOnce(&mut HostWindowPresentationData) -> R,
    ) -> R {
        let mut state = self.state.borrow_mut();
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        let mut next = state.host_presentation.as_ref().clone();
        let result = update(&mut next);
        drop(state);
        self.prepare_canvas_anchor_popup_publication(&mut next);
        self.state.borrow_mut().replace_host_presentation(next);
        result
    }

    #[track_caller]
    pub(crate) fn set_native_floating_window_presentation(
        &self,
        window_id: &str,
        surface_tree_id: &str,
        title: &str,
        bounds: &FrameRect,
    ) -> bool {
        self.state
            .borrow_mut()
            .replace_native_floating_window_presentation(window_id, surface_tree_id, title, bounds)
    }

    #[track_caller]
    pub(crate) fn patch_host_presentation_panes<R>(
        &self,
        prepare: impl FnOnce(&HostWindowPresentationData) -> Option<(HostPanePresentationPatch, R)>,
    ) -> Option<R> {
        let (patch, result) = {
            let state = self.state.borrow();
            let (patch, result) = prepare(state.host_presentation.as_ref())?;
            patch.paint_model_replacements(state.host_presentation.as_ref())?;
            (patch, result)
        };
        if self
            .canvas_anchor_popup_instance()
            .as_ref()
            .is_some_and(|id| patch.changes_canvas_anchor_target(id.as_str()))
        {
            self.retire_current_canvas_anchor_popup();
        }
        let mut state = self.state.borrow_mut();
        if !state.patch_host_presentation_panes(patch) {
            return None;
        }
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        Some(result)
    }

    #[track_caller]
    pub(crate) fn patch_host_presentation<R>(
        &self,
        prepare: impl FnOnce(&HostWindowPresentationData) -> Option<(HostPresentationPatch, R)>,
    ) -> Option<R> {
        let (patch, result) = {
            let state = self.state.borrow();
            let (patch, result) = prepare(state.host_presentation.as_ref())?;
            patch.paint_model_replacements(state.host_presentation.as_ref())?;
            (patch, result)
        };
        if self
            .canvas_anchor_popup_instance()
            .as_ref()
            .is_some_and(|id| patch.changes_canvas_anchor_target(id.as_str()))
        {
            self.retire_current_canvas_anchor_popup();
        }
        let mut state = self.state.borrow_mut();
        if !state.patch_host_presentation(patch) {
            return None;
        }
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        record_current_ui_perf_counter(UiPerfCounter::PresentationRebuildCount, 1.0);
        Some(result)
    }

    #[track_caller]
    pub(crate) fn patch_workbench_window_nodes(
        &self,
        nodes: ModelRc<TemplatePaneNodeData>,
        changed_rows: &[usize],
    ) -> bool {
        self.state
            .borrow_mut()
            .patch_workbench_window_nodes(nodes, changed_rows)
    }

    #[track_caller]
    pub(crate) fn patch_host_presentation_dock(
        &self,
        expected_structure_generation: u64,
        next_shell: HostWindowShellData,
        next_layout: HostWindowLayoutData,
        patch: HostDockPresentationPatch,
        replacements: &[(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)],
    ) -> bool {
        let retirement = {
            let state = self.state.borrow();
            let generation = state.presentation_generation();
            if generation.structure_generation() != expected_structure_generation
                || generation
                    .workbench_hit_index()
                    .rebind_presentation_dock_patch(
                        state.host_presentation.as_ref(),
                        &patch,
                        replacements,
                    )
                    .is_none()
            {
                return false;
            }
            state
                .canvas_anchor_popup
                .as_ref()
                .map(|popup| (popup.id.clone(), std::rc::Rc::clone(&popup.retire)))
        };
        if let Some((_, retire)) = retirement.as_ref() {
            // Retire the original domain permit before publishing the layout.
            // Do not change native presentation generations before the guarded commit.
            if let Err(error) = retire() {
                write_diagnostic_log("canvas_anchor_popup_retire", error);
                return false;
            }
        }
        let committed = self.state.borrow_mut().patch_host_presentation_dock(
            expected_structure_generation,
            next_shell,
            next_layout,
            patch,
            replacements,
        );
        if let Some((id, _)) = retirement {
            self.close_canvas_anchor_popup(&id, false);
        }
        committed
    }

    pub(crate) fn patch_scene_viewport_chrome(
        &self,
        viewport: SceneViewportChromeData,
        status_grid_text: &str,
        status_snap_text: &str,
        status_snap_enabled: bool,
    ) -> bool {
        self.state.borrow_mut().patch_scene_viewport_chrome(
            viewport,
            status_grid_text,
            status_snap_text,
            status_snap_enabled,
        )
    }

    pub(crate) fn patch_native_scene_viewport_chrome(
        &self,
        row: usize,
        window_id: &str,
        viewport: SceneViewportChromeData,
    ) -> bool {
        self.state
            .borrow_mut()
            .patch_native_scene_viewport_chrome(row, window_id, viewport)
    }

    pub(crate) fn viewport_chrome_damage_frame(&self) -> Option<FrameRect> {
        let state = self.state.borrow();
        super::super::native_pointer::viewport_chrome_damage_frame(state.host_presentation.as_ref())
    }

    pub(crate) fn native_viewport_chrome_damage_frame(&self) -> Option<FrameRect> {
        let state = self.state.borrow();
        super::super::native_pointer::native_viewport_chrome_damage_frame(
            state.host_presentation.as_ref(),
        )
    }

    pub(crate) fn get_host_presentation_generation(&self) -> HostPresentationGeneration {
        record_current_ui_perf_counter(UiPerfCounter::PresentationGenerationReadCount, 1.0);
        self.state.borrow().presentation_generation()
    }

    pub(crate) fn get_host_interaction_generation(&self) -> u64 {
        self.state.borrow().interaction_generation()
    }

    pub(crate) fn get_pane_interaction_generation(&self) -> Arc<HostPaneInteractionStateData> {
        Arc::clone(&self.state.borrow().pane_interaction_state)
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
