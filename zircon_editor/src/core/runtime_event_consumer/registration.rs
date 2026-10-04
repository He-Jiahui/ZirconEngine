use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde_json::value::RawValue;
use zircon_runtime::plugin::PluginEventConsumerManifest;

use crate::core::extension::{ContributionSource, ContributionTicket};

use super::{EditorRuntimeEventConsumerApplyError, EditorRuntimeEventConsumerError};

pub trait EditorRuntimeEventConsumerState: Send + 'static {
    type Payload: DeserializeOwned + Send + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    fn begin_session(&mut self, play_session_id: u64);

    fn consume(
        &mut self,
        play_session_id: u64,
        sequence: u64,
        payload: Self::Payload,
    ) -> Result<(), Self::Error>;

    fn end_session(&mut self, play_session_id: u64);
}

type BeginConsumer = dyn Fn(u64) + Send + Sync;
type ApplyConsumer =
    dyn Fn(u64, u64, &RawValue) -> Result<(), EditorRuntimeEventConsumerApplyError> + Send + Sync;
type EndConsumer = dyn Fn(u64) + Send + Sync;

#[derive(Clone, Debug)]
struct EditorRuntimeEventConsumerContributionOwner {
    ticket: ContributionTicket,
    source: ContributionSource,
}

#[derive(Clone)]
pub struct EditorRuntimeEventConsumerRegistration {
    manifest: PluginEventConsumerManifest,
    contribution_owner: Option<EditorRuntimeEventConsumerContributionOwner>,
    state: Arc<dyn Any + Send + Sync>,
    begin: Arc<BeginConsumer>,
    apply: Arc<ApplyConsumer>,
    end: Arc<EndConsumer>,
}

impl EditorRuntimeEventConsumerRegistration {
    pub fn typed<S>(manifest: PluginEventConsumerManifest, state: Arc<Mutex<S>>) -> Self
    where
        S: EditorRuntimeEventConsumerState + Sync,
    {
        let erased_state: Arc<dyn Any + Send + Sync> = state.clone();
        let begin_state = state.clone();
        let apply_state = state.clone();
        let end_state = state;
        Self {
            manifest,
            contribution_owner: None,
            state: erased_state,
            begin: Arc::new(move |play_session_id| {
                begin_state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .begin_session(play_session_id);
            }),
            apply: Arc::new(move |play_session_id, sequence, payload| {
                let payload = serde_json::from_str::<S::Payload>(payload.get())
                    .map_err(|source| EditorRuntimeEventConsumerApplyError::Decode { source })?;
                apply_state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .consume(play_session_id, sequence, payload)
                    .map_err(EditorRuntimeEventConsumerApplyError::state)
            }),
            end: Arc::new(move |play_session_id| {
                end_state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .end_session(play_session_id);
            }),
        }
    }

    pub fn manifest(&self) -> &PluginEventConsumerManifest {
        &self.manifest
    }

    pub(crate) fn contribution_ticket(&self) -> Option<ContributionTicket> {
        self.contribution_owner.as_ref().map(|owner| owner.ticket)
    }

    pub(crate) fn contribution_source(&self) -> Option<&ContributionSource> {
        self.contribution_owner.as_ref().map(|owner| &owner.source)
    }

    pub fn state<S>(&self) -> Option<Arc<Mutex<S>>>
    where
        S: Send + Sync + 'static,
    {
        self.state.clone().downcast::<Mutex<S>>().ok()
    }

    pub(super) fn begin_session(&self, play_session_id: u64) {
        (self.begin)(play_session_id);
    }

    pub(super) fn consume(
        &self,
        play_session_id: u64,
        sequence: u64,
        payload: &RawValue,
    ) -> Result<(), EditorRuntimeEventConsumerApplyError> {
        (self.apply)(play_session_id, sequence, payload)
    }

    pub(super) fn end_session(&self, play_session_id: u64) {
        (self.end)(play_session_id);
    }

    fn bind_contribution_owner(
        &mut self,
        ticket: ContributionTicket,
        source: &ContributionSource,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        if self.contribution_owner.is_some() {
            return Err(EditorRuntimeEventConsumerError::ContributionAlreadyOwned {
                consumer_id: self.manifest.consumer_id.clone(),
            });
        }
        self.contribution_owner = Some(EditorRuntimeEventConsumerContributionOwner {
            ticket,
            source: source.clone(),
        });
        Ok(())
    }
}

impl fmt::Debug for EditorRuntimeEventConsumerRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorRuntimeEventConsumerRegistration")
            .field("manifest", &self.manifest)
            .field("contribution_owner", &self.contribution_owner)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Default)]
pub struct EditorRuntimeEventConsumerRegistry {
    registrations: BTreeMap<String, EditorRuntimeEventConsumerRegistration>,
}

impl EditorRuntimeEventConsumerRegistry {
    pub fn register(
        &mut self,
        registration: EditorRuntimeEventConsumerRegistration,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let consumer_id = registration.manifest().consumer_id.clone();
        if self.registrations.contains_key(&consumer_id) {
            return Err(EditorRuntimeEventConsumerError::DuplicateConsumer { consumer_id });
        }
        self.registrations.insert(consumer_id, registration);
        Ok(())
    }

    pub fn extend(
        &mut self,
        mut registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        // Preflight every key before moving the batch into the live map. BTreeMap already
        // guarantees uniqueness inside the incoming registry, so checking the existing map is
        // sufficient to keep a rejected batch atomic without cloning every registration.
        for consumer_id in registry.registrations.keys() {
            if self.registrations.contains_key(consumer_id) {
                return Err(EditorRuntimeEventConsumerError::DuplicateConsumer {
                    consumer_id: consumer_id.clone(),
                });
            }
        }
        self.registrations.append(&mut registry.registrations);
        Ok(())
    }

    pub(crate) fn extend_contribution(
        &mut self,
        ticket: ContributionTicket,
        source: ContributionSource,
        mut registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        for registration in registry.registrations.values_mut() {
            registration.bind_contribution_owner(ticket, &source)?;
        }
        self.extend(registry)
    }

    pub(crate) fn without_contribution(&self, ticket: ContributionTicket) -> (Self, Vec<String>) {
        let removed = self
            .registrations
            .iter()
            .filter(|(_, registration)| registration.contribution_ticket() == Some(ticket))
            .map(|(consumer_id, _)| consumer_id.clone())
            .collect::<Vec<_>>();
        let mut candidate = self.clone();
        for consumer_id in &removed {
            candidate.registrations.remove(consumer_id);
        }
        (candidate, removed)
    }

    pub fn registrations(&self) -> impl Iterator<Item = &EditorRuntimeEventConsumerRegistration> {
        self.registrations.values()
    }

    pub fn registration(
        &self,
        consumer_id: &str,
    ) -> Option<&EditorRuntimeEventConsumerRegistration> {
        self.registrations.get(consumer_id)
    }

    pub fn manifests(&self) -> Vec<PluginEventConsumerManifest> {
        self.registrations
            .values()
            .map(|registration| registration.manifest().clone())
            .collect()
    }
}

#[cfg(test)]
#[path = "tests/registration.rs"]
mod tests;

#[cfg(test)]
#[path = "registration/tests/optimization_batch_hx_editor607_tests.rs"]
mod optimization_batch_hx_editor607_tests;
