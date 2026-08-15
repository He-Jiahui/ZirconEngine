use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde_json::value::RawValue;
use zircon_runtime::plugin::PluginEventConsumerManifest;

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
type ApplyConsumer = dyn Fn(u64, u64, Box<RawValue>) -> Result<(), EditorRuntimeEventConsumerApplyError>
    + Send
    + Sync;
type EndConsumer = dyn Fn(u64) + Send + Sync;

#[derive(Clone)]
pub struct EditorRuntimeEventConsumerRegistration {
    manifest: PluginEventConsumerManifest,
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
        payload: Box<RawValue>,
    ) -> Result<(), EditorRuntimeEventConsumerApplyError> {
        (self.apply)(play_session_id, sequence, payload)
    }

    pub(super) fn end_session(&self, play_session_id: u64) {
        (self.end)(play_session_id);
    }
}

impl fmt::Debug for EditorRuntimeEventConsumerRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorRuntimeEventConsumerRegistration")
            .field("manifest", &self.manifest)
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

    #[test]
    fn duplicate_late_in_batch_leaves_active_registry_unchanged() {
        let mut active = EditorRuntimeEventConsumerRegistry::default();
        active.register(registration("z-duplicate")).unwrap();

        let mut batch = EditorRuntimeEventConsumerRegistry::default();
        batch.register(registration("a-new")).unwrap();
        batch.register(registration("z-duplicate")).unwrap();

        let error = active
            .extend(batch)
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
