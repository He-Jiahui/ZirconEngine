use std::any::Any;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use zircon_runtime_interface::ZrStatus;

use super::action_guard::SessionActionGuard;
use super::session_owner::abi_status::OwnedSessionStatus;
use super::session_owner::runtime::RuntimeSessionOwner;
use super::session_owner::runtime::RuntimeSessionOwnerCreateError;
use super::session_owner::{OwnerDispatchError, OwnerShutdownReceipt};
use super::{RuntimeFrameActivity, RuntimeWakeRegistration};
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::RuntimeDynamicSession;
use crate::plugin::RuntimePluginRegistrationReport;

pub(in crate::dynamic_api::session) struct SessionSlot {
    owner: RuntimeSessionOwner,
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
    #[cfg(test)]
    pub(super) fn new(
        session: RuntimeDynamicSession,
        wake: RuntimeWakeRegistration,
    ) -> Result<Self, String> {
        let session = session.with_runtime_frame_wake(wake.channel_wake());
        let frame_activity = RuntimeFrameActivity::new(wake.clone());
        let code_owner = Arc::new(()) as Arc<dyn Any + Send + Sync>;
        let owner = RuntimeSessionOwner::from_session(session, wake, code_owner)?;
        Ok(Self {
            owner,
            lifecycle: Mutex::new(SessionSlotLifecycle {
                phase: SessionSlotPhase::Open,
                active_actions: 0,
            }),
            actions_drained: Condvar::new(),
            frame_activity,
        })
    }

    pub(super) fn create(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        wake: RuntimeWakeRegistration,
    ) -> Result<Self, ZrStatus> {
        let frame_activity = RuntimeFrameActivity::new(wake.clone());
        let code_owner = Arc::new(()) as Arc<dyn Any + Send + Sync>;
        let owner = match RuntimeSessionOwner::create(profile, project, wake, code_owner) {
            Ok(owner) => owner,
            Err(RuntimeSessionOwnerCreateError::Session(error)) => return Err(error.into_abi()),
            Err(RuntimeSessionOwnerCreateError::Owner(error)) => {
                return Err(crate::dynamic_api::session::status::error_status(error));
            }
        };
        Ok(Self {
            owner,
            lifecycle: Mutex::new(SessionSlotLifecycle {
                phase: SessionSlotPhase::Open,
                active_actions: 0,
            }),
            actions_drained: Condvar::new(),
            frame_activity,
        })
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

    pub(super) fn create_with_linked_plugins(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        registrations: Vec<RuntimePluginRegistrationReport>,
    ) -> Result<Self, RuntimeSessionOwnerCreateError> {
        let wake = RuntimeWakeRegistration::disabled();
        let frame_activity = RuntimeFrameActivity::new(wake.clone());
        let code_owner = Arc::new(()) as Arc<dyn Any + Send + Sync>;
        let owner = RuntimeSessionOwner::create_with_linked_plugins(
            profile,
            project,
            registrations,
            wake,
            code_owner,
        )?;
        Ok(Self {
            owner,
            lifecycle: Mutex::new(SessionSlotLifecycle {
                phase: SessionSlotPhase::Open,
                active_actions: 0,
            }),
            actions_drained: Condvar::new(),
            frame_activity,
        })
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
            // Concurrent destroy callers share the owner's retained receipt.
            SessionSlotPhase::Closing => true,
        }
    }

    pub(super) fn preserve_failed_teardown_for_retry(&self) {
        let mut lifecycle = self.lock_lifecycle();
        if lifecycle.phase != SessionSlotPhase::Open {
            lifecycle.phase = SessionSlotPhase::TeardownRetryPending;
        }
    }

    pub(super) fn wait_for_actions(&self, timeout: Duration) -> bool {
        let lifecycle = self.lock_lifecycle();
        let (lifecycle, _) = self
            .actions_drained
            .wait_timeout_while(lifecycle, timeout, |state| state.active_actions != 0)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        lifecycle.active_actions == 0
    }

    pub(super) fn dispatch_scoped<R: Send>(
        &self,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send,
    ) -> Result<Result<R, OwnedSessionStatus>, OwnerDispatchError> {
        self.owner.dispatch_scoped(action)
    }

    pub(super) fn dispatch_owned<R: Send + 'static>(
        &self,
        deadline: Instant,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send + 'static,
    ) -> Result<Receiver<Result<R, OwnedSessionStatus>>, OwnerDispatchError> {
        self.owner.dispatch(deadline, action)
    }

    pub(super) fn shutdown_until(&self, deadline: Instant) -> OwnerShutdownReceipt {
        self.owner.shutdown_until(deadline)
    }

    pub(super) fn shutdown_receipt(&self) -> Option<OwnerShutdownReceipt> {
        self.owner.shutdown_receipt()
    }

    pub(super) fn frame_activity(&self) -> &RuntimeFrameActivity {
        &self.frame_activity
    }

    #[cfg(test)]
    pub(super) fn is_closing(&self) -> bool {
        self.lock_lifecycle().phase != SessionSlotPhase::Open
    }

    #[cfg(test)]
    pub(super) fn active_actions(&self) -> usize {
        self.lock_lifecycle().active_actions
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, SessionSlotLifecycle> {
        self.lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
