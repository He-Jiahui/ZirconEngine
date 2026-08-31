use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::{
    insert_cached_manifest, AtlasCacheKey, ATLAS_MANIFEST_CACHE, MAX_ATLAS_MANIFEST_CACHE_ENTRIES,
};
use zircon_runtime::asset::SpriteAtlasAsset;

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

fn cache_key(path: impl Into<PathBuf>) -> AtlasCacheKey {
    AtlasCacheKey { path: path.into() }
}

#[test]
fn optimization_batch_20260826bq_sprite_atlas_manifest_hash_index_preserves_negative_cache() {
    let _: &std::sync::OnceLock<
        std::sync::Mutex<HashMap<AtlasCacheKey, Option<SpriteAtlasAsset>>>,
    > = &ATLAS_MANIFEST_CACHE;
    let mut cache = HashMap::new();
    let first = cache_key("assets/atlases/first.toml");
    let second = cache_key("assets/atlases/second.toml");

    insert_cached_manifest(&mut cache, first.clone(), None);
    insert_cached_manifest(&mut cache, second.clone(), None);

    assert!(cache.get(&first).is_some_and(Option::is_none));
    assert!(cache.get(&second).is_some_and(Option::is_none));
}

#[test]
fn optimization_batch_20260826bq_sprite_atlas_manifest_hash_index_preserves_deterministic_capacity()
{
    let mut cache = HashMap::new();
    for index in (0..MAX_ATLAS_MANIFEST_CACHE_ENTRIES).rev() {
        insert_cached_manifest(
            &mut cache,
            cache_key(format!("assets/atlases/{index:03}.toml")),
            None,
        );
    }
    let lowest = cache_key("assets/atlases/000.toml");
    let retained = cache_key("assets/atlases/001.toml");
    let newest = cache_key("assets/atlases/999.toml");

    insert_cached_manifest(&mut cache, newest.clone(), None);

    assert_eq!(cache.len(), MAX_ATLAS_MANIFEST_CACHE_ENTRIES);
    assert!(!cache.contains_key(&lowest));
    assert!(cache.contains_key(&retained));
    assert!(cache.contains_key(&newest));
}

fn run_ordered_workload(cache: &BTreeMap<AtlasCacheKey, usize>, key: &AtlasCacheKey) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(cache.get(key));
    }
    started.elapsed().as_nanos().max(1)
}

fn run_hash_workload(cache: &HashMap<AtlasCacheKey, usize>, key: &AtlasCacheKey) -> u128 {
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
fn optimization_batch_20260826bq_sprite_atlas_manifest_hash_index_p95() {
    let prefix = "sprite-atlas-manifest-shared-prefix/".repeat(20);
    let entries = (0..MAX_ATLAS_MANIFEST_CACHE_ENTRIES)
        .map(|index| (cache_key(format!("assets/{prefix}{index:03}.toml")), index))
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
        "EDITOR01_SPRITE_ATLAS_MANIFEST_HASH_INDEX_BENCH_V1 entries={} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95}",
        MAX_ATLAS_MANIFEST_CACHE_ENTRIES
    );
    assert!(
        hash_p95 * 100 <= ordered_p95 * 70,
        "HashMap lookup P95 must be at least 30% below BTreeMap lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );
}
