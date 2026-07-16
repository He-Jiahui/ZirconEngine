use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::tray_state::SupervisionState;
use crate::TrayError;

pub const BACKOFF_SECONDS: [u64; 5] = [1, 2, 5, 15, 30];
pub const FAILURE_WINDOW_SECONDS: u64 = 10 * 60;
pub const HEALTHY_RESET_SECONDS: u64 = 10 * 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDecision {
    RetryAfter(u64),
    CircuitOpen,
    Suppressed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecoveryGuard {
    pub state: SupervisionState,
    pub explicit_stop: bool,
    pub maintenance_hold: bool,
    pub valid_competing_instance: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryStatus {
    pub failure_count: usize,
    pub failure_window_started_at: Option<u64>,
    pub next_retry_at: Option<u64>,
    pub circuit_open_until: Option<u64>,
    pub healthy_since: Option<u64>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct RecoveryController {
    policy: RecoveryPolicy,
    last_guard: Option<RecoveryGuard>,
    last_verified_instance_id: Option<String>,
    circuit_open: bool,
    explicit_stop_requested: bool,
    explicit_restart_requested: bool,
}

impl RecoveryController {
    pub fn load(path: &Path) -> Result<Self, TrayError> {
        if !path.is_file() && !journal_previous_path(path).is_file() {
            return Ok(Self::default());
        }
        match read_journal(path) {
            Ok(controller) => Ok(controller),
            Err(primary_error) => match read_journal(&journal_previous_path(path)) {
                Ok(controller) => Ok(controller),
                Err(_) => Err(primary_error),
            },
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), TrayError> {
        let parent = path.parent().ok_or_else(|| {
            TrayError::Http("recovery journal path has no parent directory".into())
        })?;
        fs::create_dir_all(parent)?;
        let next = journal_next_path(path);
        let previous = journal_previous_path(path);
        let encoded = serde_json::to_vec_pretty(self)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&next)?;
        file.write_all(&encoded)?;
        file.sync_all()?;
        drop(file);
        if path.is_file() {
            fs::remove_file(&previous).ok();
            fs::rename(path, &previous)?;
        }
        fs::rename(next, path)?;
        Ok(())
    }

    fn validate(&self) -> Result<(), TrayError> {
        let failures = &self.policy.failures;
        let invalid = || TrayError::RecoverySuppressed("recovery journal invariants are invalid");
        if failures.len() > BACKOFF_SECONDS.len()
            || failures
                .iter()
                .zip(failures.iter().skip(1))
                .any(|(left, right)| left > right)
            || failures
                .front()
                .zip(failures.back())
                .is_some_and(|(first, last)| last.saturating_sub(*first) > FAILURE_WINDOW_SECONDS)
            || (failures.is_empty()
                && self.last_guard.is_some()
                && self.policy.next_retry_at.is_some())
        {
            return Err(invalid());
        }
        if failures.is_empty() {
            if self.circuit_open
                || self.policy.circuit_opened_at.is_some()
                || self.policy.next_retry_at.is_some()
            {
                return Err(invalid());
            }
        } else if self.last_guard.is_none() {
            return Err(invalid());
        }
        let circuit_expected = failures.len() == BACKOFF_SECONDS.len();
        if self.circuit_open != circuit_expected
            || self.policy.circuit_opened_at.is_some() != circuit_expected
            || (circuit_expected && self.policy.circuit_opened_at != failures.back().copied())
        {
            return Err(invalid());
        }
        if let Some(deadline) = self.policy.next_retry_at {
            let Some(last_failure) = failures.back().copied() else {
                return Err(invalid());
            };
            if circuit_expected
                || self.policy.healthy_since.is_some()
                || deadline != last_failure.saturating_add(BACKOFF_SECONDS[failures.len() - 1])
            {
                return Err(invalid());
            }
        }
        if self.explicit_stop_requested && self.explicit_restart_requested {
            return Err(invalid());
        }
        if self.policy.healthy_since.is_some()
            && !self
                .last_guard
                .is_some_and(|guard| guard.state == SupervisionState::Healthy)
        {
            return Err(invalid());
        }
        Ok(())
    }

    pub fn status(&self) -> RecoveryStatus {
        RecoveryStatus {
            failure_count: self.policy.failures.len(),
            failure_window_started_at: self.policy.failures.front().copied(),
            next_retry_at: self.policy.next_retry_at,
            circuit_open_until: self
                .policy
                .circuit_opened_at
                .map(|opened| opened.saturating_add(HEALTHY_RESET_SECONDS)),
            healthy_since: self.policy.healthy_since,
        }
    }

    pub fn observe_online(&mut self, now_seconds: u64, guard: RecoveryGuard) -> bool {
        self.observe_online_with_instance(now_seconds, guard, false)
    }

    /// Records an online observation whose runtime descriptor, process identity, and
    /// authenticated health response have already been verified together.
    pub fn observe_verified_online(
        &mut self,
        now_seconds: u64,
        guard: RecoveryGuard,
        instance_id: &str,
    ) -> bool {
        let instance_changed = self.last_verified_instance_id.as_deref() != Some(instance_id);
        let mut changed = false;
        if instance_changed {
            self.last_verified_instance_id = Some(instance_id.to_owned());
            changed = true;
        }
        let observation_changed =
            self.observe_online_with_instance(now_seconds, guard, instance_changed);
        changed || observation_changed
    }

    fn observe_online_with_instance(
        &mut self,
        now_seconds: u64,
        guard: RecoveryGuard,
        verified_instance_changed: bool,
    ) -> bool {
        let mut changed = self.last_guard != Some(guard);
        self.last_guard = Some(guard);
        let invalidates_explicit_restart = guard.explicit_stop
            || guard.maintenance_hold
            || guard.valid_competing_instance
            || matches!(
                guard.state,
                SupervisionState::IdentityMismatch
                    | SupervisionState::FatalIntegrityError
                    | SupervisionState::ReadOnly
            );
        if invalidates_explicit_restart && self.explicit_restart_requested {
            self.explicit_restart_requested = false;
            changed = true;
        }
        if matches!(
            guard.state,
            SupervisionState::Healthy | SupervisionState::Degraded
        ) {
            changed |= self.explicit_stop_requested || self.explicit_restart_requested;
            self.explicit_stop_requested = false;
            self.explicit_restart_requested = false;
        }
        let replacement_is_safe = guard.state == SupervisionState::Healthy
            && !guard.explicit_stop
            && !guard.maintenance_hold
            && !guard.valid_competing_instance;
        if verified_instance_changed && replacement_is_safe {
            changed |= self.policy.clear_failures();
            changed |= self.circuit_open;
            self.circuit_open = false;
            return changed;
        }
        let (policy_changed, reset) = if guard.state == SupervisionState::Healthy {
            self.policy.observe_healthy(now_seconds)
        } else {
            (self.policy.observe_non_healthy(), false)
        };
        changed |= policy_changed;
        if reset {
            changed |= self.circuit_open;
            self.circuit_open = false;
        }
        changed
    }

    pub fn request_stop(&mut self) {
        self.explicit_stop_requested = true;
        self.explicit_restart_requested = false;
    }

    pub fn request_restart(&mut self) {
        self.explicit_restart_requested = true;
        self.explicit_stop_requested = false;
    }

    pub fn cancel_explicit_request(&mut self) {
        self.explicit_stop_requested = false;
        self.explicit_restart_requested = false;
    }

    pub fn retry_finished(&mut self) {
        self.policy.next_retry_at = None;
    }

    pub fn observe_offline(&mut self, now_seconds: u64, identity_safe: bool) -> RecoveryDecision {
        let Some(guard) = self.last_guard else {
            return RecoveryDecision::Suppressed;
        };
        let automatic_restart_allowed = RecoveryPolicy::auto_restart_allowed(
            guard.state,
            guard.explicit_stop,
            guard.maintenance_hold,
            guard.valid_competing_instance,
        );
        let explicit_restart_may_cross_stopping = self.explicit_restart_requested
            && guard.state == SupervisionState::Stopping
            && !guard.explicit_stop
            && !guard.maintenance_hold
            && !guard.valid_competing_instance;
        if !identity_safe
            || self.explicit_stop_requested
            || self.circuit_open
            || (!automatic_restart_allowed && !explicit_restart_may_cross_stopping)
        {
            return RecoveryDecision::Suppressed;
        }
        if let Some(deadline) = self.policy.next_retry_at {
            return RecoveryDecision::RetryAfter(deadline.saturating_sub(now_seconds));
        }
        let decision = self.policy.record_failure(now_seconds);
        if decision == RecoveryDecision::CircuitOpen {
            self.circuit_open = true;
        }
        decision
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct RecoveryPolicy {
    failures: VecDeque<u64>,
    healthy_since: Option<u64>,
    next_retry_at: Option<u64>,
    circuit_opened_at: Option<u64>,
}

impl RecoveryPolicy {
    fn clear_failures(&mut self) -> bool {
        let changed = !self.failures.is_empty()
            || self.healthy_since.is_some()
            || self.next_retry_at.is_some()
            || self.circuit_opened_at.is_some();
        self.failures.clear();
        self.healthy_since = None;
        self.next_retry_at = None;
        self.circuit_opened_at = None;
        changed
    }

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
            self.next_retry_at = None;
            self.circuit_opened_at = Some(now_seconds);
            RecoveryDecision::CircuitOpen
        } else {
            let delay = BACKOFF_SECONDS[self.failures.len() - 1];
            self.next_retry_at = Some(now_seconds.saturating_add(delay));
            RecoveryDecision::RetryAfter(delay)
        }
    }

    pub fn observe_healthy(&mut self, now_seconds: u64) -> (bool, bool) {
        let mut changed = self.next_retry_at.take().is_some();
        let since = match self.healthy_since {
            Some(since) => since,
            None => {
                self.healthy_since = Some(now_seconds);
                changed = true;
                now_seconds
            }
        };
        if now_seconds.saturating_sub(since) >= HEALTHY_RESET_SECONDS {
            let reset = !self.failures.is_empty() || self.circuit_opened_at.is_some();
            self.failures.clear();
            self.circuit_opened_at = None;
            (changed || reset, reset)
        } else {
            (changed, false)
        }
    }

    pub fn observe_non_healthy(&mut self) -> bool {
        self.healthy_since.take().is_some()
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

fn read_journal(path: &Path) -> Result<RecoveryController, TrayError> {
    let controller: RecoveryController = serde_json::from_slice(&fs::read(path)?)?;
    controller.validate()?;
    Ok(controller)
}

fn journal_next_path(path: &Path) -> PathBuf {
    path.with_extension("next.json")
}

fn journal_previous_path(path: &Path) -> PathBuf {
    path.with_extension("previous.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
        assert_eq!((true, false), policy.observe_healthy(10));
        assert_eq!((true, true), policy.observe_healthy(610));
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
        controller.retry_finished();
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
        controller.request_restart();
        controller.cancel_explicit_request();
        assert!(!controller.explicit_stop_requested);
        assert!(!controller.explicit_restart_requested);
    }

    #[test]
    fn explicit_restart_never_overrides_protective_supervision_guards() {
        for guard in [
            RecoveryGuard {
                state: SupervisionState::FatalIntegrityError,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
            RecoveryGuard {
                state: SupervisionState::IdentityMismatch,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
            RecoveryGuard {
                state: SupervisionState::ReadOnly,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
            RecoveryGuard {
                state: SupervisionState::Stopping,
                explicit_stop: false,
                maintenance_hold: true,
                valid_competing_instance: false,
            },
            RecoveryGuard {
                state: SupervisionState::Stopping,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: true,
            },
        ] {
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
            controller.observe_online(1, guard);

            assert_eq!(
                RecoveryDecision::Suppressed,
                controller.observe_offline(2, true),
                "protective guard {guard:?} must suppress restart",
            );
            assert!(!controller.explicit_restart_requested);
        }
    }

    #[test]
    fn recovery_journal_survives_tray_restart_and_preserves_open_circuit() {
        let root = std::env::temp_dir().join(format!(
            "zircon-tray-recovery-journal-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("tray-recovery.json");
        let mut controller = RecoveryController::default();
        controller.observe_online(
            1,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        for (index, now) in [2, 3, 5, 10, 25].into_iter().enumerate() {
            controller.observe_offline(now, true);
            if index < 4 {
                controller.retry_finished();
            }
        }

        controller.save(&path).unwrap();
        let mut restored = RecoveryController::load(&path).unwrap();

        assert_eq!(5, restored.status().failure_count);
        assert!(restored.status().circuit_open_until.is_some());
        assert_eq!(
            RecoveryDecision::Suppressed,
            restored.observe_offline(7, true)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupted_recovery_journal_fails_closed_instead_of_resetting_policy() {
        let root = std::env::temp_dir().join(format!(
            "zircon-tray-recovery-corrupt-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("tray-recovery.json");
        fs::write(&path, b"{not-json").unwrap();

        assert!(RecoveryController::load(&path).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parseable_but_incoherent_journal_falls_back_or_fails_closed() {
        let root = std::env::temp_dir().join(format!(
            "zircon-tray-recovery-incoherent-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("tray-recovery.json");
        let mut controller = RecoveryController::default();
        controller.observe_online(
            1,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        controller.observe_offline(2, true);
        controller.save(&path).unwrap();
        controller.retry_finished();
        controller.save(&path).unwrap();
        let mut invalid = serde_json::to_value(&controller).unwrap();
        invalid["explicit_stop_requested"] = true.into();
        invalid["explicit_restart_requested"] = true.into();
        fs::write(&path, serde_json::to_vec_pretty(&invalid).unwrap()).unwrap();

        let restored = RecoveryController::load(&path).unwrap();
        assert_eq!(1, restored.status().failure_count);
        fs::write(
            journal_previous_path(&path),
            br#"{"explicit_stop_requested":true,"explicit_restart_requested":true}"#,
        )
        .unwrap();
        assert!(RecoveryController::load(&path).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovery_status_tracks_retry_deadline_and_healthy_reset() {
        let mut controller = RecoveryController::default();
        controller.observe_online(
            10,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        assert_eq!(
            RecoveryDecision::RetryAfter(1),
            controller.observe_offline(20, true)
        );
        let failed = controller.status();
        assert_eq!(1, failed.failure_count);
        assert_eq!(Some(20), failed.failure_window_started_at);
        assert_eq!(Some(21), failed.next_retry_at);

        controller.observe_online(
            30,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        controller.observe_online(
            630,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        let cleared = controller.status();
        assert_eq!(0, cleared.failure_count);
        assert_eq!(None, cleared.failure_window_started_at);
        assert_eq!(None, cleared.next_retry_at);
    }

    #[test]
    fn restored_backoff_waits_without_counting_the_same_outage_twice() {
        let mut controller = RecoveryController::default();
        controller.observe_online(
            1,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        assert_eq!(
            RecoveryDecision::RetryAfter(1),
            controller.observe_offline(10, true)
        );
        let encoded = serde_json::to_vec(&controller).unwrap();
        let mut restored: RecoveryController = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(
            RecoveryDecision::RetryAfter(1),
            restored.observe_offline(10, true)
        );
        assert_eq!(1, restored.status().failure_count);
        restored.retry_finished();
        assert_eq!(
            RecoveryDecision::RetryAfter(2),
            restored.observe_offline(11, true)
        );
        assert_eq!(2, restored.status().failure_count);
    }

    #[test]
    fn read_only_online_state_does_not_advance_healthy_reset_window() {
        let mut controller = RecoveryController::default();
        controller.observe_online(
            1,
            RecoveryGuard {
                state: SupervisionState::Healthy,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        controller.observe_offline(2, true);
        controller.retry_finished();
        controller.observe_online(
            3,
            RecoveryGuard {
                state: SupervisionState::ReadOnly,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );
        controller.observe_online(
            700,
            RecoveryGuard {
                state: SupervisionState::ReadOnly,
                explicit_stop: false,
                maintenance_hold: false,
                valid_competing_instance: false,
            },
        );

        assert_eq!(1, controller.status().failure_count);
        assert_eq!(None, controller.status().healthy_since);
    }

    #[test]
    fn verified_replacement_immediately_clears_an_open_circuit() {
        let guard = RecoveryGuard {
            state: SupervisionState::Healthy,
            explicit_stop: false,
            maintenance_hold: false,
            valid_competing_instance: false,
        };
        let mut controller = RecoveryController::default();
        controller.observe_verified_online(0, guard, "old-instance");
        for (index, now) in [1, 2, 4, 8, 16].into_iter().enumerate() {
            assert_ne!(
                RecoveryDecision::Suppressed,
                controller.observe_offline(now, true)
            );
            if index < 4 {
                controller.retry_finished();
            }
        }
        assert_eq!(5, controller.status().failure_count);
        assert!(controller.status().circuit_open_until.is_some());

        controller.observe_verified_online(17, guard, "old-instance");
        assert_eq!(5, controller.status().failure_count);

        assert!(controller.observe_verified_online(18, guard, "new-instance"));
        assert_eq!(0, controller.status().failure_count);
        assert_eq!(None, controller.status().circuit_open_until);
    }
}
