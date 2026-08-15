use std::time::{Duration, Instant};

use winit::event_loop::ControlFlow;
use zircon_runtime::platform::EventLoopPolicy;

use crate::entry::runtime_library::{RuntimeFrameDemand, MAX_HOST_RUNTIME_FRAME_DELAY};

const HEADLESS_FRAME_INTERVAL: Duration = Duration::from_millis(16);
const INTERACTIVE_FRAME_INTERVAL: Duration = Duration::from_nanos(16_666_667);
const UNFOCUSED_GAME_FRAME_INTERVAL: Duration = INTERACTIVE_FRAME_INTERVAL;
const MOBILE_FOREGROUND_FRAME_INTERVAL: Duration = INTERACTIVE_FRAME_INTERVAL;
const BACKGROUND_FRAME_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeFrameCadenceMode {
    Continuous,
    Reactive,
    LowPower {
        interval: Duration,
        next_deadline: Instant,
    },
    FixedInterval {
        interval: Duration,
        next_deadline: Instant,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::entry::runtime_entry_app) struct RuntimeFrameCadenceReport {
    pub(in crate::entry::runtime_entry_app) frame_requests: u64,
    pub(in crate::entry::runtime_entry_app) frame_requests_accepted: u64,
    pub(in crate::entry::runtime_entry_app) frame_requests_coalesced: u64,
    pub(in crate::entry::runtime_entry_app) frame_requests_ignored: u64,
    pub(in crate::entry::runtime_entry_app) frame_pumps: u64,
    pub(in crate::entry::runtime_entry_app) idle_pumps_suppressed: u64,
    pub(in crate::entry::runtime_entry_app) redraw_requests: u64,
    pub(in crate::entry::runtime_entry_app) focus_transitions: u64,
    pub(in crate::entry::runtime_entry_app) occlusion_transitions: u64,
    pub(in crate::entry::runtime_entry_app) low_power_pumps: u64,
    pub(in crate::entry::runtime_entry_app) low_power_pumps_suppressed: u64,
}

pub(in crate::entry::runtime_entry_app) struct RuntimeFrameCadence {
    policy: EventLoopPolicy,
    mode: RuntimeFrameCadenceMode,
    window_focused: bool,
    window_occluded: bool,
    frame_requested: bool,
    runtime_deadline: Option<Instant>,
    report: RuntimeFrameCadenceReport,
}

impl RuntimeFrameCadence {
    pub(in crate::entry::runtime_entry_app) fn new(policy: EventLoopPolicy) -> Self {
        Self::new_at(policy, Instant::now())
    }

    pub(in crate::entry::runtime_entry_app) fn new_for_window(
        policy: EventLoopPolicy,
        window_focused: bool,
    ) -> Self {
        Self::new_at_with_window_state(policy, window_focused, false, Instant::now())
    }

    fn new_at(policy: EventLoopPolicy, now: Instant) -> Self {
        Self::new_at_with_window_state(policy, true, false, now)
    }

    fn new_at_with_window_state(
        policy: EventLoopPolicy,
        window_focused: bool,
        window_occluded: bool,
        now: Instant,
    ) -> Self {
        let mode = Self::mode_for_window_state(policy, window_focused, window_occluded, now);
        Self {
            policy,
            mode,
            window_focused,
            window_occluded,
            frame_requested: true,
            runtime_deadline: None,
            report: RuntimeFrameCadenceReport {
                frame_requests: 1,
                frame_requests_accepted: 1,
                ..RuntimeFrameCadenceReport::default()
            },
        }
    }

    fn mode_for_window_state(
        policy: EventLoopPolicy,
        window_focused: bool,
        window_occluded: bool,
        now: Instant,
    ) -> RuntimeFrameCadenceMode {
        let mode = match policy {
            EventLoopPolicy::Game if window_occluded => RuntimeFrameCadenceMode::LowPower {
                interval: BACKGROUND_FRAME_INTERVAL,
                next_deadline: now + BACKGROUND_FRAME_INTERVAL,
            },
            EventLoopPolicy::Game if !window_focused => RuntimeFrameCadenceMode::LowPower {
                interval: UNFOCUSED_GAME_FRAME_INTERVAL,
                next_deadline: now + UNFOCUSED_GAME_FRAME_INTERVAL,
            },
            EventLoopPolicy::Game | EventLoopPolicy::Continuous => {
                RuntimeFrameCadenceMode::Continuous
            }
            EventLoopPolicy::DesktopApp => RuntimeFrameCadenceMode::Reactive,
            EventLoopPolicy::Mobile => {
                let interval = if window_focused && !window_occluded {
                    MOBILE_FOREGROUND_FRAME_INTERVAL
                } else {
                    BACKGROUND_FRAME_INTERVAL
                };
                RuntimeFrameCadenceMode::LowPower {
                    interval,
                    next_deadline: now + interval,
                }
            }
            EventLoopPolicy::Headless => RuntimeFrameCadenceMode::FixedInterval {
                interval: HEADLESS_FRAME_INTERVAL,
                next_deadline: now + HEADLESS_FRAME_INTERVAL,
            },
        };
        mode
    }

    fn refresh_mode_at(&mut self, now: Instant) {
        let next = Self::mode_for_window_state(
            self.policy,
            self.window_focused,
            self.window_occluded,
            now,
        );
        let unchanged = match (&self.mode, &next) {
            (RuntimeFrameCadenceMode::Continuous, RuntimeFrameCadenceMode::Continuous)
            | (RuntimeFrameCadenceMode::Reactive, RuntimeFrameCadenceMode::Reactive) => true,
            (
                RuntimeFrameCadenceMode::LowPower {
                    interval: current, ..
                },
                RuntimeFrameCadenceMode::LowPower { interval: next, .. },
            )
            | (
                RuntimeFrameCadenceMode::FixedInterval {
                    interval: current, ..
                },
                RuntimeFrameCadenceMode::FixedInterval { interval: next, .. },
            ) => current == next,
            _ => false,
        };
        if !unchanged {
            self.mode = next;
            self.runtime_deadline = None;
        }
    }

    pub(in crate::entry::runtime_entry_app) fn request_frame(&mut self) -> bool {
        self.report.frame_requests = self.report.frame_requests.saturating_add(1);
        if matches!(
            self.mode,
            RuntimeFrameCadenceMode::Reactive | RuntimeFrameCadenceMode::LowPower { .. }
        ) {
            if self.frame_requested {
                self.report.frame_requests_coalesced =
                    self.report.frame_requests_coalesced.saturating_add(1);
                return false;
            }
            self.frame_requested = true;
            self.report.frame_requests_accepted =
                self.report.frame_requests_accepted.saturating_add(1);
            true
        } else {
            self.report.frame_requests_ignored =
                self.report.frame_requests_ignored.saturating_add(1);
            false
        }
    }

    /// Applies the complete demand snapshot returned by one successful runtime tick.
    /// Returns true only when the host must issue a new coalesced proxy wake.
    pub(in crate::entry::runtime_entry_app) fn apply_runtime_demand(
        &mut self,
        now: Instant,
        demand: RuntimeFrameDemand,
    ) -> bool {
        if !matches!(
            self.mode,
            RuntimeFrameCadenceMode::Reactive | RuntimeFrameCadenceMode::LowPower { .. }
        ) {
            return false;
        }
        match demand {
            RuntimeFrameDemand::Idle => {
                self.runtime_deadline = None;
                false
            }
            RuntimeFrameDemand::Immediate => {
                self.runtime_deadline = None;
                self.request_frame()
            }
            RuntimeFrameDemand::After(delay) => {
                let delay = delay.min(MAX_HOST_RUNTIME_FRAME_DELAY);
                self.runtime_deadline = Some(
                    now.checked_add(delay)
                        .or_else(|| now.checked_add(MAX_HOST_RUNTIME_FRAME_DELAY))
                        .unwrap_or(now),
                );
                false
            }
        }
    }

    pub(in crate::entry::runtime_entry_app) fn take_frame_request(&mut self, now: Instant) -> bool {
        let should_pump = match &mut self.mode {
            RuntimeFrameCadenceMode::Reactive => {
                if std::mem::take(&mut self.frame_requested) {
                    true
                } else if self
                    .runtime_deadline
                    .is_some_and(|deadline| now >= deadline)
                {
                    self.runtime_deadline = None;
                    true
                } else {
                    false
                }
            }
            RuntimeFrameCadenceMode::Continuous => true,
            RuntimeFrameCadenceMode::LowPower {
                interval,
                next_deadline,
            } => {
                let explicitly_requested = std::mem::take(&mut self.frame_requested);
                let runtime_deadline_due = self
                    .runtime_deadline
                    .is_some_and(|runtime_deadline| now >= runtime_deadline);
                if explicitly_requested || runtime_deadline_due || now >= *next_deadline {
                    if runtime_deadline_due {
                        self.runtime_deadline = None;
                    }
                    *next_deadline = now + *interval;
                    true
                } else {
                    false
                }
            }
            RuntimeFrameCadenceMode::FixedInterval {
                interval,
                next_deadline,
            } => {
                let initial_frame = std::mem::take(&mut self.frame_requested);
                if now >= *next_deadline {
                    *next_deadline = now + *interval;
                    true
                } else {
                    initial_frame
                }
            }
        };
        if should_pump {
            self.report.frame_pumps = self.report.frame_pumps.saturating_add(1);
            if matches!(self.mode, RuntimeFrameCadenceMode::LowPower { .. }) {
                self.report.low_power_pumps = self.report.low_power_pumps.saturating_add(1);
            }
        } else {
            self.report.idle_pumps_suppressed = self.report.idle_pumps_suppressed.saturating_add(1);
            if matches!(self.mode, RuntimeFrameCadenceMode::LowPower { .. }) {
                self.report.low_power_pumps_suppressed =
                    self.report.low_power_pumps_suppressed.saturating_add(1);
            }
        }
        should_pump
    }

    pub(in crate::entry::runtime_entry_app) fn record_redraw_request(&mut self) {
        self.report.redraw_requests = self.report.redraw_requests.saturating_add(1);
    }

    pub(in crate::entry::runtime_entry_app) fn control_flow(&self) -> ControlFlow {
        match self.mode {
            RuntimeFrameCadenceMode::Continuous => ControlFlow::Poll,
            RuntimeFrameCadenceMode::Reactive if self.frame_requested => ControlFlow::Poll,
            RuntimeFrameCadenceMode::Reactive => self
                .runtime_deadline
                .map(ControlFlow::WaitUntil)
                .unwrap_or(ControlFlow::Wait),
            RuntimeFrameCadenceMode::LowPower { .. } if self.frame_requested => ControlFlow::Poll,
            RuntimeFrameCadenceMode::LowPower { next_deadline, .. } => ControlFlow::WaitUntil(
                self.runtime_deadline
                    .map(|runtime_deadline| runtime_deadline.min(next_deadline))
                    .unwrap_or(next_deadline),
            ),
            RuntimeFrameCadenceMode::FixedInterval { next_deadline, .. } => {
                ControlFlow::WaitUntil(next_deadline)
            }
        }
    }

    pub(in crate::entry::runtime_entry_app) fn policy(&self) -> EventLoopPolicy {
        self.policy
    }

    pub(in crate::entry::runtime_entry_app) fn set_window_focused(
        &mut self,
        focused: bool,
    ) -> bool {
        self.set_window_focused_at(focused, Instant::now())
    }

    fn set_window_focused_at(&mut self, focused: bool, now: Instant) -> bool {
        if self.window_focused == focused {
            return false;
        }
        self.window_focused = focused;
        self.report.focus_transitions = self.report.focus_transitions.saturating_add(1);
        self.refresh_mode_at(now);
        true
    }

    pub(in crate::entry::runtime_entry_app) fn set_window_occluded(
        &mut self,
        occluded: bool,
    ) -> bool {
        self.set_window_occluded_at(occluded, Instant::now())
    }

    fn set_window_occluded_at(&mut self, occluded: bool, now: Instant) -> bool {
        if self.window_occluded == occluded {
            return false;
        }
        self.window_occluded = occluded;
        self.report.occlusion_transitions = self.report.occlusion_transitions.saturating_add(1);
        self.refresh_mode_at(now);
        true
    }

    pub(in crate::entry::runtime_entry_app) fn report(&self) -> RuntimeFrameCadenceReport {
        self.report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reactive_cadence_coalesces_requests_and_suppresses_idle_frames() {
        let mut cadence = RuntimeFrameCadence::new(EventLoopPolicy::DesktopApp);
        let now = Instant::now();

        assert_eq!(cadence.control_flow(), ControlFlow::Poll);
        assert!(cadence.take_frame_request(now));
        assert_eq!(cadence.control_flow(), ControlFlow::Wait);
        assert!(!cadence.take_frame_request(now));
        cadence.request_frame();
        cadence.request_frame();
        assert!(cadence.take_frame_request(now));
        assert!(!cadence.take_frame_request(now));

        assert_eq!(cadence.report().frame_requests, 3);
        assert_eq!(cadence.report().frame_requests_accepted, 2);
        assert_eq!(cadence.report().frame_requests_coalesced, 1);
        assert_eq!(cadence.report().frame_requests_ignored, 0);
        assert_eq!(cadence.report().frame_pumps, 2);
        assert_eq!(cadence.report().idle_pumps_suppressed, 2);
    }

    #[test]
    fn reactive_pending_request_survives_runtime_idle_until_next_pump() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
        assert!(cadence.take_frame_request(now));

        assert!(cadence.request_frame());
        assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Idle));
        assert_eq!(cadence.control_flow(), ControlFlow::Poll);
        assert!(cadence.take_frame_request(now));
        assert_eq!(cadence.control_flow(), ControlFlow::Wait);
    }

    #[test]
    fn reactive_pending_request_still_polls_when_runtime_immediate_coalesces() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
        assert!(cadence.take_frame_request(now));

        assert!(cadence.request_frame());
        assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
        assert_eq!(cadence.control_flow(), ControlFlow::Poll);
        assert!(cadence.take_frame_request(now));
        assert_eq!(cadence.control_flow(), ControlFlow::Wait);
        assert_eq!(cadence.report().frame_requests_coalesced, 1);
    }

    #[test]
    fn reactive_runtime_immediate_demand_coalesces_one_host_wake() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
        assert!(cadence.take_frame_request(now));

        assert!(cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
        assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
        assert!(cadence.take_frame_request(now));
        assert!(!cadence.take_frame_request(now));
    }

    #[test]
    fn reactive_runtime_after_replaces_and_idle_cancels_previous_deadline() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::DesktopApp, now);
        assert!(cadence.take_frame_request(now));

        assert!(!cadence
            .apply_runtime_demand(now, RuntimeFrameDemand::After(Duration::from_millis(40)),));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + Duration::from_millis(40))
        );

        assert!(!cadence
            .apply_runtime_demand(now, RuntimeFrameDemand::After(Duration::from_millis(80)),));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + Duration::from_millis(80)),
            "a new runtime snapshot must replace, not merge with, the prior deadline"
        );
        assert!(!cadence.take_frame_request(now + Duration::from_millis(79)));
        assert!(cadence.take_frame_request(now + Duration::from_millis(80)));

        assert!(!cadence.apply_runtime_demand(
            now + Duration::from_millis(80),
            RuntimeFrameDemand::After(Duration::from_millis(20)),
        ));
        assert!(!cadence
            .apply_runtime_demand(now + Duration::from_millis(80), RuntimeFrameDemand::Idle,));
        assert_eq!(cadence.control_flow(), ControlFlow::Wait);
    }

    #[test]
    fn continuous_cadence_does_not_schedule_extra_wakes_from_runtime_demand() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, now);

        assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
        assert!(!cadence
            .apply_runtime_demand(now, RuntimeFrameDemand::After(Duration::from_millis(40)),));
        assert_eq!(cadence.control_flow(), ControlFlow::Poll);
    }

    #[test]
    fn foreground_game_and_explicit_continuous_cadence_never_suppress_frame_pumps() {
        for policy in [EventLoopPolicy::Game, EventLoopPolicy::Continuous] {
            let mut cadence = RuntimeFrameCadence::new(policy);
            let now = Instant::now();
            assert!(cadence.take_frame_request(now));
            assert!(cadence.take_frame_request(now));
            assert_eq!(cadence.control_flow(), ControlFlow::Poll);
            assert_eq!(cadence.report().idle_pumps_suppressed, 0);
        }
    }

    #[test]
    fn game_cadence_throttles_unfocused_and_occluded_windows() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, now);
        assert!(cadence.take_frame_request(now));
        assert_eq!(cadence.control_flow(), ControlFlow::Poll);

        assert!(cadence.set_window_focused_at(false, now));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + UNFOCUSED_GAME_FRAME_INTERVAL)
        );
        assert!(!cadence.take_frame_request(now + UNFOCUSED_GAME_FRAME_INTERVAL / 2));
        let unchanged_deadline = cadence.control_flow();
        assert!(!cadence.set_window_focused_at(false, now + Duration::from_millis(1)));
        assert_eq!(cadence.control_flow(), unchanged_deadline);
        cadence.request_frame();
        assert!(cadence.take_frame_request(now + UNFOCUSED_GAME_FRAME_INTERVAL / 2));

        let occluded_at = now + UNFOCUSED_GAME_FRAME_INTERVAL;
        assert!(cadence.set_window_occluded_at(true, occluded_at));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(occluded_at + BACKGROUND_FRAME_INTERVAL)
        );
        assert!(cadence.set_window_focused_at(true, occluded_at));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(occluded_at + BACKGROUND_FRAME_INTERVAL),
            "occlusion remains authoritative even when focus returns"
        );
        assert!(cadence.set_window_occluded_at(false, occluded_at));
        assert_eq!(cadence.control_flow(), ControlFlow::Poll);

        assert_eq!(cadence.report().focus_transitions, 2);
        assert_eq!(cadence.report().occlusion_transitions, 2);
        assert_eq!(cadence.report().low_power_pumps, 1);
        assert_eq!(cadence.report().low_power_pumps_suppressed, 1);
    }

    #[test]
    fn mobile_cadence_has_explicit_foreground_and_background_limits() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Mobile, now);

        assert_eq!(cadence.control_flow(), ControlFlow::Poll);
        assert!(cadence.take_frame_request(now));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + MOBILE_FOREGROUND_FRAME_INTERVAL)
        );

        assert!(cadence.set_window_focused_at(false, now));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + BACKGROUND_FRAME_INTERVAL)
        );
    }

    #[test]
    fn explicit_continuous_profile_ignores_visibility_throttling() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Continuous, now);

        assert!(cadence.set_window_focused_at(false, now));
        assert!(cadence.set_window_occluded_at(true, now));

        assert!(cadence.take_frame_request(now));
        assert_eq!(cadence.control_flow(), ControlFlow::Poll);
    }

    #[test]
    fn low_power_cadence_consumes_runtime_immediate_and_after_demand() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Game, now);
        assert!(cadence.take_frame_request(now));
        assert!(cadence.set_window_focused_at(false, now));
        cadence.request_frame();
        assert!(cadence.take_frame_request(now));

        assert!(cadence.apply_runtime_demand(now, RuntimeFrameDemand::Immediate));
        assert!(cadence.take_frame_request(now));

        let runtime_delay = Duration::from_millis(5);
        assert!(!cadence.apply_runtime_demand(now, RuntimeFrameDemand::After(runtime_delay),));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + runtime_delay)
        );
        assert!(cadence.take_frame_request(now + runtime_delay));
        assert!(!cadence.apply_runtime_demand(now + runtime_delay, RuntimeFrameDemand::Idle,));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + runtime_delay + UNFOCUSED_GAME_FRAME_INTERVAL)
        );
    }

    #[test]
    fn headless_cadence_uses_fixed_wait_deadlines() {
        let now = Instant::now();
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Headless, now);

        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(now + HEADLESS_FRAME_INTERVAL)
        );
        assert!(cadence.take_frame_request(now));
    }

    #[test]
    fn headless_early_wake_does_not_pump_or_move_fixed_deadline() {
        let start = Instant::now();
        let deadline = start + HEADLESS_FRAME_INTERVAL;
        let mut cadence = RuntimeFrameCadence::new_at(EventLoopPolicy::Headless, start);

        assert!(cadence.take_frame_request(start));
        cadence.request_frame();
        assert!(!cadence.take_frame_request(start + HEADLESS_FRAME_INTERVAL / 2));
        assert_eq!(cadence.control_flow(), ControlFlow::WaitUntil(deadline));

        assert!(cadence.take_frame_request(deadline));
        assert_eq!(
            cadence.control_flow(),
            ControlFlow::WaitUntil(deadline + HEADLESS_FRAME_INTERVAL)
        );
    }
}
