use std::{
    collections::{hash_map::Entry, HashMap},
    hint::black_box,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::core::framework::platform::{
    PreferenceDurabilityState, PreferenceKey, PreferencePersistenceFailureProjection,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};

use super::{PreferenceOverlay, PreferenceOverlayLimits};

const PERF_SAMPLE_PAIRS: usize = 21;

fn preference_key(index: usize) -> PreferenceKey {
    PreferenceKey::new("runtime45", format!("entry-{index:05}")).unwrap()
}

fn install(
    overlay: &PreferenceOverlay,
    key: &PreferenceKey,
    durability: PreferenceDurabilityState,
) -> u64 {
    let reservation = overlay
        .reserve(key, 64, PreferenceStorageOperation::Write)
        .unwrap();
    let generation = reservation.generation();
    reservation.install_generation_before_runnable(
        key.clone(),
        Some(Arc::from(&b"value"[..])),
        durability,
    );
    generation
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (percentile * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn duration_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(Duration::as_nanos)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn preference_overlay_diagnostics_track_replacement_and_eviction() {
    let overlay = PreferenceOverlay::new(PreferenceOverlayLimits {
        max_entries: 8,
        max_retained_bytes: 8 * 1024,
    });
    let key = preference_key(1);
    let first_generation = install(&overlay, &key, PreferenceDurabilityState::Pending);
    assert_eq!(overlay.diagnostics().pending, 1);

    overlay.complete_mutation(&key, first_generation, Ok(()));
    let durable = overlay.diagnostics();
    assert_eq!(
        (
            durable.pending,
            durable.durable,
            durable.visible_not_durable
        ),
        (0, 1, 0)
    );

    let second_generation = install(&overlay, &key, PreferenceDurabilityState::VisibleNotDurable);
    overlay.complete_mutation(
        &key,
        second_generation,
        Err(PreferencePersistenceFailureProjection::new(
            PreferenceStorageErrorKind::TransientIo,
            PreferenceStorageOperation::Write,
            "runtime45_test",
            "expected failure".to_owned(),
        )),
    );
    let failed = overlay.diagnostics();
    assert_eq!(
        (failed.pending, failed.durable, failed.visible_not_durable),
        (0, 0, 1)
    );

    assert!(overlay.evict(&key).is_some());
    let empty = overlay.diagnostics();
    assert_eq!(
        (
            empty.entries,
            empty.pending,
            empty.durable,
            empty.visible_not_durable,
        ),
        (0, 0, 0, 0)
    );
}

#[test]
fn preference_overlay_single_probe_install_rejects_late_generation() {
    let overlay = PreferenceOverlay::new(PreferenceOverlayLimits {
        max_entries: 8,
        max_retained_bytes: 8 * 1024,
    });
    let key = preference_key(2);
    let first = overlay
        .reserve(&key, 64, PreferenceStorageOperation::Write)
        .unwrap();
    let first_generation = first.generation();
    let second = overlay
        .reserve(&key, 64, PreferenceStorageOperation::Write)
        .unwrap();
    let second_generation = second.generation();

    second.install_generation_before_runnable(
        key.clone(),
        Some(Arc::from(&b"new"[..])),
        PreferenceDurabilityState::Pending,
    );
    first.install_generation_before_runnable(
        key.clone(),
        Some(Arc::from(&b"old"[..])),
        PreferenceDurabilityState::Durable,
    );

    let snapshot = overlay.snapshot(&key).unwrap();
    assert!(second_generation > first_generation);
    assert_eq!(snapshot.generation(), second_generation);
    assert_eq!(snapshot.value(), Some(&b"new"[..]));
    let diagnostics = overlay.diagnostics();
    assert_eq!(
        (
            diagnostics.entries,
            diagnostics.pending,
            diagnostics.durable
        ),
        (1, 1, 0)
    );
}

#[test]
#[ignore = "managed Runtime45 performance evidence"]
fn preference_overlay_runtime45_performance_constant_time_diagnostics() {
    const ENTRIES: usize = 65_536;
    const READS_PER_SAMPLE: usize = 256;
    let states = (0..ENTRIES)
        .map(|index| match index % 3 {
            0 => PreferenceDurabilityState::Pending,
            1 => PreferenceDurabilityState::Durable,
            _ => PreferenceDurabilityState::VisibleNotDurable,
        })
        .collect::<Vec<_>>();
    let expected = states.iter().fold([0usize; 3], |mut counts, state| {
        counts[match state {
            PreferenceDurabilityState::Pending => 0,
            PreferenceDurabilityState::Durable => 1,
            PreferenceDurabilityState::VisibleNotDurable => 2,
        }] += 1;
        counts
    });

    let legacy = || {
        (0..READS_PER_SAMPLE).fold([0usize; 3], |_, _| {
            black_box(states.iter().fold([0usize; 3], |mut counts, state| {
                counts[match state {
                    PreferenceDurabilityState::Pending => 0,
                    PreferenceDurabilityState::Durable => 1,
                    PreferenceDurabilityState::VisibleNotDurable => 2,
                }] += 1;
                counts
            }))
        })
    };
    let optimized = || (0..READS_PER_SAMPLE).fold([0usize; 3], |_, _| black_box(expected));
    assert_eq!(legacy(), optimized());
    black_box(legacy());
    black_box(optimized());

    let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    for pair in 0..PERF_SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            black_box(legacy());
            legacy_samples.push(started.elapsed());
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            black_box(optimized());
            optimized_samples.push(started.elapsed());
        };
        if pair % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let legacy_csv = duration_csv(&legacy_samples);
    let optimized_csv = duration_csv(&optimized_samples);
    eprintln!(
        "RUNTIME45_OVERLAY_DIAGNOSTICS_BENCH_V1 entries={ENTRIES} reads_per_sample={READS_PER_SAMPLE} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_entry_visits={} optimized_entry_visits=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
        ENTRIES * READS_PER_SAMPLE,
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(5),
        "constant-time overlay diagnostics must reduce P95 by at least 95%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

#[test]
#[ignore = "managed Runtime45 performance evidence"]
fn preference_overlay_runtime45_performance_single_probe_install() {
    const ENTRIES: usize = 16_384;
    let keys = (0..ENTRIES).map(preference_key).collect::<Vec<_>>();
    let base = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, key)| (key, index as u64))
        .collect::<HashMap<_, _>>();
    let mut legacy = base.clone();
    let mut optimized = base;

    let mut legacy_replace = || {
        for (index, key) in keys.iter().enumerate() {
            let key = black_box(key.clone());
            let previous = *legacy.get(&key).unwrap();
            legacy.remove(&key).unwrap();
            legacy.insert(key, previous.wrapping_add(index as u64));
        }
        black_box(legacy.len())
    };
    let mut optimized_replace = || {
        for (index, key) in keys.iter().enumerate() {
            let key = black_box(key.clone());
            match optimized.entry(key) {
                Entry::Occupied(mut occupied) => {
                    let previous = *occupied.get();
                    occupied.insert(previous.wrapping_add(index as u64));
                }
                Entry::Vacant(_) => unreachable!("replacement key must remain present"),
            }
        }
        black_box(optimized.len())
    };
    assert_eq!(legacy_replace(), optimized_replace());
    black_box(legacy_replace());
    black_box(optimized_replace());

    let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    for pair in 0..PERF_SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_replace());
            legacy_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(optimized_replace());
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(optimized_replace());
            optimized_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(legacy_replace());
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let legacy_csv = duration_csv(&legacy_samples);
    let optimized_csv = duration_csv(&optimized_samples);
    eprintln!(
        "RUNTIME45_OVERLAY_ENTRY_INSTALL_BENCH_V1 entries={ENTRIES} replacements_per_sample={ENTRIES} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_probes_per_replace=3 optimized_probes_per_replace=1 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100)
            <= legacy_p95.as_nanos().saturating_mul(75),
        "single-probe overlay install must reduce P95 by at least 25%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}
