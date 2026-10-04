use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError};
use std::time::{Duration, Instant};

use thiserror::Error;
use zircon_runtime_interface::{ZrRuntimeSessionHandle, ZrStatus};

use super::action_guard::SessionActionGuard;
use super::allocation_registry::{
    session_has_outstanding_allocations_until, try_lock_registry_for_destroy,
};
use super::session_owner::runtime::RuntimeSessionOwnerCreateError;
use super::session_slot::SessionSlot;
use super::{RuntimeFrameActivity, RuntimeWakeRegistration};
use crate::diagnostic_log::write_log;
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::status::{invalid_argument, not_found, teardown_incomplete};
use crate::dynamic_api::session::RuntimeDynamicSession;
use crate::plugin::RuntimePluginRegistrationReport;

static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>> = OnceLock::new();
const DYNAMIC_SESSION_ACTION_WAKE_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
const DYNAMIC_SESSION_OWNED_COMMAND_ADMISSION_TIMEOUT: Duration = Duration::from_secs(5);

struct SessionRegistry {
    next_handle: u64,
    sessions: HashMap<u64, Arc<SessionSlot>>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(in crate::dynamic_api::session) enum SessionRegistryInsertError {
    #[error("runtime session handle space exhausted")]
    HandleSpaceExhausted,
    #[error("runtime session owner unavailable: {0}")]
    OwnerUnavailable(String),
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

fn lock_registry_until(deadline: Instant) -> Option<MutexGuard<'static, SessionRegistry>> {
    loop {
        if Instant::now() >= deadline {
            return None;
        }
        let guard = match registry().try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => {
                std::thread::yield_now();
                continue;
            }
        };
        return (Instant::now() < deadline).then_some(guard);
    }
}

#[cfg(test)]
pub(super) fn with_registry_lock_for_test(action: impl FnOnce()) {
    let _registry = lock_registry();
    action();
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn session_handle_is_published_for_test(
    handle: ZrRuntimeSessionHandle,
) -> bool {
    lock_registry().sessions.contains_key(&handle.raw())
}

#[cfg(test)]
pub(super) fn session_slot_for_test(handle: ZrRuntimeSessionHandle) -> Option<Arc<SessionSlot>> {
    find_session_slot(handle).ok()
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn poison_registry_lock_for_test() {
    let _registry = lock_registry();
    panic!("poison dynamic API session registry lock");
}

#[cfg(test)]
pub(in crate::dynamic_api::session) fn try_insert_session(
    session: RuntimeDynamicSession,
) -> Result<ZrRuntimeSessionHandle, SessionRegistryInsertError> {
    try_insert_session_with_wake(session, RuntimeWakeRegistration::disabled())
}

#[cfg(test)]
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
    let slot = SessionSlot::new(session, wake)
        .map(Arc::new)
        .map_err(SessionRegistryInsertError::OwnerUnavailable)?;
    registry.sessions.insert(handle, slot);
    Ok(ZrRuntimeSessionHandle::new(handle))
}

pub(in crate::dynamic_api::session) fn try_create_linked_session(
    profile: RuntimeDynamicSessionProfile,
    project: Option<RuntimeProjectConfig>,
    registrations: Vec<RuntimePluginRegistrationReport>,
) -> Result<ZrRuntimeSessionHandle, SessionRegistryInsertError> {
    let handle = lock_registry().try_allocate_handle()?;
    #[cfg(test)]
    super::session_owner::runtime::startup_observation::record_allocated_handle(handle);
    let slot = SessionSlot::create_with_linked_plugins(profile, project, registrations).map_err(
        |error| match error {
            RuntimeSessionOwnerCreateError::Owner(message) => {
                SessionRegistryInsertError::OwnerUnavailable(message)
            }
            RuntimeSessionOwnerCreateError::Session(status) => {
                SessionRegistryInsertError::OwnerUnavailable(status.into_message())
            }
        },
    )?;
    lock_registry().sessions.insert(handle, Arc::new(slot));
    Ok(ZrRuntimeSessionHandle::new(handle))
}

pub(in crate::dynamic_api::session) fn try_create_session_with_wake(
    profile: RuntimeDynamicSessionProfile,
    project: Option<RuntimeProjectConfig>,
    wake: RuntimeWakeRegistration,
) -> Result<ZrRuntimeSessionHandle, ZrStatus> {
    let handle = lock_registry().try_allocate_handle().map_err(|_| {
        crate::dynamic_api::session::status::limit_exceeded(
            b"runtime session handle space exhausted",
        )
    })?;
    #[cfg(test)]
    super::session_owner::runtime::startup_observation::record_allocated_handle(handle);
    let slot = Arc::new(SessionSlot::create(profile, project, wake)?);
    lock_registry().sessions.insert(handle, slot);
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
    action: impl FnOnce(&mut RuntimeDynamicSession) -> ZrStatus + Send,
) -> ZrStatus {
    with_session_activity(handle, |session, _activity| action(session))
}

pub(in crate::dynamic_api::session) fn with_session_owned(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> ZrStatus + Send + 'static,
) -> ZrStatus {
    let slot = match find_session_slot(handle) {
        Ok(slot) => slot,
        Err(status) => return status,
    };
    let Some(action_guard) = slot.begin_action() else {
        return not_found(b"runtime session not found");
    };
    let admission_deadline = Instant::now()
        .checked_add(DYNAMIC_SESSION_OWNED_COMMAND_ADMISSION_TIMEOUT)
        .unwrap_or_else(Instant::now);
    let status = match slot.dispatch_owned(admission_deadline, move |session| {
        let status = action(session);
        if status.is_ok() {
            Ok(())
        } else {
            Err(status)
        }
    }) {
        Ok(result) => match result.recv() {
            Ok(Ok(())) => ZrStatus::ok(),
            Ok(Err(error)) => error.into_abi(),
            Err(_) => not_found(b"runtime session not found"),
        },
        Err(super::session_owner::OwnerDispatchError::Reentrant) => {
            invalid_argument(b"runtime session callback cannot reenter synchronously")
        }
        Err(super::session_owner::OwnerDispatchError::AdmissionIncomplete) => {
            crate::dynamic_api::session::status::error_status(
                "runtime session owner command admission incomplete",
            )
        }
        Err(_) => not_found(b"runtime session not found"),
    };
    drop(action_guard);
    status
}

pub(in crate::dynamic_api::session) fn with_session_activity(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> ZrStatus + Send,
) -> ZrStatus {
    match with_session_activity_result(handle, |session, activity| {
        let status = action(session, activity);
        if status.is_ok() {
            Ok(())
        } else {
            Err(status)
        }
    }) {
        Ok(()) => ZrStatus::ok(),
        Err(status) => status,
    }
}

pub(in crate::dynamic_api::session) fn with_session_result_finalized<T: Send, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<T, ZrStatus> + Send,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
) -> Result<U, ZrStatus> {
    with_session_activity_result_finalized(handle, |session, _activity| action(session), finalize)
}

