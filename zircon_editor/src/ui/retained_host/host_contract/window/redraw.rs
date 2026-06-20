use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::super::data::FrameRect;
use super::super::globals::UiHostContext;
use super::super::redraw::HostRedrawRequest;
use super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn request_frame_update(&self) {
        self.global::<UiHostContext>().invoke_frame_requested();
    }

    pub(crate) fn mark_completed_frame_update_scenario(&self, scenario: UiPerfScenario) {
        self.state.borrow_mut().completed_frame_update_scenario = Some(scenario);
    }

    pub(in crate::ui::retained_host::host_contract) fn take_completed_frame_update_scenario(
        &self,
    ) -> Option<UiPerfScenario> {
        self.state
            .borrow_mut()
            .completed_frame_update_scenario
            .take()
    }

    pub(crate) fn request_redraw_region(&self, frame: FrameRect) {
        self.queue_external_redraw(HostRedrawRequest::region(frame));
    }

    pub(crate) fn request_frame_update_region(&self, frame: FrameRect) {
        let redraw = HostRedrawRequest::region_with_frame_update(frame);
        if redraw.request_redraw() {
            self.queue_external_redraw(redraw);
        } else {
            self.queue_external_redraw(HostRedrawRequest::full_frame());
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn queue_external_redraw(
        &self,
        redraw: HostRedrawRequest,
    ) {
        if !redraw.request_redraw() {
            return;
        }
        let mut state = self.state.borrow_mut();
        let existing = std::mem::replace(
            &mut state.external_redraw_request,
            HostRedrawRequest::none(),
        );
        if existing.request_redraw() {
            state.external_redraw_coalesced_count =
                state.external_redraw_coalesced_count.saturating_add(1);
        }
        state.external_redraw_request = existing.merge(redraw);
        state.external_redraw_queued_count = state.external_redraw_queued_count.saturating_add(1);
    }

    pub(in crate::ui::retained_host::host_contract) fn take_external_redraw(
        &self,
    ) -> HostRedrawRequest {
        let mut state = self.state.borrow_mut();
        let redraw = std::mem::replace(
            &mut state.external_redraw_request,
            HostRedrawRequest::none(),
        );
        if redraw.request_redraw() {
            state.external_redraw_drained_count =
                state.external_redraw_drained_count.saturating_add(1);
        }
        redraw
    }
}
