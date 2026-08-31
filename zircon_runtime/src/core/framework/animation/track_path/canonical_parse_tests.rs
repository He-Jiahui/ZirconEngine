use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};

use super::{
    canonical_component_property_path, canonical_entity_path, AnimationTrackPath,
    AnimationTrackPathError,
};

const SAMPLE_PAIRS: usize = 31;
const PARSES_PER_SAMPLE: usize = 100_000;
const TRACK_PATH: &str = "World/Characters/Hero/Skeleton/UpperBody/RightArm/Hand:AnimationPlayer.layered_blend.weights.right_hand";

#[test]
fn optimization_batch_20260828is_runtime291_preserves_canonical_and_normalized_paths() {
    let canonical = AnimationTrackPath::parse("Root/Hero:AnimationPlayer.weight")
        .expect("canonical track path");
    assert_eq!(canonical.as_str(), "Root/Hero:AnimationPlayer.weight");

    let normalized = AnimationTrackPath::parse(" Root / Hero : AnimationPlayer .. weight ")
        .expect("normalized track path");
    assert_eq!(normalized.as_str(), "Root/Hero:AnimationPlayer.weight");
    let (entity, property) = normalized.split().expect("normalized components");
    assert_eq!(entity.as_str(), "Root/Hero");
    assert_eq!(property.as_str(), "AnimationPlayer.weight");
}

#[test]
fn optimization_batch_20260828is_runtime291_canonical_guard_rejects_normalization_work() {
    assert!(canonical_entity_path("Root/Hero"));
    assert!(!canonical_entity_path(" Root/Hero"));
    assert!(!canonical_entity_path("Root//Hero"));
    assert!(canonical_component_property_path("AnimationPlayer.weight"));
    assert!(!canonical_component_property_path(
        "AnimationPlayer..weight"
    ));
    assert!(!canonical_component_property_path(
        "AnimationPlayer. weight"
    ));

    let source = include_str!("../track_path.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let fast_path = implementation
        .find("canonical_entity_path(entity_path)")
        .expect("canonical fast path");
    let legacy_parse = implementation
        .find("EntityPath::parse(entity_path)")
        .expect("normalizing fallback");
    assert!(fast_path < legacy_parse);
    assert!(implementation.contains("raw: raw.to_owned()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828is_runtime291_canonical_animation_track_parse_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME291_CANONICAL_ANIMATION_TRACK_PARSE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} track_path_bytes={} \
legacy_owned_path_objects_per_parse=3 optimized_owned_path_objects_per_parse=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        TRACK_PATH.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_parse(raw: &str) -> Result<String, AnimationTrackPathError> {
    let (entity_path, property_path) = raw.split_once(':').ok_or(AnimationTrackPathError)?;
    let entity_path = EntityPath::parse(entity_path).map_err(|_| AnimationTrackPathError)?;
    let property_path =
        ComponentPropertyPath::parse(property_path).map_err(|_| AnimationTrackPathError)?;
    Ok(format!("{entity_path}:{property_path}"))
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for iteration in 0..PARSES_PER_SAMPLE {
        let length = if optimized {
            AnimationTrackPath::parse(black_box(TRACK_PATH))
                .expect("optimized canonical track path")
                .as_str()
                .len()
        } else {
            legacy_parse(black_box(TRACK_PATH))
                .expect("legacy canonical track path")
                .len()
        };
        checksum ^= black_box(length.wrapping_add(iteration));
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
