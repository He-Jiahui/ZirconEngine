use std::hint::black_box;
use std::time::Instant;

use crate::asset::AssetUri;

use super::{
    fold_event, folded_entry_bytes, try_fold_bounded, AssetWatchEvent, FoldedAssetChangeMap,
};

const SAMPLE_PAIRS: usize = 17;
const EVENTS_PER_SAMPLE: usize = 16_384;

#[test]
fn runtime88_watch_fold_stack_scratch_matches_legacy_capacity_and_rollback() {
    let first = uri("res://watch/first.material");
    let second = uri("res://watch/second.material");
    let third = uri("res://watch/third.material");
    let first_bytes = folded_entry_bytes(&first, &(super::super::AssetChangeKind::Added, None));
    let events = [
        AssetWatchEvent::Added(first.clone()),
        AssetWatchEvent::Modified(first.clone()),
        AssetWatchEvent::Modified(second.clone()),
        AssetWatchEvent::Renamed {
            from: first.clone(),
            to: second.clone(),
        },
        AssetWatchEvent::Removed(third.clone()),
        AssetWatchEvent::Renamed {
            from: second.clone(),
            to: second.clone(),
        },
        AssetWatchEvent::Added(third.clone()),
        AssetWatchEvent::Renamed {
            from: third,
            to: first.clone(),
        },
        AssetWatchEvent::Removed(first),
    ];

    for entry_capacity in [0, 1, 2, 3] {
        for byte_capacity in [0, first_bytes - 1, first_bytes, first_bytes * 2, usize::MAX] {
            let mut optimized = FoldedAssetChangeMap::new();
            let mut legacy = FoldedAssetChangeMap::new();
            let mut optimized_bytes = 0;
            let mut legacy_bytes = 0;
            for event in &events {
                let expected = legacy_try_fold_bounded(
                    &mut legacy,
                    &mut legacy_bytes,
                    event.clone(),
                    entry_capacity,
                    byte_capacity,
                );
                let actual = try_fold_bounded(
                    &mut optimized,
                    &mut optimized_bytes,
                    event.clone(),
                    entry_capacity,
                    byte_capacity,
                );
                assert_eq!(actual, expected, "admission differs for {event:?}");
                assert_eq!(optimized, legacy, "fold differs for {event:?}");
                assert_eq!(
                    optimized_bytes, legacy_bytes,
                    "byte tally differs for {event:?}"
                );
            }
        }
    }
}

#[test]
fn runtime88_watch_fold_stack_scratch_rejects_rename_and_restores_source() {
    let source = uri("res://watch/source.material");
    let destination = uri("res://watch/destination.material");
    let source_bytes = folded_entry_bytes(&source, &(super::super::AssetChangeKind::Added, None));
    let mut folded = FoldedAssetChangeMap::new();
    let mut approximate_bytes = 0;

    assert!(try_fold_bounded(
        &mut folded,
        &mut approximate_bytes,
        AssetWatchEvent::Added(source.clone()),
        1,
        source_bytes,
    ));
    assert_eq!(approximate_bytes, source_bytes);
    assert!(!try_fold_bounded(
        &mut folded,
        &mut approximate_bytes,
        AssetWatchEvent::Renamed {
            from: source.clone(),
            to: destination.clone(),
        },
        1,
        source_bytes,
    ));
    assert_eq!(approximate_bytes, source_bytes);
    assert_eq!(folded.len(), 1);
    assert_eq!(folded[&source].0, super::super::AssetChangeKind::Added);
    assert!(!folded.contains_key(&destination));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime88_watch_fold_stack_scratch_bench() {
    let uri = uri("res://watch/repeated-modification.material");
    let events = vec![AssetWatchEvent::Modified(uri); EVENTS_PER_SAMPLE];
    assert_eq!(
        run_burst(&events, legacy_try_fold_bounded),
        run_burst(&events, try_fold_bounded)
    );

    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_raw.push(measure_burst(&events, legacy_try_fold_bounded));
            optimized_raw.push(measure_burst(&events, try_fold_bounded));
        } else {
            optimized_raw.push(measure_burst(&events, try_fold_bounded));
            legacy_raw.push(measure_burst(&events, legacy_try_fold_bounded));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    println!(
        "RUNTIME88_WATCH_FOLD_STACK_SCRATCH_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} sample_pairs={SAMPLE_PAIRS} events_per_sample={EVENTS_PER_SAMPLE} short_vec_allocations_per_event=2->0 legacy_raw_ns={legacy_raw:?} optimized_raw_ns={optimized_raw:?}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(95),
        "stack scratch P95 should be at most 95% of legacy"
    );
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).expect("valid watch URI")
}

type FoldFn = fn(&mut FoldedAssetChangeMap, &mut usize, AssetWatchEvent, usize, usize) -> bool;

fn run_burst(events: &[AssetWatchEvent], fold: FoldFn) -> (FoldedAssetChangeMap, usize) {
    let mut folded = FoldedAssetChangeMap::new();
    let mut approximate_bytes = 0;
    for event in events {
        assert!(fold(
            &mut folded,
            &mut approximate_bytes,
            event.clone(),
            8,
            usize::MAX,
        ));
    }
    (folded, approximate_bytes)
}

fn measure_burst(events: &[AssetWatchEvent], fold: FoldFn) -> u64 {
    let started = Instant::now();
    let result = run_burst(black_box(events), fold);
    black_box(result);
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn legacy_try_fold_bounded(
    folded: &mut FoldedAssetChangeMap,
    approximate_bytes: &mut usize,
    event: AssetWatchEvent,
    entry_capacity: usize,
    byte_capacity: usize,
) -> bool {
    let touched = match &event {
        AssetWatchEvent::Added(uri)
        | AssetWatchEvent::Modified(uri)
        | AssetWatchEvent::Removed(uri) => vec![uri.clone()],
        AssetWatchEvent::Renamed { from, to } => vec![from.clone(), to.clone()],
    };
    let previous = touched
        .iter()
        .map(|uri| (uri.clone(), folded.get(uri).cloned()))
        .collect::<Vec<_>>();
    let previous_bytes = previous
        .iter()
        .filter_map(|(uri, value)| value.as_ref().map(|value| folded_entry_bytes(uri, value)))
        .sum::<usize>();
    fold_event(folded, event);
    let next_bytes = touched
        .iter()
        .filter_map(|uri| folded.get(uri).map(|value| folded_entry_bytes(uri, value)))
        .sum::<usize>();
    let candidate_bytes = approximate_bytes
        .saturating_sub(previous_bytes)
        .saturating_add(next_bytes);
    if folded.len() <= entry_capacity && candidate_bytes <= byte_capacity {
        *approximate_bytes = candidate_bytes;
        return true;
    }
    for uri in touched {
        folded.remove(&uri);
    }
    for (uri, value) in previous {
        if let Some(value) = value {
            folded.insert(uri, value);
        }
    }
    false
}
