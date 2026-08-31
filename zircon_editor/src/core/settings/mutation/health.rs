use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal;

use super::super::{SettingsFileGeneration, SettingsPersistenceSubmitError, SettingsScope};
use super::SettingsDocumentIdentity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsPersistenceHealthStatus {
    Unavailable,
    Ready,
    Queued,
    Durable,
    PendingAdmission(SettingsPersistenceSubmitError),
    Terminal(BoundedKeyedIoTerminal),
}

impl SettingsPersistenceHealthStatus {
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::PendingAdmission(_) | Self::Terminal(BoundedKeyedIoTerminal::Failed(_))
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsPersistenceDocumentHealth {
    scope: SettingsScope,
    binding_generation: Option<u64>,
    file_generation: Option<SettingsFileGeneration>,
    status: SettingsPersistenceHealthStatus,
}

impl SettingsPersistenceDocumentHealth {
    pub const fn scope(self) -> SettingsScope {
        self.scope
    }

    pub const fn binding_generation(self) -> Option<u64> {
        self.binding_generation
    }

    pub const fn file_generation(self) -> Option<SettingsFileGeneration> {
        self.file_generation
    }

    pub const fn status(self) -> SettingsPersistenceHealthStatus {
        self.status
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsPersistenceHealthSnapshot {
    generation: u64,
    user: SettingsPersistenceDocumentHealth,
    project: SettingsPersistenceDocumentHealth,
}

impl SettingsPersistenceHealthSnapshot {
    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn user(self) -> SettingsPersistenceDocumentHealth {
        self.user
    }

    pub const fn project(self) -> SettingsPersistenceDocumentHealth {
        self.project
    }
}

pub(crate) trait SettingsPersistenceHealthSubscriber: Send + Sync {
    /// Receives immutable status after the health owner releases its state lock.
    fn persistence_health_changed(&self, snapshot: &SettingsPersistenceHealthSnapshot);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SettingsPersistenceObservation {
    pub(super) document: SettingsDocumentIdentity,
    pub(super) id: u64,
}

#[derive(Clone, Copy)]
struct TrackedDocumentHealth {
    document: Option<SettingsDocumentIdentity>,
    active_observation: Option<u64>,
    file_generation: Option<SettingsFileGeneration>,
    status: SettingsPersistenceHealthStatus,
}

impl TrackedDocumentHealth {
    const fn unavailable() -> Self {
        Self {
            document: None,
            active_observation: None,
            file_generation: None,
            status: SettingsPersistenceHealthStatus::Unavailable,
        }
    }

    const fn ready(document: SettingsDocumentIdentity) -> Self {
        Self {
            document: Some(document),
            active_observation: None,
            file_generation: None,
            status: SettingsPersistenceHealthStatus::Ready,
        }
    }

    fn snapshot(self, scope: SettingsScope) -> SettingsPersistenceDocumentHealth {
        SettingsPersistenceDocumentHealth {
            scope,
            binding_generation: match self.document {
                Some(SettingsDocumentIdentity::Project(generation)) => Some(generation),
                _ => None,
            },
            file_generation: self.file_generation,
            status: self.status,
        }
    }
}

struct SettingsPersistenceHealthState {
    generation: u64,
    next_observation: u64,
    user: TrackedDocumentHealth,
    project: TrackedDocumentHealth,
}

pub(super) struct SettingsPersistenceHealthAuthority {
    state: Mutex<SettingsPersistenceHealthState>,
    subscriber: Mutex<Option<Arc<dyn SettingsPersistenceHealthSubscriber>>>,
}

impl SettingsPersistenceHealthAuthority {
    pub(super) fn new(user_writable: bool) -> Self {
        Self {
            state: Mutex::new(SettingsPersistenceHealthState {
                generation: 0,
                next_observation: 0,
                user: if user_writable {
                    TrackedDocumentHealth::ready(SettingsDocumentIdentity::User)
                } else {
                    TrackedDocumentHealth::unavailable()
                },
                project: TrackedDocumentHealth::unavailable(),
            }),
            subscriber: Mutex::new(None),
        }
    }

    pub(super) fn snapshot(&self) -> SettingsPersistenceHealthSnapshot {
        snapshot_for_state(&self.lock_state())
    }

    pub(super) fn configure_subscriber(
        &self,
        subscriber: Arc<dyn SettingsPersistenceHealthSubscriber>,
    ) {
        *self.lock_subscriber() = Some(Arc::clone(&subscriber));
        subscriber.persistence_health_changed(&self.snapshot());
    }

    pub(super) fn bind_project(&self, generation: u64, writable: bool) {
        self.publish_update(|state| {
            state.project = if writable {
                TrackedDocumentHealth::ready(SettingsDocumentIdentity::Project(generation))
            } else {
                TrackedDocumentHealth {
                    document: Some(SettingsDocumentIdentity::Project(generation)),
                    ..TrackedDocumentHealth::unavailable()
                }
            };
            true
        });
    }

    pub(super) fn clear_project(&self) {
        self.publish_update(|state| {
            state.project = TrackedDocumentHealth::unavailable();
            true
        });
    }

    pub(super) fn begin_submission(
        &self,
        document: SettingsDocumentIdentity,
        file_generation: SettingsFileGeneration,
    ) -> SettingsPersistenceObservation {
        let mut observation = SettingsPersistenceObservation { document, id: 0 };
        self.publish_update(|state| {
            state.next_observation = next_nonzero(state.next_observation);
            observation.id = state.next_observation;
            let tracked = tracked_document_mut(state, document);
            tracked.document = Some(document);
            tracked.active_observation = Some(observation.id);
            tracked.file_generation = Some(file_generation);
            tracked.status = SettingsPersistenceHealthStatus::Queued;
            true
        });
        observation
    }

    pub(super) fn admission_rejected(
        &self,
        observation: SettingsPersistenceObservation,
        error: SettingsPersistenceSubmitError,
    ) {
        self.publish_observation_update(observation, |tracked| {
            tracked.status = SettingsPersistenceHealthStatus::PendingAdmission(error);
        });
    }

    pub(super) fn observe_terminal(
        &self,
        observation: SettingsPersistenceObservation,
        terminal: BoundedKeyedIoTerminal,
    ) {
        self.publish_observation_update(observation, |tracked| {
            tracked.status = if terminal == BoundedKeyedIoTerminal::Succeeded {
                SettingsPersistenceHealthStatus::Durable
            } else {
                SettingsPersistenceHealthStatus::Terminal(terminal)
            };
        });
    }

    fn publish_observation_update(
        &self,
        observation: SettingsPersistenceObservation,
        update: impl FnOnce(&mut TrackedDocumentHealth),
    ) {
        self.publish_update(|state| {
            let tracked = tracked_document_mut(state, observation.document);
            if tracked.document != Some(observation.document)
                || tracked.active_observation != Some(observation.id)
            {
                return false;
            }
            update(tracked);
            true
        });
    }

    fn publish_update(&self, update: impl FnOnce(&mut SettingsPersistenceHealthState) -> bool) {
        let snapshot = {
            let mut state = self.lock_state();
            if !update(&mut state) {
                return;
            }
            state.generation = next_nonzero(state.generation);
            snapshot_for_state(&state)
        };
        let subscriber = { self.lock_subscriber().clone() };
        if let Some(subscriber) = subscriber {
            subscriber.persistence_health_changed(&snapshot);
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, SettingsPersistenceHealthState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_subscriber(
        &self,
    ) -> MutexGuard<'_, Option<Arc<dyn SettingsPersistenceHealthSubscriber>>> {
        self.subscriber
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn tracked_document_mut(
    state: &mut SettingsPersistenceHealthState,
    document: SettingsDocumentIdentity,
) -> &mut TrackedDocumentHealth {
    match document {
        SettingsDocumentIdentity::User => &mut state.user,
        SettingsDocumentIdentity::Project(_) => &mut state.project,
    }
}

fn snapshot_for_state(state: &SettingsPersistenceHealthState) -> SettingsPersistenceHealthSnapshot {
    SettingsPersistenceHealthSnapshot {
        generation: state.generation,
        user: state.user.snapshot(SettingsScope::User),
        project: state.project.snapshot(SettingsScope::Project),
    }
}

fn next_nonzero(value: u64) -> u64 {
    let next = value.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}
