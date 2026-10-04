use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use zircon_runtime::asset::project::ResolvedProjectPath;

use crate::core::play::PlayInstanceId;
use crate::ui::retained_host::host_contract::paint_theme::{
    capture_host_paint_theme_snapshot, HostPaintThemeSnapshot,
};
use crate::ui::retained_host::host_contract::surface_hit_test::HostWorkbenchHitIndex;
use crate::ui::retained_host::primitives::{
    CloseRequestResponse, ModelRc, PhysicalPosition, PhysicalSize, SharedString,
};
use crate::ui::retained_host::ui_perf::{
    record_current_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};

use super::super::data::{
    FrameRect, HostAssetDeletionBlockerData, HostClosePromptData, HostDockOverflowMenuStateData,
    HostDockPresentationPatch, HostDragStateData, HostMenuStateData, HostPageOverflowMenuStateData,
    HostPaneInteractionStateData, HostPanePresentationPatch, HostPresentationGeneration,
    HostPresentationPatch, HostResizeStateData, HostTextInputFocusData, HostViewportImageData,
    HostViewportImageSet, HostWindowGeometryPresentationData, HostWindowLayoutData,
    HostWindowPresentationData, HostWindowShellData, TemplatePaneNodeData, WelcomePaneData,
};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostWindowDiagnosticQueue};
use super::super::redraw::HostRedrawRequest;
use super::callbacks::{PaneSurfaceCallbacks, UiHostCallbacks};

mod template_press;
mod viewport_chrome;

pub(crate) trait HostContractGlobal: Sized {
    fn from_state(state: Rc<RefCell<HostContractState>>) -> Self;
}

pub(crate) struct HostContractState {
    pub(crate) window_position: PhysicalPosition,
    pub(crate) window_size: PhysicalSize,
    pub(crate) window_scale_factor: f32,
    pub(crate) window_visible: bool,
    pub(crate) exit_requested: bool,
    pub(crate) exit_after_first_presented_frame: bool,
    pub(crate) first_presented_frame_capture_path: Option<ResolvedProjectPath>,
    pub(crate) first_presented_frame_capture_error: Option<String>,
    pub(crate) host_window_diagnostics: HostWindowDiagnosticQueue,
    pub(crate) window_maximized: bool,
    pub(crate) close_requested: Option<Rc<dyn Fn() -> CloseRequestResponse>>,
    pub(crate) host_presentation: Arc<HostWindowPresentationData>,
    presentation_structure_generation: u64,
    presentation_geometry_generation: u64,
    presentation_interaction_generation: u64,
    presentation_viewport_generation: u64,
    presentation_hit_test_generation: u64,
    presentation_diagnostics_generation: u64,
    workbench_hit_index: Arc<HostWorkbenchHitIndex>,
    host_paint_theme: Arc<HostPaintThemeSnapshot>,
    pub(crate) diagnostics_overlay_text: Arc<SharedString>,
    pub(crate) refresh_invalidation_diagnostics: HostInvalidationDiagnostics,
    pub(crate) presentation_rebuild_count: u64,
    pub(crate) external_redraw_request: HostRedrawRequest,
    pub(crate) external_redraw_queued_count: u64,
    pub(crate) external_redraw_drained_count: u64,
    pub(crate) external_redraw_coalesced_count: u64,
    pub(crate) runtime_frame_wake_deadline: Option<Instant>,
    pub(crate) runtime_frame_wake_tick_pending: bool,
    pub(crate) runtime_frame_failure_retry_attempts: u32,
    pub(crate) runtime_frame_owner: Option<(PlayInstanceId, u64)>,
    pub(crate) maintenance_frame_wake_deadline: Option<Instant>,
    pub(crate) input_timer_frame_wake_deadline: Option<Instant>,
    pub(crate) lifecycle_frame_wake_deadline: Option<Instant>,
    pub(crate) completed_frame_update_scenario: Option<UiPerfScenario>,
    pub(crate) viewport_images: HostViewportImageSet,
    pub(crate) scene_surface_key: Option<String>,
    pub(crate) menu_state: Arc<HostMenuStateData>,
    pub(crate) host_page_overflow_menu_state: Arc<HostPageOverflowMenuStateData>,
    pub(crate) host_dock_overflow_menu_state: Arc<HostDockOverflowMenuStateData>,
    pub(crate) pane_interaction_state: Arc<HostPaneInteractionStateData>,
    pub(crate) drag_state: HostDragStateData,
    pub(crate) resize_state: HostResizeStateData,
    pub(crate) text_input_focus: Arc<HostTextInputFocusData>,
    pub(crate) welcome_pane: WelcomePaneData,
    pub(in crate::ui::retained_host::host_contract) ui_callbacks: UiHostCallbacks,
    pub(in crate::ui::retained_host::host_contract) pane_callbacks: PaneSurfaceCallbacks,
}

