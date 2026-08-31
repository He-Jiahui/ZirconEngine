use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use thiserror::Error;
use zircon_runtime_interface::{ZrRuntimeSessionHandle, ZrStatus};

use super::action_guard::SessionActionGuard;
use super::allocation_registry::{forget_session_census, session_has_outstanding_allocations};
use super::session_slot::SessionSlot;
use super::{RuntimeFrameActivity, RuntimeWakeRegistration};
use crate::dynamic_api::session::status::{invalid_argument, not_found, teardown_incomplete};
use crate::dynamic_api::session::RuntimeDynamicSession;

static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>> = OnceLock::new();

struct SessionRegistry {
    next_handle: u64,
    sessions: HashMap<u64, Arc<SessionSlot>>,
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(in crate::dynamic_api::session) enum SessionRegistryInsertError {
    #[error("runtime session handle space exhausted")]
    HandleSpaceExhausted,
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self {
            next_handle: 1,
            sessions: HashMap::new(),
        }
    }
}

impl SessionRegistry {
    fn try_allocate_handle(&mut self) -> Result<u64, SessionRegistryInsertError> {
        let handle = self.next_handle;
        if handle == 0 {
            return Err(SessionRegistryInsertError::HandleSpaceExhausted);
        }
        debug_assert!(!self.sessions.contains_key(&handle));
        // Zero is invalid and becomes the permanent exhausted state after u64::MAX.
        self.next_handle = handle.checked_add(1).unwrap_or(0);
        Ok(handle)
    }
}

fn registry() -> &'static Mutex<SessionRegistry> {
    SESSION_REGISTRY.get_or_init(|| Mutex::new(SessionRegistry::default()))
}

fn lock_registry() -> MutexGuard<'static, SessionRegistry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn poison_registry_lock_for_test() {
    let _registry = lock_registry();
    panic!("poison dynamic API session registry lock");
}

pub(in crate::dynamic_api::session) fn try_insert_session(
    session: RuntimeDynamicSession,
) -> Result<ZrRuntimeSessionHandle, SessionRegistryInsertError> {
    try_insert_session_with_wake(session, RuntimeWakeRegistration::disabled())
}

pub(in crate::dynamic_api::session) fn try_insert_session_with_wake(
    mut session: RuntimeDynamicSession,
    wake: RuntimeWakeRegistration,
) -> Result<ZrRuntimeSessionHandle, SessionRegistryInsertError> {
    let mut registry = lock_registry();
    let handle = match registry.try_allocate_handle() {
        Ok(handle) => handle,
        Err(error) => {
            drop(registry);
            if !session.shutdown_before_library_unload() {
                eprintln!(
                    "fatal dynamic runtime session handle allocation teardown failure; aborting before dynamic library unload"
                );
                std::process::abort();
            }
            return Err(error);
        }
    };
    registry
        .sessions
        .insert(handle, Arc::new(SessionSlot::new(session, wake)));
    Ok(ZrRuntimeSessionHandle::new(handle))
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn insert_session(
    session: RuntimeDynamicSession,
) -> ZrRuntimeSessionHandle {
    try_insert_session(session).expect("test runtime session handle")
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn insert_session_with_wake(
    session: RuntimeDynamicSession,
    wake: RuntimeWakeRegistration,
) -> ZrRuntimeSessionHandle {
    try_insert_session_with_wake(session, wake).expect("test runtime session handle")
}

pub(in crate::dynamic_api::session) fn with_session(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> ZrStatus,
) -> ZrStatus {
    with_session_activity(handle, |session, _activity| action(session))
}

pub(in crate::dynamic_api::session) fn with_session_activity(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> ZrStatus,
) -> ZrStatus {
    match with_session_activity_result(handle, |session, activity| Ok(action(session, activity))) {
        Ok(status) | Err(status) => status,
    }
}

pub(in crate::dynamic_api::session) fn with_session_result_finalized<T, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<T, ZrStatus>,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
) -> Result<U, ZrStatus> {
    with_session_activity_result_finalized(handle, |session, _activity| action(session), finalize)
}

pub(in crate::dynamic_api::session) fn with_session_result_committed<T, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<T, ZrStatus>,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
    commit: impl FnOnce(&mut RuntimeDynamicSession),
    rollback: impl FnOnce(&mut RuntimeDynamicSession),
) -> Result<U, ZrStatus> {
    let slot = find_session_slot(handle)?;
    let Some(action_guard) = slot.begin_action() else {
        return Err(not_found(b"runtime session not found"));
    };
    let value = {
        let mut session = slot.lock_session();
        let session = session
            .as_mut()
            .expect("an active runtime session action must retain its session");
        action(session)?
    };
    let finalized = match finalize(handle, value) {
        Ok(finalized) => finalized,
        Err(status) => {
            let mut session = slot.lock_session();
            let session = session
                .as_mut()
                .expect("an active runtime session rollback must retain its session");
            rollback(session);
            return Err(status);
        }
    };
    {
        let mut session = slot.lock_session();
        let session = session
            .as_mut()
            .expect("an active runtime session finalizer must retain its session");
        commit(session);
    }
    drop(action_guard);
    Ok(finalized)
}

