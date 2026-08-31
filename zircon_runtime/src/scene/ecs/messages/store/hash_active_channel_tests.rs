use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

const CHANNEL_COUNT: u64 = 32_768;
const FRAME_COUNT: usize = 8;
const SAMPLE_PAIRS: usize = 21;

struct AlphaMessage;
struct BetaMessage;

impl Message for AlphaMessage {}
impl Message for BetaMessage {}

#[test]
fn optimization_batch_20260826cm_runtime130_hash_active_channels_preserve_frame_membership() {
    let mut store = MessageStore::default();
    store.write(AlphaMessage);
    store.write(BetaMessage);
    store.clear::<BetaMessage>();

    store.advance_frame();
    assert_eq!(store.last_advance_channel_visits(), 1);
    assert_eq!(store.messages::<AlphaMessage>().unwrap().len(), 1);
    assert!(store.messages::<BetaMessage>().unwrap().is_empty());

    store.clear::<AlphaMessage>();
    store.advance_frame();
    assert_eq!(store.last_advance_channel_visits(), 0);
}

#[test]
fn optimization_batch_20260826cm_runtime130_active_channel_owner_uses_hash_membership() {
    let source = include_str!("../store.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("active_channels: HashSet<TypeId>"));
    assert!(source.contains("active_channel_spare: HashSet<TypeId>"));
    assert!(source.contains("self.active_channel_spare.drain()"));
    assert!(!source.contains("active_channels: BTreeSet<TypeId>"));
    assert!(!source.contains("let active_channels = std::mem::take"));
}

fn legacy_cycle() -> usize {
    let mut active = BTreeSet::new();
    for channel in 0..CHANNEL_COUNT {
        active.insert(channel);
    }
    for channel in (0..CHANNEL_COUNT).step_by(2) {
        active.remove(&channel);
    }
    for channel in (0..CHANNEL_COUNT).step_by(2) {
        active.insert(channel);
    }
    for _ in 0..FRAME_COUNT {
        let ready = std::mem::take(&mut active);
        for channel in ready {
            black_box(channel);
            active.insert(channel);
        }
    }
    active.len()
}

fn optimized_cycle() -> usize {
    let mut active = HashSet::with_capacity(CHANNEL_COUNT as usize);
    let mut spare = HashSet::with_capacity(CHANNEL_COUNT as usize);
    for channel in 0..CHANNEL_COUNT {
        active.insert(channel);
    }
    for channel in (0..CHANNEL_COUNT).step_by(2) {
        active.remove(&channel);
    }
    for channel in (0..CHANNEL_COUNT).step_by(2) {
        active.insert(channel);
    }
    for _ in 0..FRAME_COUNT {
        std::mem::swap(&mut active, &mut spare);
        active.clear();
        for channel in spare.drain() {
            black_box(channel);
            active.insert(channel);
        }
    }
    active.len()
}

fn elapsed_ns(run: impl FnOnce() -> usize) -> u128 {
    let started = Instant::now();
    assert_eq!(black_box(run()), CHANNEL_COUNT as usize);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826cm_runtime130_hash_active_channels_performance_evidence() {
    for _ in 0..3 {
        assert_eq!(black_box(legacy_cycle()), CHANNEL_COUNT as usize);
        assert_eq!(black_box(optimized_cycle()), CHANNEL_COUNT as usize);
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(legacy_cycle));
            optimized_samples.push(elapsed_ns(optimized_cycle));
        } else {
            optimized_samples.push(elapsed_ns(optimized_cycle));
            legacy_samples.push(elapsed_ns(legacy_cycle));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "RUNTIME130_MESSAGE_STORE_HASH_ACTIVE_CHANNELS_BENCH_V1 sample_pairs={} channel_count={} frame_count={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        CHANNEL_COUNT,
        FRAME_COUNT,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "reused hash active-channel membership p95 must be at least 30% below tree membership: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