impl HostContractState {
    pub(crate) const DEFAULT_WINDOW_SCALE_FACTOR: f32 = 1.0;

    pub(crate) fn new(window_size: PhysicalSize) -> Self {
        let host_presentation = Arc::new(HostWindowPresentationData::default());
        let diagnostics_overlay_text =
            Arc::new(host_presentation.host_shell.debug_refresh_rate.clone());
        let workbench_hit_index =
            Arc::new(HostWorkbenchHitIndex::from_presentation(&host_presentation));
        let host_paint_theme = capture_host_paint_theme_snapshot();
        Self {
            window_position: PhysicalPosition::new(0, 0),
            window_size,
            window_scale_factor: Self::DEFAULT_WINDOW_SCALE_FACTOR,
            window_visible: false,
            exit_requested: false,
            exit_after_first_presented_frame: false,
            first_presented_frame_capture_path: None,
            first_presented_frame_capture_error: None,
            host_window_diagnostics: HostWindowDiagnosticQueue::default(),
            window_maximized: false,
            close_requested: None,
            host_presentation,
            presentation_structure_generation: 0,
            presentation_geometry_generation: 0,
            presentation_interaction_generation: 0,
            presentation_viewport_generation: 0,
            presentation_hit_test_generation: 0,
            presentation_diagnostics_generation: 0,
            workbench_hit_index,
            host_paint_theme,
            diagnostics_overlay_text,
            refresh_invalidation_diagnostics: HostInvalidationDiagnostics::default(),
            presentation_rebuild_count: 0,
            external_redraw_request: HostRedrawRequest::none(),
            external_redraw_queued_count: 0,
            external_redraw_drained_count: 0,
            external_redraw_coalesced_count: 0,
            runtime_frame_wake_deadline: None,
            runtime_frame_wake_tick_pending: false,
            runtime_frame_failure_retry_attempts: 0,
            runtime_frame_owner: None,
            maintenance_frame_wake_deadline: None,
            input_timer_frame_wake_deadline: None,
            lifecycle_frame_wake_deadline: None,
            completed_frame_update_scenario: None,
            viewport_images: HostViewportImageSet::default(),
            scene_surface_key: None,
            menu_state: Arc::new(HostMenuStateData::default()),
            host_page_overflow_menu_state: Arc::new(HostPageOverflowMenuStateData::default()),
            host_dock_overflow_menu_state: Arc::new(HostDockOverflowMenuStateData::default()),
            pane_interaction_state: Arc::new(HostPaneInteractionStateData::default()),
            drag_state: HostDragStateData::default(),
            resize_state: HostResizeStateData::default(),
            text_input_focus: Arc::new(HostTextInputFocusData::default()),
            welcome_pane: WelcomePaneData::default(),
            ui_callbacks: UiHostCallbacks::default(),
            pane_callbacks: PaneSurfaceCallbacks::default(),
        }
    }

    pub(crate) fn set_window_scale_factor(&mut self, scale_factor: f32) {
        self.window_scale_factor = Self::normalize_window_scale_factor(scale_factor);
        self.reconcile_template_button_press();
    }

    pub(crate) fn presentation_generation(&self) -> HostPresentationGeneration {
        HostPresentationGeneration::new_with_geometry(
            Arc::clone(&self.host_presentation),
            Arc::clone(&self.menu_state),
            Arc::clone(&self.host_page_overflow_menu_state),
            Arc::clone(&self.host_dock_overflow_menu_state),
            Arc::clone(&self.pane_interaction_state),
            Arc::clone(&self.text_input_focus),
            self.viewport_images.clone(),
            Arc::clone(&self.workbench_hit_index),
            Arc::clone(&self.host_paint_theme),
            Arc::clone(&self.diagnostics_overlay_text),
            self.presentation_structure_generation,
            self.presentation_geometry_generation,
            self.presentation_interaction_generation,
            self.presentation_viewport_generation,
            self.presentation_hit_test_generation,
            self.presentation_diagnostics_generation,
        )
    }

