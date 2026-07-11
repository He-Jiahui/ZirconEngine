use std::collections::VecDeque;

use crate::tray_state::SupervisionState;

pub const BACKOFF_SECONDS: [u64; 5] = [1, 2, 5, 15, 30];
pub const FAILURE_WINDOW_SECONDS: u64 = 10 * 60;
pub const HEALTHY_RESET_SECONDS: u64 = 10 * 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDecision {
    RetryAfter(u64),
    CircuitOpen,
    Suppressed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryGuard {
    pub state: SupervisionState,
    pub explicit_stop: bool,
    pub maintenance_hold: bool,
    pub valid_competing_instance: bool,
}

#[derive(Default)]
pub struct RecoveryController {
    policy: RecoveryPolicy,
    last_guard: Option<RecoveryGuard>,
    circuit_open: bool,
    explicit_stop_requested: bool,
    explicit_restart_requested: bool,
}

impl RecoveryController {
    pub fn observe_online(&mut self, now_seconds: u64, guard: RecoveryGuard) {
        self.last_guard = Some(guard);
        if matches!(
            guard.state,
            SupervisionState::Healthy | SupervisionState::Degraded
        ) {
            self.explicit_stop_requested = false;
            self.explicit_restart_requested = false;
        }
        if self.policy.observe_healthy(now_seconds) {
            self.circuit_open = false;
        }
    }

    pub fn request_stop(&mut self) {
        self.explicit_stop_requested = true;
        self.explicit_restart_requested = false;
    }

    pub fn request_restart(&mut self) {
        self.explicit_restart_requested = true;
        self.explicit_stop_requested = false;
    }

    pub fn observe_offline(&mut self, now_seconds: u64, identity_safe: bool) -> RecoveryDecision {
        let Some(guard) = self.last_guard else {
            return RecoveryDecision::Suppressed;
        };
        if !identity_safe
            || self.explicit_stop_requested
            || self.circuit_open
            || (!self.explicit_restart_requested
                && !RecoveryPolicy::auto_restart_allowed(
                    guard.state,
                    guard.explicit_stop,
                    guard.maintenance_hold,
                    guard.valid_competing_instance,
                ))
        {
            return RecoveryDecision::Suppressed;
        }
        let decision = self.policy.record_failure(now_seconds);
        if decision == RecoveryDecision::CircuitOpen {
            self.circuit_open = true;
        }
        decision
    }
}

#[derive(Default)]
pub struct RecoveryPolicy {
    failures: VecDeque<u64>,
    healthy_since: Option<u64>,
}

impl RecoveryPolicy {
    pub fn record_failure(&mut self, now_seconds: u64) -> RecoveryDecision {
        while self
            .failures
            .front()
            .is_some_and(|value| now_seconds.saturating_sub(*value) > FAILURE_WINDOW_SECONDS)
        {
            self.failures.pop_front();
        }
        self.failures.push_back(now_seconds);
        self.healthy_since = None;
        if self.failures.len() >= 5 {
            RecoveryDecision::CircuitOpen
        } else {
            RecoveryDecision::RetryAfter(BACKOFF_SECONDS[self.failures.len() - 1])
        }
    }

    pub fn observe_healthy(&mut self, now_seconds: u64) -> bool {
        let since = self.healthy_since.get_or_insert(now_seconds);
        if now_seconds.saturating_sub(*since) >= HEALTHY_RESET_SECONDS {
            self.failures.clear();
            true
        } else {
            false
        }
    }

    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    pub fn auto_restart_allowed(
        state: SupervisionState,
        explicit_stop: bool,
        maintenance_hold: bool,
        valid_competing_instance: bool,
    ) -> bool {
        !explicit_stop
            && !maintenance_hold
            && !valid_competing_instance
            && !matches!(
                state,
                SupervisionState::IdentityMismatch
                    | SupervisionState::FatalIntegrityError
                    | SupervisionState::ReadOnly
                    | SupervisionState::Stopping
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifth_failure_in_ten_minutes_opens_circuit() {
        let mut policy = RecoveryPolicy::default();
        assert_eq!(RecoveryDecision::RetryAfter(1), policy.record_failure(0));
        assert_eq!(RecoveryDecision::RetryAfter(2), policy.record_failure(1));
        assert_eq!(RecoveryDecision::RetryAfter(5), policy.record_failure(2));
        assert_eq!(RecoveryDecision::RetryAfter(15), policy.record_failure(3));
        assert_eq!(RecoveryDecision::CircuitOpen, policy.record_failure(4));
    }

    #[test]
    fn ten_healthy_minutes_clear_failures() {
        let mut policy = RecoveryPolicy::default();
        policy.record_failure(1);
        assert!(!policy.observe_healthy(10));
        assert!(policy.observe_healthy(610));
        assert_eq!(0, policy.failure_count());
    }

    #[test]
    fn explicit_stop_and_integrity_failures_never_auto_restart() {
        assert!(!RecoveryPolicy::auto_restart_allowed(
            SupervisionState::Offline,
            true,
            false,
            false,
        ));
        assert!(!RecoveryPolicy::auto_restart_allowed(
            SupervisionState::FatalIntegrityError,
            false,
            false,
            false,
        ));
    }

    #[test]
    fn first_offline_observation_cannot_guess_that_restart_is_safe() {
        let mut controller = RecoveryController::default();
        assert_eq!(
            RecoveryDecision::Suppressed,
            controller.observe_offline(1, true)
        );
    }

    #[test]
    fn healthy_then_unexpected_offline_uses_bounded_backoff() {
        let mut controller = RecoveryController::default();
        controller.observe_online(
            0,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        assert_eq!(
            RecoveryDecision::RetryAfter(1),
            controller.observe_offline(1, true)
        );
        assert_eq!(
            RecoveryDecision::RetryAfter(2),
            controller.observe_offline(2, true)
        );
    }

    #[test]
    fn explicit_restart_overrides_stopping_guard_but_explicit_stop_does_not() {
        let mut controller = RecoveryController::default();
        controller.observe_online(
            0,
            RecoveryGuard {
                state: SupervisionState::Stopping,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        controller.request_restart();
        assert_eq!(
            RecoveryDecision::RetryAfter(1),
            controller.observe_offline(1, true)
        );
        controller.request_stop();
        assert_eq!(
            RecoveryDecision::Suppressed,
            controller.observe_offline(2, true)
        );
    }
}
