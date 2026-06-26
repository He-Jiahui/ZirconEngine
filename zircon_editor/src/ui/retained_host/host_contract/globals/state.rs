use std::cell::RefCell;
use std::rc::Rc;

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
    pub(crate) window_visible: bool,
    pub(crate) exit_requested: bool,
    pub(crate) window_maximized: bool,
    pub(crate) close_requested: Option<Rc<dyn Fn() -> CloseRequestResponse>>,
    pub(crate) host_presentation: HostWindowPresentationData,
    pub(crate) refresh_invalidation_diagnostics: HostInvalidationDiagnostics,
    pub(crate) presentation_rebuild_count: u64,
    pub(crate) external_redraw_request: HostRedrawRequest,
    pub(crate) external_redraw_queued_count: u64,
    pub(crate) external_redraw_drained_count: u64,
    pub(crate) external_redraw_coalesced_count: u64,
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
    pub(crate) fn new(window_size: PhysicalSize) -> Self {
        Self {
            window_position: PhysicalPosition::new(0, 0),
            window_size,
            window_visible: false,
            exit_requested: false,
            window_maximized: false,
            close_requested: None,
            host_presentation: HostWindowPresentationData::default(),
            refresh_invalidation_diagnostics: HostInvalidationDiagnostics::default(),
            presentation_rebuild_count: 0,
            external_redraw_request: HostRedrawRequest::none(),
            external_redraw_queued_count: 0,
            external_redraw_drained_count: 0,
            external_redraw_coalesced_count: 0,
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
}
