use std::time::{Duration, Instant};

use crate::core::gateway::EditorRuntimeFrameDemand;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::super::data::FrameRect;
use super::super::globals::UiHostContext;
use super::super::redraw::HostRedrawRequest;
use super::UiHostWindow;

const MAX_RUNTIME_FRAME_WAKE_DELAY: Duration = Duration::from_secs(60);

impl UiHostWindow {
    /// Replaces the prior runtime request because each completed runtime tick owns the next wake.
    pub(crate) fn apply_runtime_frame_demand(
        &self,
        demand: EditorRuntimeFrameDemand,
        now: Instant,
    ) {
        let queue_immediate_frame = match demand {
            EditorRuntimeFrameDemand::OnDemand => {
                self.state.borrow_mut().runtime_frame_wake_deadline = None;
                false
            }
            EditorRuntimeFrameDemand::SleepUntil(delay) => {
                let delay = delay.min(MAX_RUNTIME_FRAME_WAKE_DELAY);
                self.state.borrow_mut().runtime_frame_wake_deadline =
                    Some(now.checked_add(delay).unwrap_or(now));
                false
            }
            EditorRuntimeFrameDemand::Continuous => {
                self.state.borrow_mut().runtime_frame_wake_deadline = None;
                true
            }
        };
        if queue_immediate_frame {
            self.queue_external_redraw(HostRedrawRequest::full_frame());
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn runtime_frame_wake_deadline(
        &self,
    ) -> Option<Instant> {
        self.state.borrow().runtime_frame_wake_deadline
    }

    pub(in crate::ui::retained_host::host_contract) fn take_due_runtime_frame_wake(
        &self,
        now: Instant,
    ) -> bool {
        let due = self
            .state
            .borrow()
            .runtime_frame_wake_deadline
            .is_some_and(|deadline| deadline <= now);
        if due {
            self.state.borrow_mut().runtime_frame_wake_deadline = None;
            self.queue_external_redraw(HostRedrawRequest::full_frame());
        }
        due
    }

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

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::core::gateway::EditorRuntimeFrameDemand;

    #[test]
    fn runtime_frame_wake_replaces_stale_requests_and_bounds_extreme_delays() {
        let host = super::UiHostWindow::new().expect("host window");
        let now = Instant::now();

        host.apply_runtime_frame_demand(EditorRuntimeFrameDemand::Continuous, now);
        assert!(host.take_external_redraw().requires_frame_update());
        assert_eq!(host.runtime_frame_wake_deadline(), None);

        host.apply_runtime_frame_demand(
            EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(25)),
            now,
        );
        assert_eq!(
            host.runtime_frame_wake_deadline(),
            Some(now + Duration::from_millis(25))
        );
        host.apply_runtime_frame_demand(EditorRuntimeFrameDemand::OnDemand, now);
        assert_eq!(host.runtime_frame_wake_deadline(), None);
        assert!(!host.take_due_runtime_frame_wake(now + Duration::from_millis(25)));

        host.apply_runtime_frame_demand(
            EditorRuntimeFrameDemand::SleepUntil(Duration::from_millis(25)),
            now,
        );
        assert!(!host.take_due_runtime_frame_wake(now));
        assert!(host.take_due_runtime_frame_wake(now + Duration::from_millis(25)));
        assert!(host.take_external_redraw().requires_frame_update());

        host.apply_runtime_frame_demand(EditorRuntimeFrameDemand::SleepUntil(Duration::MAX), now);
        assert_eq!(
            host.runtime_frame_wake_deadline(),
            Some(now + Duration::from_secs(60)),
            "an extreme transport delay must remain a bounded native wake"
        );
    }
}
