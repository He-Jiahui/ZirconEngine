use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::super::{
    insert_cached_resolution, AtlasResolution, AtlasResolutionCacheKey, ATLAS_RESOLUTION_CACHE,
    MAX_ATLAS_RESOLUTION_CACHE_ENTRIES,
};
use crate::ui::retained_host::host_contract::paint_frame::HostPaintImageUvRect;

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

fn cache_key(
    source_key: impl Into<String>,
    source_path: impl Into<PathBuf>,
) -> AtlasResolutionCacheKey {
    AtlasResolutionCacheKey {
        source_key: source_key.into(),
        source_path: source_path.into(),
    }
}

fn resolution(resource_key: &str) -> AtlasResolution {
    AtlasResolution {
        manifest_path: PathBuf::from("atlas.toml"),
        resource_key: resource_key.to_string(),
        width: 64,
        height: 32,
        uv: HostPaintImageUvRect {
            min: [0.0, 0.0],
            max: [1.0, 1.0],
        },
    }
}

#[test]
fn optimization_batch_20260826bp_sprite_atlas_hash_index_preserves_full_key_and_negative_cache() {
    let _: &std::sync::OnceLock<
        std::sync::Mutex<HashMap<AtlasResolutionCacheKey, Option<AtlasResolution>>>,
    > = &ATLAS_RESOLUTION_CACHE;
    let mut cache = HashMap::new();
    let first = cache_key("template-icon:shared", "assets/first.png");
    let second = cache_key("template-icon:shared", "assets/second.png");

    insert_cached_resolution(&mut cache, first.clone(), None);
    insert_cached_resolution(
        &mut cache,
        second.clone(),
        Some(resolution("atlas://second")),
    );

    assert!(cache.get(&first).is_some_and(Option::is_none));
    assert_eq!(
        cache
            .get(&second)
            .and_then(Option::as_ref)
            .map(|resolved| resolved.resource_key.as_str()),
        Some("atlas://second")
    );
}

#[test]
fn optimization_batch_20260826bp_sprite_atlas_hash_index_preserves_deterministic_capacity() {
    let mut cache = HashMap::new();
    for index in (0..MAX_ATLAS_RESOLUTION_CACHE_ENTRIES).rev() {
        insert_cached_resolution(
            &mut cache,
            cache_key(format!("template-icon:{index:03}"), "assets/shared.png"),
            None,
        );
    }
    let lowest = cache_key("template-icon:000", "assets/shared.png");
    let retained = cache_key("template-icon:001", "assets/shared.png");
    let newest = cache_key("template-icon:999", "assets/shared.png");

    insert_cached_resolution(&mut cache, newest.clone(), None);

    assert_eq!(cache.len(), MAX_ATLAS_RESOLUTION_CACHE_ENTRIES);
    assert!(!cache.contains_key(&lowest));
    assert!(cache.contains_key(&retained));
    assert!(cache.contains_key(&newest));
}

fn run_ordered_workload(
    cache: &BTreeMap<AtlasResolutionCacheKey, usize>,
    key: &AtlasResolutionCacheKey,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(cache.get(key));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_hash_workload(
    cache: &HashMap<AtlasResolutionCacheKey, usize>,
    key: &AtlasResolutionCacheKey,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(cache.get(key));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &mut [u128], numerator: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * numerator).div_ceil(100).saturating_sub(1);
    samples[rank]
}

#[test]
#[ignore = "release performance gate; managed validation only"]
fn optimization_batch_20260826bp_sprite_atlas_resolution_hash_index_p95() {
    let prefix = "sprite-atlas-shared-prefix/".repeat(24);
    let entries = (0..MAX_ATLAS_RESOLUTION_CACHE_ENTRIES)
        .map(|index| {
            (
                cache_key(
                    format!("template-icon:{prefix}{index:03}"),
                    format!("assets/{prefix}{index:03}.png"),
                ),
                index,
            )
        })
        .collect::<Vec<_>>();
    let target = entries.last().unwrap().0.clone();
    let ordered = entries.iter().cloned().collect::<BTreeMap<_, _>>();
    let hashed = entries.into_iter().collect::<HashMap<_, _>>();
    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            ordered_samples.push(run_ordered_workload(&ordered, &target));
            hash_samples.push(run_hash_workload(&hashed, &target));
        } else {
            hash_samples.push(run_hash_workload(&hashed, &target));
            ordered_samples.push(run_ordered_workload(&ordered, &target));
        }
    }

    let ordered_p50 = percentile(&mut ordered_samples.clone(), 50);
    let ordered_p95 = percentile(&mut ordered_samples, 95);
    let hash_p50 = percentile(&mut hash_samples.clone(), 50);
    let hash_p95 = percentile(&mut hash_samples, 95);
    println!(
        "EDITOR01_SPRITE_ATLAS_RESOLUTION_HASH_INDEX_BENCH_V1 entries={} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95}",
        MAX_ATLAS_RESOLUTION_CACHE_ENTRIES
    );
    assert!(
        hash_p95 * 100 <= ordered_p95 * 70,
        "HashMap lookup P95 must be at least 30% below BTreeMap lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );
}
