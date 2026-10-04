use std::collections::{BTreeSet, HashSet, VecDeque};
use std::convert::Infallible;
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use zircon_runtime::plugin::PluginEventConsumerManifest;
use zircon_runtime_interface::ZrRuntimePluginEventSubscriptionHandle;

use crate::core::extension::{
    ContributionBatch, ContributionSource, ContributionStore, ContributionTicket,
    PluginContributionId,
};
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerError, EditorRuntimeEventConsumerRegistration,
    EditorRuntimeEventConsumerRegistry, EditorRuntimeEventConsumerState,
};

use super::super::execution_support::LifecycleExecutionGuard;
use super::super::health::ConsumerCallbackHealth;
use super::super::{ActiveConsumer, QualifiedSubscription};

#[derive(Default)]
struct ConsumerState {
    ended_session: Option<u64>,
}

impl EditorRuntimeEventConsumerState for ConsumerState {
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

    fn end_session(&mut self, play_session_id: u64) {
        self.ended_session = Some(play_session_id);
    }
}

fn registration(
    consumer_id: &str,
) -> (
    EditorRuntimeEventConsumerRegistration,
    Arc<Mutex<ConsumerState>>,
) {
    let state = Arc::new(Mutex::new(ConsumerState::default()));
    (
        EditorRuntimeEventConsumerRegistration::typed(
            PluginEventConsumerManifest::new(
                consumer_id,
                format!("{consumer_id}.event"),
                format!("{consumer_id}.event.v1"),
            ),
            Arc::clone(&state),
        ),
        state,
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

fn registry_with(consumer_id: &str) -> EditorRuntimeEventConsumerRegistry {
    let mut registry = EditorRuntimeEventConsumerRegistry::default();
    registry.register(registration(consumer_id).0).unwrap();
    registry
}

#[test]
fn host_ticket_retirement_preserves_builtin_and_other_contribution() {
    let host = super::super::EditorRuntimeEventConsumerHost::default();
    let mut store = ContributionStore::default();
    let (weather_ticket, weather_source) = plugin_ticket(&mut store, "weather");
    let (lighting_ticket, lighting_source) = plugin_ticket(&mut store, "lighting");
    host.register(registry_with("builtin.console")).unwrap();
    let candidate = host
        .prepare_contribution_registration(
            weather_ticket,
            weather_source,
            registry_with("weather.clouds"),
        )
        .unwrap();
    host.install_prepared_registration(candidate);
    let candidate = host
        .prepare_contribution_registration(
            lighting_ticket,
            lighting_source,
            registry_with("lighting.exposure"),
        )
        .unwrap();
    host.install_prepared_registration(candidate);

    let removed = host.retire_contribution(weather_ticket).unwrap();

    assert_eq!(removed.removed, ["weather.clouds"]);
    let registry = host
        .registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(registry.registration("weather.clouds").is_none());
    assert!(registry.registration("builtin.console").is_some());
    assert!(registry.registration("lighting.exposure").is_some());
}

#[test]
fn cleanup_error_still_retires_active_generation_and_registry_owner() {
    let gateway = EditorRuntimeGatewayHandle::detached();
    let host = super::super::EditorRuntimeEventConsumerHost::new(gateway.clone());
    let mut store = ContributionStore::default();
    let (ticket, source) = plugin_ticket(&mut store, "weather");
    let (registration, state) = registration("weather.clouds");
    let mut contributed = EditorRuntimeEventConsumerRegistry::default();
    contributed.register(registration).unwrap();
    let candidate = host
        .prepare_contribution_registration(ticket, source, contributed)
        .unwrap();
    let owned_registration = candidate
        .registration("weather.clouds")
        .expect("candidate should retain its registration")
        .clone();
    host.install_prepared_registration(candidate);
    let origin = gateway.current_lease().origin();
    let subscription = QualifiedSubscription::new(
        ZrRuntimePluginEventSubscriptionHandle::new(17),
        origin.identity().clone(),
    );
    host.active
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(
            "weather.clouds".to_string(),
            ActiveConsumer {
                registration: owned_registration,
                origin,
                health: ConsumerCallbackHealth::default(),
                subscription,
                generation: 1,
                last_sequence: None,
                pending: VecDeque::new(),
                pending_retained_bytes: 0,
                last_observed_runtime_remaining_deliveries: None,
                last_observed_runtime_oldest_pending_age_millis: None,
                runtime_backlog_observed_at: None,
            },
        );
    *host
        .play_session_id
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(41);

    let report = host
        .retire_contribution(ticket)
        .expect("cleanup errors are reported after local publication");
    assert!(report.cleanup_error.is_some());

    assert_eq!(host.active_consumer_count(), 0);
    assert!(host
        .registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .registration("weather.clouds")
        .is_none());
    assert_eq!(
        state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .ended_session,
        Some(41)
    );
}

#[test]
fn busy_lifecycle_rejects_before_registry_publication() {
    let host = super::super::EditorRuntimeEventConsumerHost::default();
    let mut store = ContributionStore::default();
    let (ticket, source) = plugin_ticket(&mut store, "weather");
    let candidate = host
        .prepare_contribution_registration(ticket, source, registry_with("weather.clouds"))
        .unwrap();
    host.install_prepared_registration(candidate);
    let _busy = LifecycleExecutionGuard::enter(&host.execution_state, "test owner").unwrap();

    let error = host
        .retire_contribution(ticket)
        .expect_err("busy lifecycle must reject before local publication");

    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::LifecycleMutationBusy { .. }
    ));
    assert!(host
        .registry
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .registration("weather.clouds")
        .is_some());
}

