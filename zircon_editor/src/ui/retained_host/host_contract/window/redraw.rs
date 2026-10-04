use std::time::{Duration, Instant};

use crate::core::gateway::EditorRuntimeFrameDemand;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::super::data::FrameRect;
use super::super::globals::UiHostContext;
use super::super::redraw::HostRedrawRequest;
use super::UiHostWindow;

const MAX_RUNTIME_FRAME_WAKE_DELAY: Duration = Duration::from_secs(60);
const RUNTIME_FRAME_FAILURE_RETRY_BASE_DELAY: Duration = Duration::from_millis(50);
const MAX_RUNTIME_FRAME_FAILURE_RETRY_DELAY: Duration = Duration::from_secs(1);

fn runtime_frame_failure_retry_delay(attempt: u32) -> Duration {
    let multiplier = 1_u32 << attempt.min(5);
    RUNTIME_FRAME_FAILURE_RETRY_BASE_DELAY
        .saturating_mul(multiplier)
        .min(MAX_RUNTIME_FRAME_FAILURE_RETRY_DELAY)
}

impl UiHostWindow {
    pub(crate) fn background_event_wake_callback(
        &self,
    ) -> zircon_runtime::core::framework::channel::ChannelWakeCallback {
        self.event_wake.callback()
    }

    pub(crate) fn background_visual_asset_wake_callback(
        &self,
    ) -> zircon_runtime::core::framework::channel::ChannelWakeCallback {
        self.visual_asset_wake.callback()
    }

    pub(in crate::ui::retained_host::host_contract) fn take_visual_asset_completion_wake(
        &self,
    ) -> bool {
        self.visual_asset_wake.take_request()
    }

    pub(in crate::ui::retained_host::host_contract) fn take_background_event_wake(&self) -> bool {
        self.event_wake.take_request()
    }

    /// Replaces the prior runtime request because each completed runtime tick owns the next wake.
    pub(crate) fn apply_runtime_frame_demand(
        &self,
        demand: EditorRuntimeFrameDemand,
        now: Instant,
    ) {
        let queue_immediate_frame = {
            let mut state = self.state.borrow_mut();
            state.runtime_frame_wake_tick_pending = false;
            state.runtime_frame_failure_retry_attempts = 0;
            match demand {
                EditorRuntimeFrameDemand::OnDemand => {
                    state.runtime_frame_wake_deadline = None;
                    false
                }
                EditorRuntimeFrameDemand::SleepUntil(delay) => {
                    let delay = delay.min(MAX_RUNTIME_FRAME_WAKE_DELAY);
                    state.runtime_frame_wake_deadline = Some(now.checked_add(delay).unwrap_or(now));
                    false
                }
                EditorRuntimeFrameDemand::Continuous => {
                    state.runtime_frame_wake_deadline = None;
                    true
                }
            }
        };
        if queue_immediate_frame {
            self.queue_external_redraw(HostRedrawRequest::full_frame());
        }
    }

    pub(crate) fn set_runtime_frame_owner(
        &self,
        owner: Option<(crate::core::play::PlayInstanceId, u64)>,
    ) {
        let mut state = self.state.borrow_mut();
        if state.runtime_frame_owner != owner {
            state.runtime_frame_owner = owner;
            state.runtime_frame_wake_deadline = None;
            state.runtime_frame_wake_tick_pending = false;
            state.runtime_frame_failure_retry_attempts = 0;
        }
    }

