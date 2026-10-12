use std::sync::{Arc, Mutex, Weak};

use super::{BoxedRuntimeSceneSystem, BoxedSceneSystem};

/// Part of the original executable registry, never a World or plan authority.
/// Taking the payload releases the mutex before restoration or any box drop.
#[derive(Default)]
pub(super) struct ExecutableReturns {
    returned: Arc<Mutex<ReturnedSceneSystems>>,
}

#[derive(Default)]
pub(crate) struct ReturnedSceneSystems {
    pub(crate) native: Vec<ReturnedNativeSystem>,
    pub(crate) runtime: Vec<BoxedRuntimeSceneSystem>,
}

pub(crate) struct ReturnedNativeSystem {
    pub(crate) id: String,
    pub(crate) system: BoxedSceneSystem,
}

pub(crate) struct ExecutableReturnHandle {
    returned: Weak<Mutex<ReturnedSceneSystems>>,
}

impl ExecutableReturns {
    pub(super) fn handle(&self) -> ExecutableReturnHandle {
        ExecutableReturnHandle {
            returned: Arc::downgrade(&self.returned),
        }
    }

    pub(super) fn take(&mut self) -> ReturnedSceneSystems {
        let mut returned = self.returned.lock().unwrap_or_else(|p| p.into_inner());
        std::mem::take(&mut *returned)
    }

    pub(super) fn cover(&self, native_ids: &[String], runtime_count: usize) -> bool {
        let returned = self.returned.lock().unwrap_or_else(|p| p.into_inner());
        returned.native.len() == native_ids.len()
            && returned.runtime.len() == runtime_count
            && returned
                .native
                .iter()
                .all(|system| native_ids.contains(&system.id))
    }
}

impl ExecutableReturnHandle {
    pub(crate) fn native(&self, system: BoxedSceneSystem) {
        // Upgrade before locking. Destruction of the original registry drops
        // this box here without keeping a dead World alive.
        let id = system.id().to_string();
        if let Some(returned) = self.returned.upgrade() {
            returned
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .native
                .push(ReturnedNativeSystem { id, system });
        }
    }

    pub(crate) fn runtime(&self, system: BoxedRuntimeSceneSystem) {
        if let Some(returned) = self.returned.upgrade() {
            returned
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .runtime
                .push(system);
        }
    }
}