#[test]
fn optimization_batch_id_editor614_retirement_membership_uses_preallocated_hash_index() {
    let source = include_str!("../contribution_lifecycle.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(production.contains("HashSet::with_capacity(removed.len())"));
    assert!(production.contains("removed_ids.extend(removed.iter().map(String::as_str))"));
    assert!(!production.contains("collect::<BTreeSet<_>>()"));
}

fn ordered_retirement_membership(ids: &[String], probes: &[String]) -> usize {
    let removed = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    probes
        .iter()
        .filter(|probe| removed.contains(probe.as_str()))
        .count()
}

fn hash_retirement_membership(ids: &[String], probes: &[String]) -> usize {
    let mut removed = HashSet::with_capacity(ids.len());
    removed.extend(ids.iter().map(String::as_str));
    probes
        .iter()
        .filter(|probe| removed.contains(probe.as_str()))
        .count()
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_id_editor614_retirement_hash_membership_performance_evidence() {
    const IDS: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let suffix = "x".repeat(128);
    let ids = (0..IDS)
        .map(|index| format!("consumer.retirement.{suffix}.{index:05}"))
        .collect::<Vec<_>>();
    let probes = (0..IDS * 2)
        .map(|index| format!("consumer.retirement.{suffix}.{:05}", index % IDS))
        .collect::<Vec<_>>();
    assert_eq!(
        ordered_retirement_membership(&ids, &probes),
        hash_retirement_membership(&ids, &probes)
    );
    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_retirement_membership(
                black_box(&ids),
                black_box(&probes),
            ));
            ordered_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(hash_retirement_membership(
                black_box(&ids),
                black_box(&probes),
            ));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_retirement_membership(
                black_box(&ids),
                black_box(&probes),
            ));
            hash_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(ordered_retirement_membership(
                black_box(&ids),
                black_box(&probes),
            ));
            ordered_samples.push(started.elapsed());
        }
    }
    ordered_samples.sort_unstable();
    hash_samples.sort_unstable();
    let ordered_p95 = ordered_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    let hash_p95 = hash_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    println!(
        "EDITOR614_HASH_RETIREMENT_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} removed_ids={IDS} probes={} package_id_bytes={} ordered_p95_ns={} hash_p95_ns={} target_ratio_bp=5000",
        probes.len(),
        ids[0].len(),
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95 <= ordered_p95.mul_f64(0.5),
        "hash retirement membership P95 {:?} exceeded 50% of ordered P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