    pub(crate) fn complete_runtime_frame_tick<E>(
        &self,
        result: Result<EditorRuntimeFrameDemand, E>,
        owner_before_pump: Option<(crate::core::play::PlayInstanceId, u64)>,
        owner_after_pump: Option<(crate::core::play::PlayInstanceId, u64)>,
        now: Instant,
    ) -> Result<(), E> {
        self.set_runtime_frame_owner(owner_after_pump);
        match result {
            Ok(demand) => {
                if owner_before_pump == owner_after_pump {
                    self.apply_runtime_frame_demand(demand, now);
                }
                Ok(())
            }
            Err(error) => {
                let mut state = self.state.borrow_mut();
                if state.runtime_frame_wake_tick_pending {
                    state.runtime_frame_wake_tick_pending = false;
                    if state.runtime_frame_owner.is_some() {
                        let delay = runtime_frame_failure_retry_delay(
                            state.runtime_frame_failure_retry_attempts,
                        );
                        state.runtime_frame_failure_retry_attempts =
                            state.runtime_frame_failure_retry_attempts.saturating_add(1);
                        state.runtime_frame_wake_deadline = now.checked_add(delay);
                    }
                }
                Err(error)
            }
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
        let due = {
            let mut state = self.state.borrow_mut();
            let due = state
                .runtime_frame_wake_deadline
                .is_some_and(|deadline| deadline <= now);
            if due {
                state.runtime_frame_wake_deadline = None;
                state.runtime_frame_wake_tick_pending = true;
            }
            due
        };
        if due {
            self.queue_external_redraw(HostRedrawRequest::full_frame());
        }
        due
    }

    pub(crate) fn request_frame_update(&self) {
        self.global::<UiHostContext>().invoke_frame_requested();
    }

    pub(crate) fn request_interactive_frame_update(&self) {
        self.global::<UiHostContext>()
            .invoke_interactive_frame_requested();
    }

    pub(crate) fn request_maintenance_frame_update(&self) {
        self.state.borrow_mut().maintenance_frame_wake_deadline = None;
        self.queue_external_redraw(HostRedrawRequest::frame_update_only_for_scenario(
            UiPerfScenario::AssetRefresh,
        ));
    }

    pub(crate) fn schedule_maintenance_frame_update(&self, deadline: Instant) {
        self.state.borrow_mut().maintenance_frame_wake_deadline = Some(deadline);
    }

    pub(crate) fn clear_maintenance_frame_update(&self) {
        self.state.borrow_mut().maintenance_frame_wake_deadline = None;
    }

    /// Replaces the input-manager wake because each pointer observation owns its next deadline.
    pub(crate) fn set_input_timer_frame_update(&self, deadline: Option<Instant>) {
        self.state.borrow_mut().input_timer_frame_wake_deadline = deadline;
    }

    /// Schedules one retained-host lifecycle tick without borrowing the asset-refresh wake slot.
    ///
    /// Lifecycle work must stay independently schedulable while the native window is idle; asset
    /// refresh owns and may clear its separate maintenance deadline on every tick.
    pub(crate) fn set_lifecycle_frame_update(&self, deadline: Option<Instant>) {
        self.state.borrow_mut().lifecycle_frame_wake_deadline = deadline;
    }

    pub(in crate::ui::retained_host::host_contract) fn maintenance_frame_wake_deadline(
        &self,
    ) -> Option<Instant> {
        self.state.borrow().maintenance_frame_wake_deadline
    }

    pub(in crate::ui::retained_host::host_contract) fn lifecycle_frame_wake_deadline(
        &self,
    ) -> Option<Instant> {
        self.state.borrow().lifecycle_frame_wake_deadline
    }

    pub(in crate::ui::retained_host::host_contract) fn input_timer_frame_wake_deadline(
        &self,
    ) -> Option<Instant> {
        self.state.borrow().input_timer_frame_wake_deadline
    }

    pub(in crate::ui::retained_host::host_contract) fn take_due_maintenance_frame_wake(
        &self,
        now: Instant,
    ) -> bool {
        let due = self
            .state
            .borrow()
            .maintenance_frame_wake_deadline
            .is_some_and(|deadline| deadline <= now);
        if due {
            self.state.borrow_mut().maintenance_frame_wake_deadline = None;
            self.queue_external_redraw(HostRedrawRequest::frame_update_only_for_scenario(
                UiPerfScenario::AssetRefresh,
            ));
        }
        due
    }

    pub(in crate::ui::retained_host::host_contract) fn take_due_lifecycle_frame_wake(
        &self,
        now: Instant,
    ) -> bool {
        let due = self
            .state
            .borrow()
            .lifecycle_frame_wake_deadline
            .is_some_and(|deadline| deadline <= now);
        if due {
            self.state.borrow_mut().lifecycle_frame_wake_deadline = None;
            self.queue_external_redraw(HostRedrawRequest::frame_update_only_for_scenario(
                UiPerfScenario::SessionHeartbeat,
            ));
        }
        due
    }

    pub(in crate::ui::retained_host::host_contract) fn take_due_input_timer_frame_wake(
        &self,
        now: Instant,
    ) -> bool {
        let due = self
            .state
            .borrow()
            .input_timer_frame_wake_deadline
            .is_some_and(|deadline| deadline <= now);
        if due {
            self.state.borrow_mut().input_timer_frame_wake_deadline = None;
            self.queue_external_redraw(HostRedrawRequest::frame_update_only_for_scenario(
                UiPerfScenario::IdleHover,
            ));
        }
        due
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

    #[cfg(test)]
    pub(crate) fn take_external_redraw_for_test(&self) -> HostRedrawRequest {
        self.take_external_redraw()
    }
}

#[cfg(test)]
#[path = "tests/redraw.rs"]
mod tests;
