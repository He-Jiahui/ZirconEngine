use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use zircon_runtime_interface::ZrStatus;

use crate::diagnostic_log::DynamicProcessLogLease;
use crate::dynamic_api::session::construction::RuntimeConstructionFailure;
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::registry::RuntimeWakeRegistration;
use crate::dynamic_api::session::status::error_status;
use crate::dynamic_api::session::RuntimeDynamicSession;
use crate::plugin::RuntimePluginRegistrationReport;

use super::abi_status::OwnedSessionStatus;
use super::{OwnerDispatchError, OwnerShutdownReceipt, SessionOwner};

const OWNER_STARTUP_SHUTDOWN_WAIT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub(in crate::dynamic_api::session) enum RuntimeSessionOwnerCreateError {
    Owner(String),
    Session(OwnedSessionStatus),
}

struct RuntimeSessionStartupFailure {
    status: OwnedSessionStatus,
    construction: RuntimeConstructionFailure,
    dynamic_process_log: Option<DynamicProcessLogLease>,
}

impl RuntimeSessionStartupFailure {
    fn capture(construction: RuntimeConstructionFailure) -> Self {
        // Capture only the diagnostic; the typed primary and unfinished Core remain owned here.
        let status = unsafe { OwnedSessionStatus::capture(error_status(construction.primary())) };
        Self {
            status,
            construction,
            dynamic_process_log: None,
        }
    }

    fn with_dynamic_process_log(mut self, mut log: DynamicProcessLogLease) -> Self {
        if self.construction.has_pending_core() {
            self.dynamic_process_log = Some(log);
        } else if !log.shutdown() {
            // Preserve the existing post-lease bootstrap log failure policy.
            eprintln!("fatal runtime session owner bootstrap log teardown incomplete");
            std::process::abort();
        }
        self
    }

    fn shutdown_until(&mut self, deadline: Instant) -> bool {
        if !self.construction.shutdown_until(deadline) {
            eprintln!(
                "runtime startup cleanup incomplete: {}; secondary shutdown: {}",
                self.construction.primary(),
                self.construction
                    .shutdown_error()
                    .expect("incomplete Core close diagnostic")
            );
            return false;
        }
        if let Some(log) = self.dynamic_process_log.as_mut() {
            if !log.shutdown_until(deadline) {
                return false;
            }
            self.dynamic_process_log = None;
        }
        Instant::now() <= deadline
    }
}

fn shutdown_owner_state_until(
    state: &mut Result<RuntimeDynamicSession, RuntimeSessionStartupFailure>,
    deadline: Instant,
) -> bool {
    match state {
        Ok(session) => session.shutdown_before_library_unload_until(deadline),
        Err(failure) => failure.shutdown_until(deadline),
    }
}

pub(in crate::dynamic_api::session) struct RuntimeSessionOwner {
    owner: SessionOwner<Result<RuntimeDynamicSession, RuntimeSessionStartupFailure>>,
    wake: RuntimeWakeRegistration,
    #[cfg(test)]
    startup_observation: Option<startup_observation::ObservationSink>,
}

impl RuntimeSessionOwner {
    #[cfg(test)]
    pub(in crate::dynamic_api::session) fn from_session(
        session: RuntimeDynamicSession,
        wake: RuntimeWakeRegistration,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, String> {
        let owner = SessionOwner::create(move || Ok(session), code_owner)?;
        Ok(Self {
            owner,
            wake,
            #[cfg(test)]
            startup_observation: None,
        })
    }

    pub(in crate::dynamic_api::session) fn create(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        wake: RuntimeWakeRegistration,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, RuntimeSessionOwnerCreateError> {
        let owner_wake = wake.clone();
        #[cfg(test)]
        let startup_observation = startup_observation::current();
        #[cfg(test)]
        let factory_observation = startup_observation.clone();
        let owner = SessionOwner::create(
            move || {
                let log =
                    crate::diagnostic_log::acquire_dynamic_unity_process_log("runtime-dynamic");
                let session = RuntimeDynamicSession::new(profile, project);
                crate::diagnostic_log::write_log(
                    "runtime_session",
                    "runtime_session_owner_factory_returned",
                );
                match session {
                    Ok(session) => Ok(session
                        .with_runtime_frame_wake(owner_wake.channel_wake())
                        .with_dynamic_process_log_lease(log)),
                    Err(error) => {
                        #[cfg(test)]
                        startup_observation::record_failure(&factory_observation, &error);
                        Err(RuntimeSessionStartupFailure::capture(error)
                            .with_dynamic_process_log(log))
                    }
                }
            },
            code_owner,
        )
        .map_err(RuntimeSessionOwnerCreateError::Owner)?;
        crate::diagnostic_log::write_log("runtime_session", "runtime_session_owner_task_spawned");
        let session_owner = Self {
            owner,
            wake,
            #[cfg(test)]
            startup_observation,
        };
        let startup = session_owner.owner.dispatch_scoped(|state| match state {
            Ok(_) => Ok(()),
            Err(error) => Err(error.status.clone()),
        });
        crate::diagnostic_log::write_log(
            "runtime_session",
            "runtime_session_owner_bootstrap_returned",
        );
        match startup {
            Ok(Ok(())) => Ok(session_owner),
            Ok(Err(error)) => {
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Session(error))
            }
            Err(error) => {
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Owner(format!(
                    "runtime session owner terminated during startup: {error:?}"
                )))
            }
        }
    }

