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
