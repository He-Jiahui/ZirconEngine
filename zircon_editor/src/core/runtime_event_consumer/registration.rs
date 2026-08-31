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
        registry: EditorRuntimeEventConsumerRegistry,
    ) -> Result<(), EditorRuntimeEventConsumerError> {
        let mut candidate = self.clone();
        for registration in registry.registrations.into_values() {
            candidate.register(registration)?;
        }
        *self = candidate;
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
mod tests {
    use std::convert::Infallible;
    use std::sync::{Arc, Mutex};

    use zircon_runtime::plugin::PluginEventConsumerManifest;

    use crate::core::extension::{
        ContributionBatch, ContributionSource, ContributionStore, ContributionTicket,
        PluginContributionId,
    };

    use super::{
        EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerRegistration,
        EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerState,
    };

    struct NoopConsumer;

    impl EditorRuntimeEventConsumerState for NoopConsumer {
        type Payload = ();
        type Error = Infallible;

        fn begin_session(&mut self, _play_session_id: u64) {}

        fn consume(
            &mut self,
            _play_session_id: u64,
            _sequence: u64,
            _payload: Self::Payload,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        fn end_session(&mut self, _play_session_id: u64) {}
    }

    fn registration(consumer_id: &str) -> EditorRuntimeEventConsumerRegistration {
        EditorRuntimeEventConsumerRegistration::typed(
            PluginEventConsumerManifest::new(
                consumer_id,
                format!("{consumer_id}.event"),
                format!("{consumer_id}.event.v1"),
            ),
            Arc::new(Mutex::new(NoopConsumer)),
        )
    }

    fn plugin_ticket(
        store: &mut ContributionStore,
        plugin_id: &str,
    ) -> (ContributionTicket, ContributionSource) {
        let source = ContributionSource::Plugin(
            PluginContributionId::parse(plugin_id).expect("plugin id should be valid"),
        );
        let ticket = store
            .contribute(source.clone(), ContributionBatch::default())
            .expect("empty ownership batch should allocate a ticket");
        (ticket, source)
    }

    #[test]
    fn ticket_owned_revoke_candidate_preserves_other_generations_and_live_registry() {
        let mut store = ContributionStore::default();
        let (weather_ticket, weather_source) = plugin_ticket(&mut store, "weather");
        let (lighting_ticket, lighting_source) = plugin_ticket(&mut store, "lighting");
        let mut active = EditorRuntimeEventConsumerRegistry::default();
        active.register(registration("builtin.console")).unwrap();

        let mut weather = EditorRuntimeEventConsumerRegistry::default();
        weather.register(registration("weather.clouds")).unwrap();
        weather.register(registration("weather.rain")).unwrap();
        active
            .extend_contribution(weather_ticket, weather_source.clone(), weather)
            .unwrap();
        let mut lighting = EditorRuntimeEventConsumerRegistry::default();
        lighting
            .register(registration("lighting.exposure"))
            .unwrap();
        active
            .extend_contribution(lighting_ticket, lighting_source.clone(), lighting)
            .unwrap();

        assert_eq!(
            active
                .registration("weather.clouds")
                .and_then(|registration| registration.contribution_ticket()),
            Some(weather_ticket)
        );
        assert_eq!(
            active
                .registration("lighting.exposure")
                .and_then(|registration| registration.contribution_source()),
            Some(&lighting_source)
        );
        let (candidate, removed) = active.without_contribution(weather_ticket);

        assert_eq!(removed, vec!["weather.clouds", "weather.rain"]);
        assert!(active.registration("weather.clouds").is_some());
        assert!(candidate.registration("weather.clouds").is_none());
        assert!(candidate.registration("weather.rain").is_none());
        assert!(candidate.registration("builtin.console").is_some());
        assert!(candidate.registration("lighting.exposure").is_some());
    }

    #[test]
    fn contribution_batch_cannot_be_rebound_to_another_ticket() {
        let mut store = ContributionStore::default();
        let (weather_ticket, weather_source) = plugin_ticket(&mut store, "weather");
        let (lighting_ticket, lighting_source) = plugin_ticket(&mut store, "lighting");
        let mut weather = EditorRuntimeEventConsumerRegistry::default();
        let mut unbound = EditorRuntimeEventConsumerRegistry::default();
        unbound.register(registration("weather.clouds")).unwrap();
        weather
            .extend_contribution(weather_ticket, weather_source, unbound)
            .unwrap();
        let mut target = EditorRuntimeEventConsumerRegistry::default();

        let error = target
            .extend_contribution(lighting_ticket, lighting_source, weather)
            .expect_err("an owned batch must not change contribution identity");

        assert!(matches!(
            error,
            EditorRuntimeEventConsumerError::ContributionAlreadyOwned { consumer_id }
                if consumer_id == "weather.clouds"
        ));
        assert!(target.registration("weather.clouds").is_none());
    }

    #[test]
    fn duplicate_late_in_batch_leaves_active_registry_unchanged() {
        let mut store = ContributionStore::default();
        let (ticket, source) = plugin_ticket(&mut store, "weather");
        let mut active = EditorRuntimeEventConsumerRegistry::default();
        active.register(registration("z-duplicate")).unwrap();

        let mut batch = EditorRuntimeEventConsumerRegistry::default();
        batch.register(registration("a-new")).unwrap();
        batch.register(registration("z-duplicate")).unwrap();

        let error = active
            .extend_contribution(ticket, source, batch)
            .expect_err("a duplicate later in the batch must reject the whole batch");
        assert!(matches!(
            error,
            EditorRuntimeEventConsumerError::DuplicateConsumer { consumer_id }
                if consumer_id == "z-duplicate"
        ));
        assert!(active.registration("z-duplicate").is_some());
        assert!(active.registration("a-new").is_none());
        assert_eq!(active.manifests().len(), 1);
    }
}
