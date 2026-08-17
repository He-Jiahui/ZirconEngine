use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use zircon_runtime_interface::{ZrRuntimeSessionHandle, ZrStatus};

use super::action_guard::SessionActionGuard;
use super::allocation_registry::{forget_session_census, session_has_outstanding_allocations};
use super::session_slot::SessionSlot;
use super::{RuntimeFrameActivity, RuntimeWakeRegistration};
use crate::dynamic_api::session::status::{invalid_argument, not_found, teardown_incomplete};
use crate::dynamic_api::session::RuntimeDynamicSession;

static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>> = OnceLock::new();

struct SessionRegistry {
    next_handle: AtomicU64,
    sessions: HashMap<u64, Arc<SessionSlot>>,
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self {
            next_handle: AtomicU64::new(1),
            sessions: HashMap::new(),
        }
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

pub(in crate::dynamic_api::session) fn insert_session(
    session: RuntimeDynamicSession,
) -> ZrRuntimeSessionHandle {
    insert_session_with_wake(session, RuntimeWakeRegistration::disabled())
}

pub(in crate::dynamic_api::session) fn insert_session_with_wake(
    session: RuntimeDynamicSession,
    wake: RuntimeWakeRegistration,
) -> ZrRuntimeSessionHandle {
    let mut registry = lock_registry();
    let handle = registry.next_handle.fetch_add(1, Ordering::SeqCst);
    registry
        .sessions
        .insert(handle, Arc::new(SessionSlot::new(session, wake)));
    ZrRuntimeSessionHandle::new(handle)
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
