use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::{AvatarMaskCache, AvatarMaskCacheKey, MAX_AVATAR_MASK_CACHE_ENTRIES};
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::HostPaintImagePixels;

const ENTRY_COUNT: usize = MAX_AVATAR_MASK_CACHE_ENTRIES;
const HITS_PER_FRAME: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const SHARED_RESOURCE_KEY_PREFIX_BYTES: usize = 130;

#[test]
fn optimization_batch_20260826bo_avatar_mask_hash_index_preserves_arc_lru() {
    let mut cache = AvatarMaskCache::default();
    for index in 0..ENTRY_COUNT {
        let image = image(index);
        cache.insert(AvatarMaskCacheKey::new(&image, 1.0), image);
    }

    let oldest_key = key(0);
    let resident = Arc::clone(&cache.entries.get(&oldest_key).unwrap().image.rgba);
    let cached = cache.get(&oldest_key).expect("oldest mask is promoted");
    assert!(Arc::ptr_eq(&resident, &cached.rgba));

    let newest = image(ENTRY_COUNT);
    cache.insert(AvatarMaskCacheKey::new(&newest, 1.0), newest);
    assert!(cache.entries.contains_key(&oldest_key));
    assert!(!cache.entries.contains_key(&key(1)));
    assert!(cache.entries.contains_key(&key(ENTRY_COUNT)));
    assert_eq!(cache.entries.len(), ENTRY_COUNT);
}

#[test]
fn optimization_batch_20260826bo_avatar_mask_hash_index_eliminates_ordered_lookup() {
    const SOURCE: &str = include_str!("../cache.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(ENTRY_COUNT, 64);
    assert!(production.contains("HashMap<AvatarMaskCacheKey, AvatarMaskCacheEntry>"));
    assert!(production.contains("self.entries.get_mut(key)"));
    assert!(!production.contains("BTreeMap<AvatarMaskCacheKey"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bo_avatar_mask_hash_index_p95() {
    let keys = (0..ENTRY_COUNT).map(key).collect::<Vec<_>>();
    assert_eq!(
        format!("avatar-mask:{}:", "shared-prefix".repeat(9)).len(),
        SHARED_RESOURCE_KEY_PREFIX_BYTES
    );
    let mut legacy = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, key)| (key, index as u64))
        .collect::<BTreeMap<_, _>>();
    let mut optimized = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, key)| (key, index as u64))
        .collect::<HashMap<_, _>>();
    let hot_key = keys.last().unwrap();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || lookup_frame(black_box(&mut legacy), hot_key),
        || lookup_frame(black_box(&mut optimized), hot_key),
    );
    assert_eq!(
        lookup_frame(&mut legacy, hot_key),
        lookup_frame(&mut optimized, hot_key)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR01_AVATAR_MASK_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits_per_frame={HITS_PER_FRAME} shared_resource_key_prefix_bytes={SHARED_RESOURCE_KEY_PREFIX_BYTES} samples={SAMPLE_COUNT} sample_order=alternating legacy_ordered_lookups={HITS_PER_FRAME} optimized_hash_lookups={HITS_PER_FRAME} legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95 * 7,
        "optimized P95 {optimized_p95}ns must be at least 30% below legacy P95 {legacy_p95}ns"
    );
}

trait LookupIndex {
    fn lookup_mut(&mut self, key: &AvatarMaskCacheKey) -> Option<&mut u64>;
}

impl LookupIndex for BTreeMap<AvatarMaskCacheKey, u64> {
    fn lookup_mut(&mut self, key: &AvatarMaskCacheKey) -> Option<&mut u64> {
        self.get_mut(key)
    }
}

impl LookupIndex for HashMap<AvatarMaskCacheKey, u64> {
    fn lookup_mut(&mut self, key: &AvatarMaskCacheKey) -> Option<&mut u64> {
        self.get_mut(key)
    }
}

fn lookup_frame(index: &mut impl LookupIndex, hot_key: &AvatarMaskCacheKey) -> u64 {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        let value = index.lookup_mut(hot_key).unwrap();
        *value = value.saturating_add(1);
        observed = *value;
    }
    observed
}

fn key(index: usize) -> AvatarMaskCacheKey {
    AvatarMaskCacheKey::new(&image(index), 1.0)
}

fn image(index: usize) -> HostPaintImagePixels {
    HostPaintImagePixels {
        resource_key: format!("avatar-mask:{}:{index:04}", "shared-prefix".repeat(9)),
        width: 1,
        height: 1,
        rgba: vec![index as u8; 4].into(),
        atlas: None,
    }
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for index in 0..N {
        if index % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * percentile / 100]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