    pub(in crate::dynamic_api::session) fn dispatch<R: Send + 'static>(
        &self,
        deadline: Instant,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send + 'static,
    ) -> Result<Receiver<Result<R, OwnedSessionStatus>>, OwnerDispatchError> {
        self.owner.dispatch(deadline, move |state| {
            let session = state.as_mut().map_err(|error| error.status.clone())?;
            action(session).map_err(|status| unsafe { OwnedSessionStatus::capture(status) })
        })
    }

    pub(in crate::dynamic_api::session) fn create_with_linked_plugins(
        profile: RuntimeDynamicSessionProfile,
        project: Option<RuntimeProjectConfig>,
        registrations: Vec<RuntimePluginRegistrationReport>,
        wake: RuntimeWakeRegistration,
        code_owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, RuntimeSessionOwnerCreateError> {
        let owner_wake = wake.clone();
        #[cfg(test)]
        let startup_observation = startup_observation::current();
        #[cfg(test)]
        let factory_observation = startup_observation.clone();
        let owner = SessionOwner::create(
            move || {
                RuntimeDynamicSession::new_with_linked_plugins(profile, project, registrations)
                    .map(|session| session.with_runtime_frame_wake(owner_wake.channel_wake()))
                    .map_err(|error| {
                        #[cfg(test)]
                        startup_observation::record_failure(&factory_observation, &error);
                        RuntimeSessionStartupFailure::capture(error)
                    })
            },
            code_owner,
        )
        .map_err(RuntimeSessionOwnerCreateError::Owner)?;
        let session_owner = Self {
            owner,
            wake,
            #[cfg(test)]
            startup_observation,
        };
        match session_owner.owner.dispatch_scoped(|state| match state {
            Ok(_) => Ok(()),
            Err(error) => Err(error.status.clone()),
        }) {
            Ok(Ok(())) => Ok(session_owner),
            Ok(Err(error)) => {
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Session(error))
            }
            Err(error) => {
                session_owner.join_failed_startup();
                Err(RuntimeSessionOwnerCreateError::Owner(format!(
                    "runtime session owner terminated during startup: {error:?}"
                )))
            }
        }
    }

    pub(in crate::dynamic_api::session) fn dispatch_scoped<R: Send>(
        &self,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> Result<R, ZrStatus> + Send,
    ) -> Result<Result<R, OwnedSessionStatus>, OwnerDispatchError> {
        self.owner.dispatch_scoped(move |state| {
            let session = state.as_mut().map_err(|error| error.status.clone())?;
            action(session).map_err(|status| unsafe { OwnedSessionStatus::capture(status) })
        })
    }

    pub(in crate::dynamic_api::session) fn shutdown_until(
        &self,
        deadline: Instant,
    ) -> OwnerShutdownReceipt {
        self.wake.disable_new_entries();
        if !self
            .wake
            .wait_for_callbacks(deadline.saturating_duration_since(Instant::now()))
        {
            return OwnerShutdownReceipt::Pending;
        }
        self.owner.shutdown_until(deadline, move |state| {
            shutdown_owner_state_until(state, deadline)
        })
    }

    pub(in crate::dynamic_api::session) fn shutdown_receipt(&self) -> Option<OwnerShutdownReceipt> {
        self.owner.shutdown_receipt()
    }

    fn join_failed_startup(&self) {
        let deadline = Instant::now()
            .checked_add(OWNER_STARTUP_SHUTDOWN_WAIT)
            .unwrap_or_else(Instant::now);
        #[cfg(all(test, windows))]
        super::receipt_publication::startup_deadline(deadline);
        #[cfg(test)]
        let startup_observation = self.startup_observation.clone();
        let receipt = self.owner.shutdown_until(deadline, move |state| {
            #[cfg(all(test, windows))]
            super::receipt_publication::callback_entered(deadline);
            let completed = shutdown_owner_state_until(state, deadline);
            #[cfg(test)]
            startup_observation::record_cleanup(&startup_observation, state, completed);
            #[cfg(all(test, windows))]
            super::receipt_publication::callback_completed(deadline, completed);
            completed
        });
        #[cfg(test)]
        startup_observation::record_owner_receipt(&self.startup_observation, receipt);
        if !matches!(
            receipt,
            OwnerShutdownReceipt::Joined | OwnerShutdownReceipt::Panicked
        ) {
            eprintln!("fatal runtime session owner startup teardown incomplete");
            std::process::abort();
        }
    }
}

#[cfg(test)]
#[path = "tests/runtime_startup_observation.rs"]
pub(in crate::dynamic_api::session) mod startup_observation;

#[cfg(test)]
#[path = "tests/runtime.rs"]
mod tests;
