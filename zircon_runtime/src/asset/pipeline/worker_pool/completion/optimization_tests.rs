use std::hint::black_box;
use std::time::Instant;

use crate::asset::types::TextureSource;

use super::*;

const BENCHMARK_ENTRIES: usize = 4_096;
const BENCHMARK_KEY_BYTES: usize = 64;
const BENCHMARK_ITERATIONS: usize = 64;
const BENCHMARK_SAMPLES: usize = 11;

#[test]
fn runtime59_asset_expiry_in_flight_retain_preserves_waiter_report() {
    let now = Instant::now();
    let past = now.checked_sub(Duration::from_secs(1)).unwrap();
    let future = now.checked_add(Duration::from_secs(60)).unwrap();
    let mut state = empty_registry_state();
    let expired_request = request(1);
    let live_request = request(2);
    let expired_entry = Arc::new(CompletionEntry::new(expired_request.clone(), past));
    let live_entry = Arc::new(CompletionEntry::new(live_request.clone(), future));
    state
        .in_flight
        .insert(expired_request.clone(), Arc::clone(&expired_entry));
    state
        .in_flight
        .insert(live_request.clone(), Arc::clone(&live_entry));

    let report = expire_entries(&mut state, now);

    assert_eq!(report.in_flight_entries, 1);
    assert_eq!(report.in_flight_waiters, 1);
    assert!(!state.in_flight.contains_key(&expired_request));
    assert!(state.in_flight.contains_key(&live_request));
    assert!(matches!(
        expired_entry.try_add_waiter(4),
        WaiterAdmission::Terminal
    ));
}

#[test]
fn runtime59_asset_expiry_completed_retain_preserves_byte_report() {
    let now = Instant::now();
    let past = now.checked_sub(Duration::from_secs(1)).unwrap();
    let future = now.checked_add(Duration::from_secs(60)).unwrap();
    let mut state = empty_registry_state();
    let expired_request = request(3);
    let live_request = request(4);
    let expired_entry = ready_entry(expired_request.clone(), future, past);
    let live_entry = ready_entry(live_request.clone(), future, future);
    state.completed.insert(
        expired_request.clone(),
        CompletedEntry {
            entry: Arc::clone(&expired_entry),
            bytes: 24,
        },
    );
    state.completed.insert(
        live_request.clone(),
        CompletedEntry {
            entry: Arc::clone(&live_entry),
            bytes: 40,
        },
    );
    state.completed_bytes = 64;

    let report = expire_entries(&mut state, now);

    assert_eq!(report.completed_entries, 1);
    assert_eq!(report.completed_bytes, 24);
    assert_eq!(state.completed_bytes, 40);
    assert!(!state.completed.contains_key(&expired_request));
    assert!(state.completed.contains_key(&live_request));
    assert!(matches!(
        expired_entry.try_add_waiter(4),
        WaiterAdmission::Terminal
    ));

    let source = include_str!("../completion.rs");
    let expire = function_body(
        source,
        "pub(super) fn expire_entries(",
        "pub(super) fn record_expiry_for_diagnostics(",
    );
    assert_eq!(expire.matches(".retain(").count(), 2);
    assert!(!expire.contains("collect::<Vec"));
    assert!(!expire.contains("request.clone()"));
}