pub(in crate::dynamic_api::session) fn with_session_result_committed<T: Send, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<T, ZrStatus> + Send,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
    commit: impl FnOnce(&mut RuntimeDynamicSession) + Send,
    rollback: impl FnOnce(&mut RuntimeDynamicSession) + Send,
) -> Result<U, ZrStatus> {
    let slot = find_session_slot(handle)?;
    let Some(action_guard) = slot.begin_action() else {
        return Err(not_found(b"runtime session not found"));
    };
    let value = dispatch_session_action(&slot, action)?;
    let finalized = match finalize(handle, value) {
        Ok(finalized) => finalized,
        Err(status) => {
            dispatch_session_action(&slot, move |session| {
                rollback(session);
                Ok(())
            })?;
            return Err(status);
        }
    };
    dispatch_session_action(&slot, move |session| {
        commit(session);
        Ok(())
    })?;
    drop(action_guard);
    Ok(finalized)
}

fn with_session_activity_result<T: Send>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> Result<T, ZrStatus> + Send,
) -> Result<T, ZrStatus> {
    with_session_activity_result_finalized(handle, action, |_active_handle, value| Ok(value))
}

pub(in crate::dynamic_api::session) fn with_session_activity_result_finalized<T: Send, U>(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession, &RuntimeFrameActivity) -> Result<T, ZrStatus> + Send,
    finalize: impl FnOnce(ZrRuntimeSessionHandle, T) -> Result<U, ZrStatus>,
) -> Result<U, ZrStatus> {
    let slot = match find_session_slot(handle) {
        Ok(slot) => slot,
        Err(status) => return Err(status),
    };
    let Some(action_guard) = slot.begin_action() else {
        return Err(not_found(b"runtime session not found"));
    };
    let activity = slot.frame_activity();
    let value = dispatch_session_action(&slot, move |session| action(session, activity))?;
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
    destroy_session_slot_with_timeout(handle, DYNAMIC_SESSION_ACTION_WAKE_DRAIN_TIMEOUT)
}

