use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use zircon_runtime_interface::{ZrRuntimeSessionHandle, ZrStatus};

use super::status::{invalid_argument, not_found};
use super::RuntimeDynamicSession;

static SESSION_REGISTRY: OnceLock<Mutex<SessionRegistry>> = OnceLock::new();

pub(super) struct SessionRegistry {
    next_handle: AtomicU64,
    pub(super) sessions: HashMap<u64, Arc<Mutex<RuntimeDynamicSession>>>,
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

pub(super) fn lock_registry() -> MutexGuard<'static, SessionRegistry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn lock_session(
    session: &Mutex<RuntimeDynamicSession>,
) -> MutexGuard<'_, RuntimeDynamicSession> {
    session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(super) fn insert_session(session: RuntimeDynamicSession) -> ZrRuntimeSessionHandle {
    let mut registry = lock_registry();
    let handle = registry.next_handle.fetch_add(1, Ordering::SeqCst);
    registry
        .sessions
        .insert(handle, Arc::new(Mutex::new(session)));
    ZrRuntimeSessionHandle::new(handle)
}

pub(super) fn with_session(
    handle: ZrRuntimeSessionHandle,
    action: impl FnOnce(&mut RuntimeDynamicSession) -> ZrStatus,
) -> ZrStatus {
    if !handle.is_valid() {
        return invalid_argument(b"invalid runtime session handle");
    }
    let session = {
        let registry = lock_registry();
        registry.sessions.get(&handle.raw()).cloned()
    };
    let Some(session) = session else {
        return not_found(b"runtime session not found");
    };
    let mut session = lock_session(session.as_ref());
    action(&mut session)
}
