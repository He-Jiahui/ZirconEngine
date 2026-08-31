use std::collections::VecDeque;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::text::TextFontFaceHandle;

use super::{
    HostTextFont, RuntimeArtifactFontCache, RuntimeArtifactFontKey,
    RUNTIME_ARTIFACT_FONT_CACHE_CAPACITY,
};

const FACE_COUNT: usize = RUNTIME_ARTIFACT_FONT_CACHE_CAPACITY;
const SAMPLE_COUNT: usize = 17;
const LEGACY_KEY_COMPARISONS: usize = FACE_COUNT * FACE_COUNT;
const OPTIMIZED_HASH_LOOKUPS: usize = FACE_COUNT;

#[test]
fn optimization_batch_20260826bk_runtime_artifact_font_hash_lru_preserves_eviction() {
    let mut cache = RuntimeArtifactFontCache::default();
    for index in 0..FACE_COUNT {
        cache.insert(key(index), font(index));
    }

    let retained = cache.get(key(0)).expect("oldest entry is promoted");
    let duplicate = cache.insert(key(0), font(999));
    assert!(Arc::ptr_eq(&retained, &duplicate));
    cache.insert(key(FACE_COUNT), font(FACE_COUNT));

    assert!(cache.get(key(0)).is_some());
    assert!(cache.get(key(1)).is_none());
    assert!(cache.get(key(FACE_COUNT)).is_some());
    assert_eq!(cache.entries.len(), FACE_COUNT);
}

#[test]
fn optimization_batch_20260826bk_runtime_artifact_font_hash_lru_eliminates_hit_scan() {
    const SOURCE: &str = include_str!("../runtime_artifact.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_KEY_COMPARISONS, 4_096);
    assert_eq!(OPTIMIZED_HASH_LOOKUPS, 64);
    assert!(production.contains("HashMap<RuntimeArtifactFontKey, RuntimeArtifactFontCacheEntry>"));
    assert!(production.contains("self.entries.get_mut(&key)"));
    assert!(!production.contains("VecDeque<RuntimeArtifactFontCacheEntry>"));
    assert!(!production.contains(".iter().position"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bk_runtime_artifact_font_hash_lru_p95() {
    let keys = (0..FACE_COUNT).map(key).collect::<Vec<_>>();
    let mut legacy = VecDeque::new();
    let mut optimized = RuntimeArtifactFontCache::default();
    for index in 0..FACE_COUNT {
        let font = font(index);
        legacy.push_front((keys[index], Arc::clone(&font)));
        optimized.insert(keys[index], font);
    }

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_frame(black_box(&mut legacy), black_box(&keys)),
        || optimized_frame(black_box(&mut optimized), black_box(&keys)),
    );
    assert_eq!(
        legacy_frame(&mut legacy, &keys),
        optimized_frame(&mut optimized, &keys)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR01_RUNTIME_ARTIFACT_FONT_HASH_LRU_BENCH_V1 faces={FACE_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_key_comparisons={LEGACY_KEY_COMPARISONS} optimized_hash_lookups={OPTIMIZED_HASH_LOOKUPS} deterministic_lookup_work_reduction_percent=98.4375 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 50% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_frame(
    cache: &mut VecDeque<(RuntimeArtifactFontKey, Arc<HostTextFont>)>,
    keys: &[RuntimeArtifactFontKey],
) -> u64 {
    keys.iter()
        .filter_map(|key| {
            let index = cache.iter().position(|(cached, _)| cached == key)?;
            let entry = cache.remove(index)?;
            let value = entry.1.cache_key;
            cache.push_front(entry);
            Some(value)
        })
        .fold(0, u64::wrapping_add)
}

fn optimized_frame(cache: &mut RuntimeArtifactFontCache, keys: &[RuntimeArtifactFontKey]) -> u64 {
    keys.iter()
        .filter_map(|key| cache.get(*key))
        .map(|font| font.cache_key)
        .fold(0, u64::wrapping_add)
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for sample_index in 0..N {
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

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn key(index: usize) -> RuntimeArtifactFontKey {
    RuntimeArtifactFontKey {
        source_identity: [index as u8; 16],
        font_generation: index as u64,
        font_face: TextFontFaceHandle::new(index as u32, index as u64),
        font_instance: None,
        collection_index: index as u32,
    }
}

fn font(index: usize) -> Arc<HostTextFont> {
    Arc::new(HostTextFont {
        font: None,
        bytes: Arc::from([]),
        runtime_family: Arc::from("Benchmark Runtime Artifact"),
        weight: 400,
        collection_index: index as u32,
        cache_key: index as u64,
    })
}
