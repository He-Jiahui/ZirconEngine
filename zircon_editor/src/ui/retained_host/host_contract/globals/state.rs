use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use crate::ui::retained_host::host_contract::paint_theme::{
    capture_host_paint_theme_snapshot, HostPaintThemeSnapshot,
};
use crate::ui::retained_host::host_contract::surface_hit_test::HostWorkbenchHitIndex;
use crate::ui::retained_host::primitives::{
    CloseRequestResponse, PhysicalPosition, PhysicalSize, SharedString,
};
use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::super::data::{
    HostDragStateData, HostMenuStateData, HostPageOverflowMenuStateData,
    HostPaneInteractionStateData, HostPresentationGeneration, HostResizeStateData,
    HostTextInputFocusData, HostViewportImageData, HostWindowPresentationData, WelcomePaneData,
};
use super::super::diagnostics::HostInvalidationDiagnostics;
use super::super::redraw::HostRedrawRequest;
use super::callbacks::{PaneSurfaceCallbacks, UiHostCallbacks};

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
    pub(crate) first_presented_frame_capture_path: Option<PathBuf>,
    pub(crate) first_presented_frame_capture_error: Option<String>,
    pub(crate) window_maximized: bool,
    pub(crate) close_requested: Option<Rc<dyn Fn() -> CloseRequestResponse>>,
    pub(crate) host_presentation: Arc<HostWindowPresentationData>,
    presentation_structure_generation: u64,
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
    pub(crate) completed_frame_update_scenario: Option<UiPerfScenario>,
    pub(crate) viewport_image: Option<Arc<HostViewportImageData>>,
    pub(crate) menu_state: Arc<HostMenuStateData>,
    pub(crate) host_page_overflow_menu_state: Arc<HostPageOverflowMenuStateData>,
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
            window_maximized: false,
            close_requested: None,
            host_presentation,
            presentation_structure_generation: 0,
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
            completed_frame_update_scenario: None,
            viewport_image: None,
            menu_state: Arc::new(HostMenuStateData::default()),
            host_page_overflow_menu_state: Arc::new(HostPageOverflowMenuStateData::default()),
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
    }

    pub(crate) fn presentation_generation(&self) -> HostPresentationGeneration {
        HostPresentationGeneration::new(
            Arc::clone(&self.host_presentation),
            Arc::clone(&self.menu_state),
            Arc::clone(&self.host_page_overflow_menu_state),
            Arc::clone(&self.pane_interaction_state),
            Arc::clone(&self.text_input_focus),
            self.viewport_image.as_ref().map(Arc::clone),
            Arc::clone(&self.workbench_hit_index),
            Arc::clone(&self.host_paint_theme),
            Arc::clone(&self.diagnostics_overlay_text),
            self.presentation_structure_generation,
            self.presentation_interaction_generation,
            self.presentation_viewport_generation,
            self.presentation_hit_test_generation,
            self.presentation_diagnostics_generation,
        )
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
        presentation.pane_interaction_state = HostPaneInteractionStateData::default();
        presentation.text_input_focus = HostTextInputFocusData::default();
        presentation.viewport_image = None;
        self.replace_diagnostics_overlay_text(presentation.host_shell.debug_refresh_rate.clone());
        self.host_presentation = Arc::new(presentation);
        self.presentation_structure_generation =
            self.presentation_structure_generation.saturating_add(1);
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
        self.presentation_structure_generation =
            self.presentation_structure_generation.saturating_add(1);
        result
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

    pub(crate) fn replace_viewport_image(&mut self, value: HostViewportImageData) -> bool {
        if self
            .viewport_image
            .as_ref()
            .is_some_and(|current| current.resource_key == value.resource_key)
        {
            return false;
        }
        self.viewport_image = Some(Arc::new(value));
        self.presentation_viewport_generation =
            self.presentation_viewport_generation.saturating_add(1);
        true
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
