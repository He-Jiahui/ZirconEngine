use std::collections::BTreeMap;
use std::convert::Infallible;
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::*;

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
fn optimization_batch_hx_editor607_extend_moves_disjoint_registrations() {
    let mut active = EditorRuntimeEventConsumerRegistry::default();
    active.register(registration("builtin.console")).unwrap();
    let mut incoming = EditorRuntimeEventConsumerRegistry::default();
    incoming.register(registration("plugin.weather")).unwrap();
    incoming.register(registration("plugin.lighting")).unwrap();

    active.extend(incoming).unwrap();

    assert_eq!(active.registrations().count(), 3);
    assert!(active.registration("builtin.console").is_some());
    assert!(active.registration("plugin.weather").is_some());
    assert!(active.registration("plugin.lighting").is_some());
}

#[test]
fn optimization_batch_hx_editor607_extend_rejects_late_duplicate_atomically() {
    let mut active = EditorRuntimeEventConsumerRegistry::default();
    active.register(registration("z-existing")).unwrap();

    let mut incoming = EditorRuntimeEventConsumerRegistry::default();
    incoming.register(registration("a-new")).unwrap();
    incoming.register(registration("z-existing")).unwrap();

    let error = active
        .extend(incoming)
        .expect_err("a late duplicate must reject the complete incoming batch");
    assert!(matches!(
        error,
        EditorRuntimeEventConsumerError::DuplicateConsumer { consumer_id }
            if consumer_id == "z-existing"
    ));
    assert!(active.registration("z-existing").is_some());
    assert!(active.registration("a-new").is_none());
    assert_eq!(active.registrations().count(), 1);
}

#[test]
fn optimization_batch_hx_editor607_extend_preflights_then_appends() {
    let source = include_str!("../../registration.rs");
    let body = source
        .split("pub fn extend(")
        .nth(1)
        .expect("registry extend")
        .split("pub(crate) fn extend_contribution")
        .next()
        .expect("bounded registry extend");

    assert!(body.contains("registry"));
    assert!(body.contains(".registrations"));
    assert!(body.contains(".keys()"));
    assert!(body.contains("self.registrations.contains_key"));
    assert!(body.contains("self.registrations.append(&mut registry.registrations)"));
    assert!(!body.contains("let mut candidate = self.clone()"));
}

fn benchmark_map(offset: usize, count: usize) -> BTreeMap<String, Vec<u8>> {
    (0..count)
        .map(|index| {
            (
                format!("editor.performance.consumer.{:06}", index + offset),
                vec![(index % 251) as u8; 512],
            )
        })
        .collect()
}

fn legacy_extend(base: &mut BTreeMap<String, Vec<u8>>, incoming: BTreeMap<String, Vec<u8>>) {
    let mut candidate = base.clone();
    candidate.extend(incoming);
    *base = candidate;
}

fn move_extend(base: &mut BTreeMap<String, Vec<u8>>, mut incoming: BTreeMap<String, Vec<u8>>) {
    black_box(!incoming.keys().any(|id| base.contains_key(id)));
    base.append(&mut incoming);
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hx_editor607_move_only_registry_extend_performance_evidence() {
    const BASE_COUNT: usize = 8_192;
    const INCOMING_COUNT: usize = 2_048;
    const SAMPLE_PAIRS: usize = 17;
    let base = benchmark_map(0, BASE_COUNT);
    let incoming = benchmark_map(BASE_COUNT, INCOMING_COUNT);
    let measure_legacy = || {
        let mut active = base.clone();
        let input = incoming.clone();
        let started = Instant::now();
        legacy_extend(black_box(&mut active), input);
        black_box(active);
        started.elapsed().as_nanos().max(1)
    };
    let measure_move = || {
        let mut active = base.clone();
        let input = incoming.clone();
        let started = Instant::now();
        move_extend(black_box(&mut active), input);
        black_box(active);
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_move());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut move_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            move_samples.push(measure_move());
        } else {
            move_samples.push(measure_move());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    move_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let move_p50 = move_samples[8];
    let move_p95 = move_samples[16];
    println!(
        "EDITOR607_MOVE_ONLY_REGISTRY_EXTEND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 move_first_pairs=8 base_registrations={BASE_COUNT} incoming_registrations={INCOMING_COUNT} payload_bytes=512 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} move_p50_ns={move_p50} move_p95_ns={move_p95} base_registration_clones=8192->0 target_ratio_bp=2000"
    );
    assert!(
        move_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(2_000),
        "move-only registry extend P95 {move_p95} ns exceeded 20% of legacy {legacy_p95} ns"
    );
}
