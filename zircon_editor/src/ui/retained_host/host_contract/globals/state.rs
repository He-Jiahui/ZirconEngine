use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

use crate::ui::retained_host::primitives::{CloseRequestResponse, PhysicalPosition, PhysicalSize};
use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::super::data::{
    HostDragStateData, HostMenuStateData, HostPageOverflowMenuStateData,
    HostPaneInteractionStateData, HostResizeStateData, HostTextInputFocusData,
    HostViewportImageData, HostWindowPresentationData, WelcomePaneData,
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
    pub(crate) host_presentation: HostWindowPresentationData,
    pub(crate) refresh_invalidation_diagnostics: HostInvalidationDiagnostics,
    pub(crate) presentation_rebuild_count: u64,
    pub(crate) external_redraw_request: HostRedrawRequest,
    pub(crate) external_redraw_queued_count: u64,
    pub(crate) external_redraw_drained_count: u64,
    pub(crate) external_redraw_coalesced_count: u64,
    pub(crate) runtime_frame_wake_deadline: Option<Instant>,
    pub(crate) completed_frame_update_scenario: Option<UiPerfScenario>,
    pub(crate) viewport_image: Option<HostViewportImageData>,
    pub(crate) menu_state: HostMenuStateData,
    pub(crate) host_page_overflow_menu_state: HostPageOverflowMenuStateData,
    pub(crate) pane_interaction_state: HostPaneInteractionStateData,
    pub(crate) drag_state: HostDragStateData,
    pub(crate) resize_state: HostResizeStateData,
    pub(crate) text_input_focus: HostTextInputFocusData,
    pub(crate) welcome_pane: WelcomePaneData,
    pub(in crate::ui::retained_host::host_contract) ui_callbacks: UiHostCallbacks,
    pub(in crate::ui::retained_host::host_contract) pane_callbacks: PaneSurfaceCallbacks,
}

impl HostContractState {
    pub(crate) const DEFAULT_WINDOW_SCALE_FACTOR: f32 = 1.0;

    pub(crate) fn new(window_size: PhysicalSize) -> Self {
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
            host_presentation: HostWindowPresentationData::default(),
            refresh_invalidation_diagnostics: HostInvalidationDiagnostics::default(),
            presentation_rebuild_count: 0,
            external_redraw_request: HostRedrawRequest::none(),
            external_redraw_queued_count: 0,
            external_redraw_drained_count: 0,
            external_redraw_coalesced_count: 0,
            runtime_frame_wake_deadline: None,
            completed_frame_update_scenario: None,
            viewport_image: None,
            menu_state: HostMenuStateData::default(),
            host_page_overflow_menu_state: HostPageOverflowMenuStateData::default(),
            pane_interaction_state: HostPaneInteractionStateData::default(),
            drag_state: HostDragStateData::default(),
            resize_state: HostResizeStateData::default(),
            text_input_focus: HostTextInputFocusData::default(),
            welcome_pane: WelcomePaneData::default(),
            ui_callbacks: UiHostCallbacks::default(),
            pane_callbacks: PaneSurfaceCallbacks::default(),
        }
    }

    pub(crate) fn set_window_scale_factor(&mut self, scale_factor: f32) {
        self.window_scale_factor = Self::normalize_window_scale_factor(scale_factor);
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