fn with_session_activity_result<T>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> Result<T, ZrStatus>,
) -> Result<T, ZrStatus> {
    with_session_activity_result_finalized(handle, action, |_active_handle, value| Ok(value))
}

fn with_session_activity_result_finalized<T, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> Result<T, ZrStatus>,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
) -> Result<U, ZrStatus> {
    let slot = match find_session_slot(handle) {
        Ok(slot) => slot,
        Err(status) => return Err(status),
    };
    let Some(action_guard) = slot.begin_action() else {
        return Err(not_found(b"runtime session not found"));
    };
    let result = {
        let mut session = slot.lock_session();
        let Some(session) = session.as_mut() else {
            return Err(not_found(b"runtime session not found"));
        };
        action(session, slot.frame_activity())
    };
    let value = result?;
    let finalized = finalize(handle, value);
    drop(action_guard);
    finalized
}

pub(super) fn begin_session_action(
    handle: ZrRuntimeSessionHandle,
) -> Result<SessionActionGuard, ZrStatus> {
    let slot = find_session_slot(handle)?;
    slot.begin_action()
        .ok_or_else(|| not_found(b"runtime session not found"))
}

pub(super) fn begin_session_release_action(
    handle: ZrRuntimeSessionHandle,
) -> Result<SessionActionGuard, ZrStatus> {
    let slot = find_session_slot(handle)?;
    slot.begin_release_action()
        .ok_or_else(|| not_found(b"runtime session not found"))
}