#[test]
#[ignore = "release performance gate; run through the managed Runtime59 validator"]
fn runtime59_asset_expiry_allocation_free_in_flight_release_benchmark() {
    let source = benchmark_entries();
    assert_eq!(
        retired_in_flight_expiry(source.clone()),
        retained_in_flight_expiry(source.clone())
    );

    let (retired_samples, optimized_samples) = measure_expiry_pairs(&source, true);
    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "RUNTIME59_ALLOCATION_FREE_IN_FLIGHT_EXPIRY_SWEEP_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank entries={} expired_entries={} key_bytes={} iterations={} retired_key_clones={} optimized_key_clones=0 retired_temporary_vectors=1 optimized_temporary_vectors=0 retired_hash_removals={} optimized_hash_removals=0 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_ENTRIES,
        BENCHMARK_ENTRIES / 2,
        BENCHMARK_KEY_BYTES,
        BENCHMARK_ITERATIONS,
        BENCHMARK_ENTRIES / 2,
        BENCHMARK_ENTRIES / 2,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(60),
        "allocation-free in-flight expiry P95 must be at most 60% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

#[test]
#[ignore = "release performance gate; run through the managed Runtime59 validator"]
fn runtime59_asset_expiry_allocation_free_completed_release_benchmark() {
    let source = benchmark_entries();
    assert_eq!(
        retired_completed_expiry(source.clone()),
        retained_completed_expiry(source.clone())
    );

    let (retired_samples, optimized_samples) = measure_expiry_pairs(&source, false);
    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "RUNTIME59_ALLOCATION_FREE_COMPLETED_EXPIRY_SWEEP_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank entries={} expired_entries={} expired_bytes={} key_bytes={} iterations={} retired_key_clones={} optimized_key_clones=0 retired_temporary_vectors=1 optimized_temporary_vectors=0 retired_hash_removals={} optimized_hash_removals=0 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_ENTRIES,
        BENCHMARK_ENTRIES / 2,
        (BENCHMARK_ENTRIES / 2) * 3,
        BENCHMARK_KEY_BYTES,
        BENCHMARK_ITERATIONS,
        BENCHMARK_ENTRIES / 2,
        BENCHMARK_ENTRIES / 2,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(60),
        "allocation-free completed expiry P95 must be at most 60% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

fn empty_registry_state() -> CompletionRegistryState {
    CompletionRegistryState {
        closing: false,
        scheduled_jobs: 0,
        in_flight: HashMap::new(),
        completed: HashMap::new(),
        completed_bytes: 0,
    }
}

fn request(id: usize) -> AssetRequest {
    AssetRequest::Texture(TextureSource::Path(format!("expiry-{id}.png")))
}

fn ready_entry(
    request: AssetRequest,
    request_deadline: Instant,
    completion_deadline: Instant,
) -> Arc<CompletionEntry> {
    let entry = Arc::new(CompletionEntry::new(request.clone(), request_deadline));
    assert!(matches!(
        entry.ready(
            Arc::new(CpuAssetPayload::Failure {
                request,
                message: "fixture".to_string(),
            }),
            completion_deadline,
        ),
        ReadyTransition::Ready
    ));
    entry
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct BenchKey {
    id: usize,
    payload: [u8; BENCHMARK_KEY_BYTES],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BenchEntry {
    expired: bool,
    waiters: usize,
    bytes: usize,
}

fn benchmark_entries() -> HashMap<BenchKey, BenchEntry> {
    (0..BENCHMARK_ENTRIES)
        .map(|id| {
            (
                BenchKey {
                    id,
                    payload: [id as u8; BENCHMARK_KEY_BYTES],
                },
                BenchEntry {
                    expired: id % 2 == 0,
                    waiters: 1,
                    bytes: 3,
                },
            )
        })
        .collect()
}

fn retired_in_flight_expiry(
    mut entries: HashMap<BenchKey, BenchEntry>,
) -> (HashMap<BenchKey, BenchEntry>, usize, usize) {
    let expired = entries
        .iter()
        .filter_map(|(key, entry)| entry.expired.then(|| key.clone()))
        .collect::<Vec<_>>();
    let mut expired_entries = 0;
    let mut expired_waiters = 0;
    for key in expired {
        if let Some(entry) = entries.remove(&key) {
            expired_entries += 1;
            expired_waiters += entry.waiters;
        }
    }
    (entries, expired_entries, expired_waiters)
}

fn retained_in_flight_expiry(
    mut entries: HashMap<BenchKey, BenchEntry>,
) -> (HashMap<BenchKey, BenchEntry>, usize, usize) {
    let mut expired_entries = 0;
    let mut expired_waiters = 0;
    entries.retain(|_, entry| {
        if entry.expired {
            expired_entries += 1;
            expired_waiters += entry.waiters;
            false
        } else {
            true
        }
    });
    (entries, expired_entries, expired_waiters)
}

fn retired_completed_expiry(
    mut entries: HashMap<BenchKey, BenchEntry>,
) -> (HashMap<BenchKey, BenchEntry>, usize, usize) {
    let expired = entries
        .iter()
        .filter_map(|(key, entry)| entry.expired.then(|| key.clone()))
        .collect::<Vec<_>>();
    let mut expired_entries = 0;
    let mut expired_bytes = 0;
    for key in expired {
        if let Some(entry) = entries.remove(&key) {
            expired_entries += 1;
            expired_bytes += entry.bytes;
        }
    }
    (entries, expired_entries, expired_bytes)
}

fn retained_completed_expiry(
    mut entries: HashMap<BenchKey, BenchEntry>,
) -> (HashMap<BenchKey, BenchEntry>, usize, usize) {
    let mut expired_entries = 0;
    let mut expired_bytes = 0;
    entries.retain(|_, entry| {
        if entry.expired {
            expired_entries += 1;
            expired_bytes += entry.bytes;
            false
        } else {
            true
        }
    });
    (entries, expired_entries, expired_bytes)
}

fn measure_expiry_pairs(
    source: &HashMap<BenchKey, BenchEntry>,
    in_flight: bool,
) -> (Vec<u128>, Vec<u128>) {
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_expiry(source, in_flight, true));
            optimized_samples.push(measure_expiry(source, in_flight, false));
        } else {
            optimized_samples.push(measure_expiry(source, in_flight, false));
            retired_samples.push(measure_expiry(source, in_flight, true));
        }
    }
    (retired_samples, optimized_samples)
}

fn measure_expiry(source: &HashMap<BenchKey, BenchEntry>, in_flight: bool, retired: bool) -> u128 {
    let mut elapsed = 0;
    for _ in 0..BENCHMARK_ITERATIONS {
        let entries = source.clone();
        let started = Instant::now();
        if in_flight {
            black_box(if retired {
                retired_in_flight_expiry(entries)
            } else {
                retained_in_flight_expiry(entries)
            });
        } else {
            black_box(if retired {
                retired_completed_expiry(entries)
            } else {
                retained_completed_expiry(entries)
            });
        }
        elapsed += started.elapsed().as_nanos();
    }
    elapsed
}

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|body| body.split(end).next())
        .expect("function source should remain available")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn reduction_basis_points(retired_ns: u128, optimized_ns: u128) -> u128 {
    if retired_ns == 0 {
        return 0;
    }
    retired_ns
        .saturating_sub(optimized_ns)
        .saturating_mul(10_000)
        / retired_ns
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