    pub(crate) const fn interaction_generation(&self) -> u64 {
        self.presentation_interaction_generation
    }

    fn advance_structure_generation(&mut self) {
        self.presentation_structure_generation =
            self.presentation_structure_generation.saturating_add(1);
        record_current_ui_perf_counter(
            UiPerfCounter::PresentationStructureGenerationChangeCount,
            1.0,
        );
    }

    fn advance_geometry_generation(&mut self) {
        self.presentation_geometry_generation =
            self.presentation_geometry_generation.saturating_add(1);
        self.reconcile_template_button_press();
    }

    pub(crate) fn replace_host_presentation(
        &mut self,
        mut presentation: HostWindowPresentationData,
    ) {
        if !self.workbench_hit_index.indexes_presentation(&presentation) {
            self.workbench_hit_index =
                Arc::new(HostWorkbenchHitIndex::from_presentation(&presentation));
            self.presentation_hit_test_generation =
                self.presentation_hit_test_generation.saturating_add(1);
        }
        presentation.menu_state = HostMenuStateData::default();
        presentation.host_page_overflow_menu_state = HostPageOverflowMenuStateData::default();
        presentation.host_dock_overflow_menu_state = HostDockOverflowMenuStateData::default();
        presentation.pane_interaction_state = HostPaneInteractionStateData::default();
        presentation.text_input_focus = HostTextInputFocusData::default();
        presentation.viewport_images = HostViewportImageSet::default();
        self.replace_diagnostics_overlay_text(presentation.host_shell.debug_refresh_rate.clone());
        let presentation = Arc::new(presentation);
        self.host_presentation = presentation;
        self.advance_structure_generation();
        self.advance_geometry_generation();
    }

