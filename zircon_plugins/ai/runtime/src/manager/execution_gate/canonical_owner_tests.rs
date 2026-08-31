use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::plugin::PluginModuleId;

use super::{canonicalize_owners, release_owners, BehaviorNodeExecutionGate, ExecutionGateState};

const OWNER_COUNT: usize = 2_048;
const BENCHMARK_ITERATIONS: usize = 2_048;
const BENCHMARK_SAMPLES: usize = 21;
const RELEASE_BENCHMARK_ITERATIONS: usize = 65_536;

#[test]
fn canonical_owner_list_preserves_empty_and_sorted_inputs() {
    let mut empty = Vec::new();
    canonicalize_owners(&mut empty);
    assert!(empty.is_empty());

    let mut sorted = vec![owner(1), owner(3), owner(5)];
    canonicalize_owners(&mut sorted);
    assert_eq!(sorted, [owner(1), owner(3), owner(5)]);
}

#[test]
fn canonical_owner_list_sorts_and_deduplicates_general_inputs() {
    let mut owners = vec![owner(5), owner(1), owner(3), owner(1), owner(5)];

    canonicalize_owners(&mut owners);

    assert_eq!(owners, [owner(1), owner(3), owner(5)]);
}

#[test]
fn canonical_owner_list_has_a_single_provider_fast_path() {
    let source = include_str!("../execution_gate.rs");
    let canonicalize = source
        .split("fn canonicalize_owners(")
        .nth(1)
        .and_then(|body| body.split("\n}").next())
        .expect("canonicalize_owners body");

    assert!(canonicalize.contains("owners[1..].iter().all"));
    assert!(canonicalize.contains("owners.truncate(1)"));
    assert!(canonicalize.contains("return;"));
}

#[test]
fn release_only_notifies_when_a_revoking_owner_becomes_idle() {
    let owner = owner(11);
    let mut state = ExecutionGateState::default();
    state.in_flight.insert(owner, 2);
    state.revoking.insert(owner);

    assert!(!release_owners(&mut state, &[owner]));
    assert_eq!(state.in_flight.get(&owner), Some(&1));
    assert!(release_owners(&mut state, &[owner]));
    assert!(!state.in_flight.contains_key(&owner));

    state.in_flight.insert(owner, 1);
    state.revoking.clear();
    assert!(!release_owners(&mut state, &[owner]));
}

#[test]
fn normal_release_skips_the_condvar_broadcast() {
    let source = include_str!("../execution_gate.rs");
    let release = source
        .split("fn release(&self, owners:")
        .nth(1)
        .and_then(|body| body.split("\n    }").next())
        .expect("release body");

    assert!(release.contains("if release_owners(&mut state, owners)"));
    assert!(!release.contains("\n        self.inner.idle.notify_all();"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_provider_owner_canonicalization_release_benchmark_evidence() {
    let fixture = vec![owner(7); OWNER_COUNT];
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut checksum = 0_u64;
            for _ in 0..BENCHMARK_ITERATIONS {
                let mut owners = fixture.clone();
                legacy_canonicalize_owners(&mut owners);
                checksum += black_box(owners.len()) as u64;
            }
            checksum
        },
        || {
            let mut checksum = 0_u64;
            for _ in 0..BENCHMARK_ITERATIONS {
                let mut owners = fixture.clone();
                canonicalize_owners(&mut owners);
                checksum += black_box(owners.len()) as u64;
            }
            checksum
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_single_provider_owner_canonicalization owners={OWNER_COUNT} iterations_per_sample={BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLES} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_sort_calls_per_iteration=1 optimized_sort_calls_per_iteration=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn normal_release_notification_elision_release_benchmark_evidence() {
    let owner = owner(13);
    let legacy_gate = BehaviorNodeExecutionGate::default();
    let optimized_gate = BehaviorNodeExecutionGate::default();
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || benchmark_normal_releases(&legacy_gate, owner, true),
        || benchmark_normal_releases(&optimized_gate, owner, false),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_normal_release_notification_elision iterations_per_sample={RELEASE_BENCHMARK_ITERATIONS} owners_per_release=1 concurrent_in_flight_per_owner=2 revoking_owners=0 samples={BENCHMARK_SAMPLES} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_notify_all_calls_per_sample={RELEASE_BENCHMARK_ITERATIONS} optimized_notify_all_calls_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must not exceed legacy P95 {legacy_p95}ns"
    );
}

fn owner(raw: u32) -> PluginModuleId {
    PluginModuleId::from_raw(raw)
}

fn legacy_canonicalize_owners(owners: &mut Vec<PluginModuleId>) {
    owners.sort_by_key(|owner| owner.raw());
    owners.dedup();
}

fn benchmark_normal_releases(
    gate: &BehaviorNodeExecutionGate,
    owner: PluginModuleId,
    legacy: bool,
) -> u64 {
    let mut checksum = 0_u64;
    for _ in 0..RELEASE_BENCHMARK_ITERATIONS {
        {
            let mut state = gate
                .inner
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.in_flight.insert(owner, 2);
        }
        if legacy {
            legacy_release(gate, owner);
        } else {
            gate.release(&[owner]);
        }
        checksum += 1;
    }
    checksum
}

fn legacy_release(gate: &BehaviorNodeExecutionGate, owner: PluginModuleId) {
    let mut state = gate
        .inner
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(count) = state.in_flight.get_mut(&owner) {
        *count -= 1;
        if *count == 0 {
            state.in_flight.remove(&owner);
        }
    }
    drop(state);
    gate.inner.idle.notify_all();
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample_index in 0..BENCHMARK_SAMPLES {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