pub(in crate::dynamic_api::session) fn destroy_session_slot(
    handle: ZrRuntimeSessionHandle,
) -> ZrStatus {
    let slot = match find_session_slot(handle) {
        Ok(slot) => slot,
        Err(status) => return status,
    };
    if slot
        .frame_activity()
        .wake_callback_active_on_current_thread()
    {
        return invalid_argument(b"runtime wake callback cannot destroy its session synchronously");
    }
    if !slot.begin_close() {
        return not_found(b"runtime session not found");
    }

    slot.frame_activity().disable_wake_entries();
    slot.wait_for_actions();
    slot.frame_activity().wait_for_wake_callbacks();
    if session_has_outstanding_allocations(handle) {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    let session_shutdown = slot
        .lock_session()
        .as_mut()
        .is_none_or(RuntimeDynamicSession::shutdown_before_library_unload);

    if !session_shutdown {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }

    drop(slot.take_session());

    let mut registry = lock_registry();
    if registry
        .sessions
        .get(&handle.raw())
        .is_some_and(|registered| Arc::ptr_eq(registered, &slot))
    {
        registry.sessions.remove(&handle.raw());
    }
    drop(registry);
    forget_session_census(handle);
    ZrStatus::ok()
}

fn find_session_slot(handle: ZrRuntimeSessionHandle) -> Result<Arc<SessionSlot>, ZrStatus> {
    if !handle.is_valid() {
        return Err(invalid_argument(b"invalid runtime session handle"));
    }
    let registry = lock_registry();
    registry
        .sessions
        .get(&handle.raw())
        .cloned()
        .ok_or_else(|| not_found(b"runtime session not found"))
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn session_is_closing(handle: ZrRuntimeSessionHandle) -> bool {
    find_session_slot(handle).is_ok_and(|slot| slot.is_closing())
}

#[cfg(test)]
mod handle_allocation_tests {
    use std::hint::black_box;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use super::{SessionRegistry, SessionRegistryInsertError};

    const PERFORMANCE_SAMPLE_PAIRS: usize = 21;
    const PERFORMANCE_ITERATIONS: usize = 250_000;
    const BASIS_POINTS_SCALE: u128 = 10_000;
    const PERFORMANCE_MAX_RATIO_BPS: u128 = 7_500;

    fn nearest_rank_percentile(samples: &[Duration], percentile: usize) -> Duration {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn measure_legacy_atomic_allocation() -> Duration {
        let next_handle = AtomicU64::new(1);
        let started = Instant::now();
        for _ in 0..PERFORMANCE_ITERATIONS {
            black_box(next_handle.fetch_add(1, Ordering::SeqCst));
        }
        started.elapsed()
    }

    fn measure_checked_allocation() -> Duration {
        let mut registry = SessionRegistry::default();
        let started = Instant::now();
        for _ in 0..PERFORMANCE_ITERATIONS {
            black_box(registry.try_allocate_handle().unwrap());
        }
        started.elapsed()
    }

    #[test]
    fn session_handle_allocation_exhausts_after_the_maximum_value_without_wrapping() {
        let mut registry = SessionRegistry::default();
        registry.next_handle = u64::MAX;

        assert_eq!(registry.try_allocate_handle().unwrap(), u64::MAX);
        assert_eq!(
            registry.try_allocate_handle(),
            Err(SessionRegistryInsertError::HandleSpaceExhausted)
        );
        assert_eq!(
            registry.try_allocate_handle(),
            Err(SessionRegistryInsertError::HandleSpaceExhausted)
        );
    }

    #[test]
    fn session_handle_allocation_rejects_zero_as_exhausted() {
        let mut registry = SessionRegistry::default();
        registry.next_handle = 0;

        assert_eq!(
            registry.try_allocate_handle(),
            Err(SessionRegistryInsertError::HandleSpaceExhausted)
        );
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn checked_session_handle_allocation_release_performance_acceptance() {
        black_box(measure_legacy_atomic_allocation());
        black_box(measure_checked_allocation());

        let mut legacy_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_PAIRS);
        let mut checked_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_PAIRS);
        for pair in 0..PERFORMANCE_SAMPLE_PAIRS {
            let (legacy, checked) = if pair % 2 == 0 {
                (
                    measure_legacy_atomic_allocation(),
                    measure_checked_allocation(),
                )
            } else {
                let checked = measure_checked_allocation();
                let legacy = measure_legacy_atomic_allocation();
                (legacy, checked)
            };
            legacy_samples.push(legacy);
            checked_samples.push(checked);
        }

        let legacy_p95 = nearest_rank_percentile(&legacy_samples, 95);
        let checked_p95 = nearest_rank_percentile(&checked_samples, 95);
        let ratio_bps = checked_p95.as_nanos() * BASIS_POINTS_SCALE / legacy_p95.as_nanos().max(1);
        let legacy_samples_ns = legacy_samples
            .iter()
            .map(|sample| sample.as_nanos().to_string())
            .collect::<Vec<_>>()
            .join(",");
        let checked_samples_ns = checked_samples
            .iter()
            .map(|sample| sample.as_nanos().to_string())
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "PERF-MVP-INTERFACE01-HANDLE: sample_pairs={PERFORMANCE_SAMPLE_PAIRS};order=alternating_legacy_first_even;iterations={PERFORMANCE_ITERATIONS};legacy_samples_ns={legacy_samples_ns};optimized_samples_ns={checked_samples_ns};legacy_p95_ns={};optimized_p95_ns={};ratio_bps={ratio_bps};threshold_bps={PERFORMANCE_MAX_RATIO_BPS}",
            legacy_p95.as_nanos(),
            checked_p95.as_nanos(),
        );

        assert!(
            checked_p95.as_nanos() * BASIS_POINTS_SCALE
                <= legacy_p95.as_nanos() * PERFORMANCE_MAX_RATIO_BPS,
            "checked handle allocation P95 {checked_p95:?} must be at least 25% faster than legacy SeqCst allocation P95 {legacy_p95:?}"
        );
    }
}