    pub(crate) fn replace_host_geometry_presentation(
        &mut self,
        geometry: HostWindowGeometryPresentationData,
        workbench_changed_rows: &[usize],
    ) -> bool {
        let presentation = geometry.apply_to(&self.host_presentation);
        let Some(next_hit_index) = self.workbench_hit_index.patch_geometry_presentation(
            &self.host_presentation,
            &presentation,
            workbench_changed_rows,
        ) else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.hit_index_geometry_patch_fallback_count",
                1_u8
            );
            return false;
        };
        self.workbench_hit_index = Arc::new(next_hit_index);
        self.presentation_hit_test_generation =
            self.presentation_hit_test_generation.saturating_add(1);
        self.host_presentation = Arc::new(presentation);
        self.advance_geometry_generation();
        true
    }

    pub(crate) fn update_host_presentation<R>(
        &mut self,
        update: impl FnOnce(&mut HostWindowPresentationData) -> R,
    ) -> R {
        let result = update(Arc::make_mut(&mut self.host_presentation));
        if !self
            .workbench_hit_index
            .indexes_presentation(&self.host_presentation)
        {
            self.workbench_hit_index = Arc::new(HostWorkbenchHitIndex::from_presentation(
                &self.host_presentation,
            ));
            self.presentation_hit_test_generation =
                self.presentation_hit_test_generation.saturating_add(1);
        }
        self.advance_structure_generation();
        self.advance_geometry_generation();
        result
    }

    pub(crate) fn replace_close_prompt(&mut self, prompt: HostClosePromptData) {
        Arc::make_mut(&mut self.host_presentation).close_prompt = prompt;
        self.advance_structure_generation();
        self.advance_geometry_generation();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.overlay_presentation.close_prompt_commit_count",
            1_u8
        );
    }

    pub(crate) fn replace_asset_deletion_blocker(&mut self, blocker: HostAssetDeletionBlockerData) {
        Arc::make_mut(&mut self.host_presentation).asset_deletion_blocker = blocker;
        self.advance_structure_generation();
        self.advance_geometry_generation();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.overlay_presentation.asset_deletion_blocker_commit_count",
            1_u8
        );
    }

    pub(crate) fn replace_native_floating_window_presentation(
        &mut self,
        window_id: &str,
        surface_tree_id: &str,
        title: &str,
        bounds: &FrameRect,
    ) -> bool {
        let presentation = self.host_presentation.as_ref();
        let shell = &presentation.host_shell;
        let surface = &presentation.native_floating_surface_data;
        let structure_changed = !shell.native_floating_window_mode
            || shell.native_floating_window_id != window_id
            || shell.native_surface_tree_id != surface_tree_id
            || shell.native_window_title != title
            || surface.native_floating_window_id != window_id
            || surface.native_surface_tree_id != surface_tree_id;
        let geometry_changed =
            shell.native_window_bounds != *bounds || surface.native_window_bounds != *bounds;
        if !structure_changed && !geometry_changed {
            return false;
        }

        let presentation = Arc::make_mut(&mut self.host_presentation);
        presentation.host_shell.native_floating_window_mode = true;
        replace_string(
            &mut presentation.host_shell.native_floating_window_id,
            window_id,
        );
        replace_string(
            &mut presentation.host_shell.native_surface_tree_id,
            surface_tree_id,
        );
        replace_string(&mut presentation.host_shell.native_window_title, title);
        presentation.host_shell.native_window_bounds = bounds.clone();
        replace_string(
            &mut presentation
                .native_floating_surface_data
                .native_floating_window_id,
            window_id,
        );
        replace_string(
            &mut presentation
                .native_floating_surface_data
                .native_surface_tree_id,
            surface_tree_id,
        );
        presentation
            .native_floating_surface_data
            .native_window_bounds = bounds.clone();
        if structure_changed {
            self.advance_structure_generation();
        }
        self.advance_geometry_generation();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.native_floating_presentation.commit_count",
            1_u8
        );
        true
    }

    pub(crate) fn patch_host_presentation_panes(
        &mut self,
        patch: HostPanePresentationPatch,
    ) -> bool {
        let Some(model_replacements) =
            patch.paint_model_replacements(self.host_presentation.as_ref())
        else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.pane_presentation_transaction.validation_fallback_count",
                1_u8
            );
            return false;
        };
        let replacement_count = patch.replacement_count();
        let rebind_node_count = model_replacements
            .iter()
            .map(|(_, next)| next.row_count())
            .sum::<usize>();
        let next_hit_index = if model_replacements.is_empty() {
            None
        } else {
            let Some(index) = self
                .workbench_hit_index
                .rebind_paint_models(&model_replacements)
            else {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.pane_presentation_transaction.hit_index_fallback_count",
                    1_u8
                );
                return false;
            };
            Some(index)
        };

        patch.apply(Arc::make_mut(&mut self.host_presentation));
        if let Some(index) = next_hit_index {
            self.workbench_hit_index = Arc::new(index);
        }
        self.presentation_hit_test_generation =
            self.presentation_hit_test_generation.saturating_add(1);
        self.advance_structure_generation();
        self.advance_geometry_generation();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.pane_presentation_transaction.commit_count",
            1_u8
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.pane_presentation_transaction.pane_replacement_count",
            replacement_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.pane_presentation_transaction.paint_model_rebind_count",
            model_replacements.len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.pane_presentation_transaction.paint_model_rebind_node_count",
            rebind_node_count
        );
        true
    }

    pub(crate) fn patch_host_presentation(&mut self, patch: HostPresentationPatch) -> bool {
        if patch.is_empty() {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.sparse_presentation_transaction.empty_fallback_count",
                1_u8
            );
            return false;
        }
        let Some(model_replacements) =
            patch.paint_model_replacements(self.host_presentation.as_ref())
        else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.sparse_presentation_transaction.pane_validation_fallback_count",
                1_u8
            );
            return false;
        };
        let pane_replacement_count = patch.pane_replacement_count();
        let workbench_changed_row_count = patch
            .workbench_nodes()
            .map_or(0, |(_, changed_rows)| changed_rows.len());

        let mut next_hit_index = if let Some((next_nodes, changed_rows)) = patch.workbench_nodes() {
            let previous_nodes = &self.host_presentation.workbench_window_nodes;
            let Some(index) = self.workbench_hit_index.rebind_workbench_nodes(
                previous_nodes,
                next_nodes,
                changed_rows,
            ) else {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.sparse_presentation_transaction.workbench_rebind_fallback_count",
                    1_u8
                );
                return false;
            };
            Some(index)
        } else {
            None
        };
        if !model_replacements.is_empty() {
            let source_index = next_hit_index.as_ref().unwrap_or(&self.workbench_hit_index);
            let Some(index) = source_index.rebind_paint_models(&model_replacements) else {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.sparse_presentation_transaction.paint_rebind_fallback_count",
                    1_u8
                );
                return false;
            };
            next_hit_index = Some(index);
        }

        patch.apply(Arc::make_mut(&mut self.host_presentation));
        if let Some(index) = next_hit_index {
            self.workbench_hit_index = Arc::new(index);
            self.presentation_hit_test_generation =
                self.presentation_hit_test_generation.saturating_add(1);
        }
        self.advance_structure_generation();
        self.advance_geometry_generation();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.sparse_presentation_transaction.commit_count",
            1_u8
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.sparse_presentation_transaction.pane_replacement_count",
            pane_replacement_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.sparse_presentation_transaction.workbench_changed_row_count",
            workbench_changed_row_count
        );
        true
    }

    pub(crate) fn patch_workbench_window_nodes(
        &mut self,
        next_nodes: ModelRc<TemplatePaneNodeData>,
        changed_rows: &[usize],
    ) -> bool {
        let previous_nodes = self.host_presentation.workbench_window_nodes.clone();
        let Some(next_hit_index) = self.workbench_hit_index.rebind_workbench_nodes(
            &previous_nodes,
            &next_nodes,
            changed_rows,
        ) else {
            return false;
        };
        Arc::make_mut(&mut self.host_presentation).workbench_window_nodes = next_nodes;
        self.workbench_hit_index = Arc::new(next_hit_index);
        self.advance_structure_generation();
        self.advance_geometry_generation();
        true
    }

    pub(crate) fn patch_host_presentation_dock(
        &mut self,
        expected_structure_generation: u64,
        next_shell: HostWindowShellData,
        next_layout: HostWindowLayoutData,
        patch: HostDockPresentationPatch,
        replacements: &[(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)],
    ) -> bool {
        if self.presentation_structure_generation != expected_structure_generation {
            return false;
        }
        let Some(next_hit_index) = self.workbench_hit_index.rebind_presentation_dock_patch(
            &self.host_presentation,
            &patch,
            replacements,
        ) else {
            return false;
        };

        if Arc::strong_count(&self.host_presentation) > 1 {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.structure_copy_count",
                1_u8
            );
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.shell_content.structure_in_place_count",
                1_u8
            );
        }
        let presentation = Arc::make_mut(&mut self.host_presentation);
        presentation.host_shell = next_shell;
        presentation.host_layout = next_layout;
        match patch {
            HostDockPresentationPatch::Left(next) => presentation.host_scene_data.left_dock = next,
            HostDockPresentationPatch::Right(next) => {
                presentation.host_scene_data.right_dock = next;
            }
            HostDockPresentationPatch::Bottom(next) => {
                presentation.host_scene_data.bottom_dock = next;
            }
        }
        self.workbench_hit_index = Arc::new(next_hit_index);
        self.advance_structure_generation();
        self.advance_geometry_generation();
        self.presentation_hit_test_generation =
            self.presentation_hit_test_generation.saturating_add(1);
        true
    }

    pub(crate) fn replace_menu_state(&mut self, value: HostMenuStateData) -> bool {
        if self.menu_state.as_ref() == &value {
            return false;
        }
        self.menu_state = Arc::new(value);
        self.advance_interaction_generation();
        true
    }

    pub(crate) fn replace_page_overflow_menu_state(
        &mut self,
        value: HostPageOverflowMenuStateData,
    ) -> bool {
        if self.host_page_overflow_menu_state.as_ref() == &value {
            return false;
        }
        self.host_page_overflow_menu_state = Arc::new(value);
        self.advance_interaction_generation();
        true
    }

    pub(crate) fn replace_dock_overflow_menu_state(
        &mut self,
        value: HostDockOverflowMenuStateData,
    ) -> bool {
        if self.host_dock_overflow_menu_state.as_ref() == &value {
            return false;
        }
        self.host_dock_overflow_menu_state = Arc::new(value);
        self.advance_interaction_generation();
        true
    }

    pub(crate) fn update_pane_interaction(
        &mut self,
        update: impl FnOnce(&mut HostPaneInteractionStateData),
    ) -> bool {
        let mut value = self.pane_interaction_state.as_ref().clone();
        update(&mut value);
        if self.pane_interaction_state.as_ref() == &value {
            return false;
        }
        self.pane_interaction_state = Arc::new(value);
        self.advance_interaction_generation();
        true
    }

    pub(crate) fn replace_text_input_focus(&mut self, value: HostTextInputFocusData) -> bool {
        if self.text_input_focus.as_ref() == &value {
            return false;
        }
        self.text_input_focus = Arc::new(value);
        self.advance_interaction_generation();
        true
    }

    pub(crate) fn replace_scene_viewport_image(&mut self, value: HostViewportImageData) -> bool {
        let surface_key = self.scene_surface_key.clone();
        let changed = match surface_key.as_deref() {
            Some(surface_key) => self
                .viewport_images
                .replace_scene_for_surface(surface_key, value),
            None => self.viewport_images.replace_scene(value),
        };
        self.advance_viewport_generation_when(changed);
        changed
    }

    pub(crate) fn replace_scene_viewport_image_for_surface(
        &mut self,
        surface_key: &str,
        value: HostViewportImageData,
    ) -> bool {
        let changed = self
            .viewport_images
            .replace_scene_for_surface(surface_key, value);
        self.advance_viewport_generation_when(changed);
        changed
    }

    pub(crate) fn set_scene_surface_key(&mut self, surface_key: &str) {
        self.scene_surface_key = (!surface_key.is_empty()).then(|| surface_key.to_string());
    }

    pub(crate) fn replace_game_viewport_image(&mut self, value: HostViewportImageData) -> bool {
        let changed = self.viewport_images.replace_game(value);
        self.advance_viewport_generation_when(changed);
        changed
    }

    pub(crate) fn replace_simulate_viewport_image(&mut self, value: HostViewportImageData) -> bool {
        let changed = self.viewport_images.replace_simulate(value);
        self.advance_viewport_generation_when(changed);
        changed
    }

    pub(crate) fn clear_game_viewport_image(&mut self) -> bool {
        let changed = self.viewport_images.clear_game();
        self.advance_viewport_generation_when(changed);
        changed
    }

    pub(crate) fn clear_simulate_viewport_image(&mut self) -> bool {
        let changed = self.viewport_images.clear_simulate();
        self.advance_viewport_generation_when(changed);
        changed
    }

    fn advance_viewport_generation_when(&mut self, changed: bool) {
        if changed {
            self.presentation_viewport_generation =
                self.presentation_viewport_generation.saturating_add(1);
        }
    }

    pub(crate) fn replace_diagnostics_overlay_text(&mut self, value: SharedString) -> bool {
        if self.diagnostics_overlay_text.as_ref() == &value {
            return false;
        }
        self.diagnostics_overlay_text = Arc::new(value);
        self.presentation_diagnostics_generation =
            self.presentation_diagnostics_generation.saturating_add(1);
        true
    }

    pub(crate) fn sync_host_paint_theme(&mut self) -> bool {
        let theme = capture_host_paint_theme_snapshot();
        if Arc::ptr_eq(&self.host_paint_theme, &theme)
            || self.host_paint_theme.generation() == theme.generation()
        {
            return false;
        }
        self.host_paint_theme = theme;
        true
    }

    fn advance_interaction_generation(&mut self) {
        self.presentation_interaction_generation =
            self.presentation_interaction_generation.saturating_add(1);
    }

    pub(crate) fn window_scale_factor(&self) -> f32 {
        self.window_scale_factor
    }

    pub(crate) fn normalize_window_scale_factor(scale_factor: f32) -> f32 {
        if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            Self::DEFAULT_WINDOW_SCALE_FACTOR
        }
    }
}

fn replace_string(target: &mut String, value: &str) {
    target.clear();
    target.push_str(value);
}
