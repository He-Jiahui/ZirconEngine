use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::{
    eviction_texture_path, AtlasRgba, AtlasRgbaCache, AtlasRgbaResourceIndex, ATLAS_RGBA_CACHE,
};

const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ENTRY_COUNT: usize = 64;

fn rgba(resource_key: &str, generation: u64) -> AtlasRgba {
    AtlasRgba {
        resource_key: resource_key.to_string(),
        rgba: vec![generation as u8; 4],
        generation,
    }
}

#[test]
fn optimization_batch_20260826br_sprite_atlas_rgba_hash_index_preserves_generation_removal() {
    let _: &std::sync::OnceLock<std::sync::Mutex<AtlasRgbaCache>> = &ATLAS_RGBA_CACHE;
    let mut cache = AtlasRgbaCache::default();
    let texture_path = PathBuf::from("atlases/shared.png");
    cache
        .entries
        .insert(texture_path.clone(), rgba("atlas://shared", 7));
    cache.resource_index.insert(
        "atlas://shared".to_string(),
        AtlasRgbaResourceIndex {
            generation: 7,
            texture_path: texture_path.clone(),
        },
    );
    cache.resident_bytes = 4;

    cache.remove(&texture_path);

    assert!(cache.entries.is_empty());
    assert!(cache.resource_index.is_empty());
    assert_eq!(cache.resident_bytes, 0);
}

#[test]
fn optimization_batch_20260826br_sprite_atlas_rgba_hash_index_preserves_deterministic_eviction() {
    let mut entries = HashMap::new();
    for index in (0..ENTRY_COUNT).rev() {
        entries.insert(
            PathBuf::from(format!("atlases/{index:03}.png")),
            rgba(&format!("atlas://{index:03}"), index as u64),
        );
    }

    assert_eq!(
        eviction_texture_path(&entries),
        Some(PathBuf::from("atlases/000.png"))
    );
}

fn run_ordered_workload(
    resource_index: &BTreeMap<String, PathBuf>,
    entries: &BTreeMap<PathBuf, usize>,
    resource_key: &str,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(
            resource_index
                .get(resource_key)
                .and_then(|texture_path| entries.get(texture_path)),
        );
    }
    started.elapsed().as_nanos().max(1)
}

fn run_hash_workload(
    resource_index: &HashMap<String, PathBuf>,
    entries: &HashMap<PathBuf, usize>,
    resource_key: &str,
) -> u128 {
    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(
            resource_index
                .get(resource_key)
                .and_then(|texture_path| entries.get(texture_path)),
        );
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
fn optimization_batch_20260826br_sprite_atlas_rgba_hash_index_p95() {
    let prefix = "sprite-atlas-rgba-shared-prefix/".repeat(20);
    let rows = (0..ENTRY_COUNT)
        .map(|index| {
            (
                format!("atlas://{prefix}{index:03}"),
                PathBuf::from(format!("atlases/{prefix}{index:03}.png")),
                index,
            )
        })
        .collect::<Vec<_>>();
    let target = rows.last().unwrap().0.clone();
    let ordered_resources = rows
        .iter()
        .map(|(resource, path, _)| (resource.clone(), path.clone()))
        .collect::<BTreeMap<_, _>>();
    let ordered_entries = rows
        .iter()
        .map(|(_, path, value)| (path.clone(), *value))
        .collect::<BTreeMap<_, _>>();
    let hash_resources = rows
        .iter()
        .map(|(resource, path, _)| (resource.clone(), path.clone()))
        .collect::<HashMap<_, _>>();
    let hash_entries = rows
        .into_iter()
        .map(|(_, path, value)| (path, value))
        .collect::<HashMap<_, _>>();
    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            ordered_samples.push(run_ordered_workload(
                &ordered_resources,
                &ordered_entries,
                &target,
            ));
            hash_samples.push(run_hash_workload(&hash_resources, &hash_entries, &target));
        } else {
            hash_samples.push(run_hash_workload(&hash_resources, &hash_entries, &target));
            ordered_samples.push(run_ordered_workload(
                &ordered_resources,
                &ordered_entries,
                &target,
            ));
        }
    }

    let ordered_p50 = percentile(&mut ordered_samples.clone(), 50);
    let ordered_p95 = percentile(&mut ordered_samples, 95);
    let hash_p50 = percentile(&mut hash_samples.clone(), 50);
    let hash_p95 = percentile(&mut hash_samples, 95);
    println!(
        "EDITOR01_SPRITE_ATLAS_RGBA_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} lookups_before={} lookups_after={}",
        HIT_COUNT * 2,
        HIT_COUNT * 2
    );
    assert!(
        hash_p95 * 100 <= ordered_p95 * 70,
        "HashMap two-stage lookup P95 must be at least 30% below BTreeMap lookup: ordered={ordered_p95}ns hash={hash_p95}ns"
    );
}
