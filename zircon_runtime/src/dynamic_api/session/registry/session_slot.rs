use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use super::action_guard::SessionActionGuard;
use super::{RuntimeFrameActivity, RuntimeWakeRegistration};
use crate::dynamic_api::session::RuntimeDynamicSession;

pub(in crate::dynamic_api::session) struct SessionSlot {
    session: Mutex<Option<RuntimeDynamicSession>>,
    lifecycle: Mutex<SessionSlotLifecycle>,
    actions_drained: Condvar,
    frame_activity: RuntimeFrameActivity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionSlotPhase {
    Open,
    Closing,
    TeardownRetryPending,
}

#[derive(Debug)]
struct SessionSlotLifecycle {
    phase: SessionSlotPhase,
    active_actions: usize,
}

impl SessionSlot {
    pub(super) fn new(session: RuntimeDynamicSession, wake: RuntimeWakeRegistration) -> Self {
        Self {
            session: Mutex::new(Some(session)),
            lifecycle: Mutex::new(SessionSlotLifecycle {
                phase: SessionSlotPhase::Open,
                active_actions: 0,
            }),
            actions_drained: Condvar::new(),
            frame_activity: RuntimeFrameActivity::new(wake),
        }
    }

    pub(super) fn begin_action(self: &Arc<Self>) -> Option<SessionActionGuard> {
        let mut lifecycle = self.lock_lifecycle();
        if lifecycle.phase != SessionSlotPhase::Open {
            return None;
        }
        lifecycle.active_actions += 1;
        drop(lifecycle);
        Some(SessionActionGuard::new(Arc::clone(self)))
    }

    pub(super) fn begin_release_action(self: &Arc<Self>) -> Option<SessionActionGuard> {
        let mut lifecycle = self.lock_lifecycle();
        if lifecycle.phase == SessionSlotPhase::Closing {
            return None;
        }
        lifecycle.active_actions += 1;
        drop(lifecycle);
        Some(SessionActionGuard::new(Arc::clone(self)))
    }

    pub(super) fn finish_action(&self) {
        let mut lifecycle = self.lock_lifecycle();
        lifecycle.active_actions -= 1;
        if lifecycle.active_actions == 0 {
            self.actions_drained.notify_all();
        }
    }

    pub(super) fn begin_close(&self) -> bool {
        let mut lifecycle = self.lock_lifecycle();
        match lifecycle.phase {
            SessionSlotPhase::Open | SessionSlotPhase::TeardownRetryPending => {
                lifecycle.phase = SessionSlotPhase::Closing;
                true
            }
            SessionSlotPhase::Closing => false,
        }
    }

    pub(super) fn preserve_failed_teardown_for_retry(&self) {
        let mut lifecycle = self.lock_lifecycle();
        debug_assert_eq!(lifecycle.phase, SessionSlotPhase::Closing);
        debug_assert_eq!(lifecycle.active_actions, 0);
        lifecycle.phase = SessionSlotPhase::TeardownRetryPending;
    }

    pub(super) fn wait_for_actions(&self) {
        let lifecycle = self.lock_lifecycle();
        drop(
            self.actions_drained
                .wait_while(lifecycle, |state| state.active_actions != 0)
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        );
    }

    pub(super) fn lock_session(&self) -> MutexGuard<'_, Option<RuntimeDynamicSession>> {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn take_session(&self) -> Option<RuntimeDynamicSession> {
        self.lock_session().take()
    }

    pub(super) fn frame_activity(&self) -> &RuntimeFrameActivity {
        &self.frame_activity
    }

    #[cfg(test)]
    pub(super) fn is_closing(&self) -> bool {
        self.lock_lifecycle().phase != SessionSlotPhase::Open
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, SessionSlotLifecycle> {
        self.lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
