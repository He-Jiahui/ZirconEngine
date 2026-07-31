use std::time::{Duration, Instant};

use winit::event_loop::ControlFlow;
use zircon_runtime::platform::EventLoopPolicy;

use crate::entry::runtime_library::{RuntimeFrameDemand, MAX_HOST_RUNTIME_FRAME_DELAY};

const HEADLESS_FRAME_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeFrameCadenceMode {
    Continuous,
    Reactive,
    FixedInterval {
        interval: Duration,
        next_deadline: Instant,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::entry::runtime_entry_app) struct RuntimeFrameCadenceReport {
    pub(in crate::entry::runtime_entry_app) frame_requests: u64,
    pub(in crate::entry::runtime_entry_app) frame_pumps: u64,
    pub(in crate::entry::runtime_entry_app) idle_pumps_suppressed: u64,
    pub(in crate::entry::runtime_entry_app) redraw_requests: u64,
}

pub(in crate::entry::runtime_entry_app) struct RuntimeFrameCadence {
    policy: EventLoopPolicy,
    mode: RuntimeFrameCadenceMode,
    frame_requested: bool,
    runtime_deadline: Option<Instant>,
    report: RuntimeFrameCadenceReport,
}

impl RuntimeFrameCadence {
    pub(in crate::entry::runtime_entry_app) fn new(policy: EventLoopPolicy) -> Self {
        Self::new_at(policy, Instant::now())
    }

    fn new_at(policy: EventLoopPolicy, now: Instant) -> Self {
        let mode = match policy {
            EventLoopPolicy::Game | EventLoopPolicy::Continuous | EventLoopPolicy::Mobile => {
                RuntimeFrameCadenceMode::Continuous
            }
            EventLoopPolicy::DesktopApp => RuntimeFrameCadenceMode::Reactive,
            EventLoopPolicy::Headless => RuntimeFrameCadenceMode::FixedInterval {
                interval: HEADLESS_FRAME_INTERVAL,
                next_deadline: now + HEADLESS_FRAME_INTERVAL,
            },
        };
        Self {
            policy,
            mode,
            frame_requested: true,
            runtime_deadline: None,
            report: RuntimeFrameCadenceReport {
                frame_requests: 1,
                ..RuntimeFrameCadenceReport::default()
            },
        }
    }

    pub(in crate::entry::runtime_entry_app) fn request_frame(&mut self) {
        if self.mode == RuntimeFrameCadenceMode::Reactive {
            self.frame_requested = true;
        }
        self.report.frame_requests = self.report.frame_requests.saturating_add(1);
    }

    /// Applies the complete demand snapshot returned by one successful runtime tick.
    /// Returns true only when the host must issue a new coalesced proxy wake.
    pub(in crate::entry::runtime_entry_app) fn apply_runtime_demand(
        &mut self,
        now: Instant,
        demand: RuntimeFrameDemand,
    ) -> bool {
        if self.mode != RuntimeFrameCadenceMode::Reactive {
            return false;
        }
        match demand {
            RuntimeFrameDemand::Idle => {
                self.runtime_deadline = None;
                false
            }
            RuntimeFrameDemand::Immediate => {
                self.runtime_deadline = None;
                if self.frame_requested {
                    false
                } else {
                    self.request_frame();
                    true
                }
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
        } else {
            self.report.idle_pumps_suppressed = self.report.idle_pumps_suppressed.saturating_add(1);
        }
        should_pump
    }

    pub(in crate::entry::runtime_entry_app) fn record_redraw_request(&mut self) {
        self.report.redraw_requests = self.report.redraw_requests.saturating_add(1);
    }

    pub(in crate::entry::runtime_entry_app) fn control_flow(&self) -> ControlFlow {
        match self.mode {
            RuntimeFrameCadenceMode::Continuous => ControlFlow::Poll,
            RuntimeFrameCadenceMode::Reactive => self
                .runtime_deadline
                .map(ControlFlow::WaitUntil)
                .unwrap_or(ControlFlow::Wait),
            RuntimeFrameCadenceMode::FixedInterval { next_deadline, .. } => {
                ControlFlow::WaitUntil(next_deadline)
            }
        }
    }

    pub(in crate::entry::runtime_entry_app) fn policy(&self) -> EventLoopPolicy {
        self.policy
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

        assert_eq!(cadence.control_flow(), ControlFlow::Wait);
        assert!(cadence.take_frame_request(now));
        assert!(!cadence.take_frame_request(now));
        cadence.request_frame();
        cadence.request_frame();
        assert!(cadence.take_frame_request(now));
        assert!(!cadence.take_frame_request(now));

        assert_eq!(cadence.report().frame_requests, 3);
        assert_eq!(cadence.report().frame_pumps, 2);
        assert_eq!(cadence.report().idle_pumps_suppressed, 2);
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
    fn continuous_cadence_never_suppresses_frame_pumps() {
        for policy in [
            EventLoopPolicy::Game,
            EventLoopPolicy::Continuous,
            EventLoopPolicy::Mobile,
        ] {
            let mut cadence = RuntimeFrameCadence::new(policy);
            let now = Instant::now();
            assert!(cadence.take_frame_request(now));
            assert!(cadence.take_frame_request(now));
            assert_eq!(cadence.control_flow(), ControlFlow::Poll);
            assert_eq!(cadence.report().idle_pumps_suppressed, 0);
        }
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