pub(super) fn destroy_session_slot_with_timeout(
    handle: ZrRuntimeSessionHandle,
    timeout: Duration,
) -> ZrStatus {
    let destroy_started_at = Instant::now();
    let Some(destroy_deadline) = destroy_started_at.checked_add(timeout) else {
        return teardown_incomplete();
    };
    write_log("runtime_session", "destroy_find_slot_start");
    let slot = match find_session_slot_until(handle, destroy_deadline) {
        Ok(slot) => slot,
        Err(status) => return status,
    };
    write_log("runtime_session", "destroy_find_slot_done");
    if slot
        .frame_activity()
        .wake_callback_active_on_current_thread()
    {
        return invalid_argument(b"runtime wake callback cannot destroy its session synchronously");
    }
    write_log("runtime_session", "destroy_begin_close_start");
    if !slot.begin_close() {
        return not_found(b"runtime session not found");
    }
    write_log("runtime_session", "destroy_begin_close_done");

    slot.frame_activity().disable_wake_entries();
    write_log("runtime_session", "destroy_wait_actions_start");
    if !slot.wait_for_actions(destroy_deadline.saturating_duration_since(Instant::now())) {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_wait_actions_done");
    write_log("runtime_session", "destroy_wait_wake_callbacks_start");
    if !slot
        .frame_activity()
        .wait_for_wake_callbacks(destroy_deadline.saturating_duration_since(Instant::now()))
    {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_wait_wake_callbacks_done");
    write_log("runtime_session", "destroy_allocation_check_start");
    let Some(has_outstanding_allocations) =
        session_has_outstanding_allocations_until(handle, destroy_deadline)
    else {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    };
    if has_outstanding_allocations {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_allocation_check_done");
    write_log("runtime_session", "destroy_shutdown_start");
    let session_shutdown = matches!(
        slot.shutdown_until(destroy_deadline),
        super::session_owner::OwnerShutdownReceipt::Joined
    );
    write_log("runtime_session", "destroy_shutdown_done");

    if !session_shutdown {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }

    write_log("runtime_session", "destroy_registry_remove_start");
    let removed = remove_session_and_census_until(handle, &slot, destroy_deadline);
    if !removed {
        slot.preserve_failed_teardown_for_retry();
        return teardown_incomplete();
    }
    write_log("runtime_session", "destroy_registry_remove_done");
    ZrStatus::ok()
}

pub(super) fn remove_session_and_census_until(
    handle: ZrRuntimeSessionHandle,
    slot: &Arc<SessionSlot>,
    destroy_deadline: Instant,
) -> bool {
    let removed = loop {
        let Some(mut registry) = lock_registry_until(destroy_deadline) else {
            break false;
        };
        let Some(mut allocations) = try_lock_registry_for_destroy() else {
            drop(registry);
            if Instant::now() >= destroy_deadline {
                break false;
            }
            std::thread::yield_now();
            continue;
        };
        if Instant::now() >= destroy_deadline {
            break false;
        }
        match registry.sessions.get(&handle.raw()) {
            Some(registered) if Arc::ptr_eq(registered, &slot) => {}
            None if !allocations.has_census(handle) => break true,
            _ => break false,
        }
        if allocations.has_outstanding_allocations(handle) || Instant::now() >= destroy_deadline {
            break false;
        }
        registry.sessions.remove(&handle.raw());
        allocations.forget_empty_census(handle);
        break true;
    };
    removed
}

fn dispatch_session_action<R: Send>(
    slot: &SessionSlot,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send,
) -> Result<R, ZrStatus> {
    match slot.dispatch_scoped(action) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(error.into_abi()),
        Err(super::session_owner::OwnerDispatchError::Reentrant) => Err(invalid_argument(
            b"runtime session callback cannot reenter synchronously",
        )),
        Err(_) => Err(not_found(b"runtime session not found")),
    }
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

fn find_session_slot_until(
    handle: ZrRuntimeSessionHandle,
    deadline: Instant,
) -> Result<Arc<SessionSlot>, ZrStatus> {
    if !handle.is_valid() {
        return Err(invalid_argument(b"invalid runtime session handle"));
    }
    let registry = lock_registry_until(deadline).ok_or_else(teardown_incomplete)?;
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
pub(super) fn session_active_actions(handle: ZrRuntimeSessionHandle) -> Option<usize> {
    find_session_slot(handle)
        .ok()
        .map(|slot| slot.active_actions())
}

#[cfg(test)]
#[path = "tests/session_store_handle_allocation_tests.rs"]
mod handle_allocation_tests;
