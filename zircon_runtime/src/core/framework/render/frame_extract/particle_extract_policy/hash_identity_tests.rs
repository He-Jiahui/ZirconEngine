use std::collections::{BTreeMap, HashMap, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::framework::render::{
    ParticleExtract, RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot,
};
use crate::core::math::{Vec2, Vec3};

const BENCHMARK_IDENTITY_COUNT: usize = 16_384;
const BENCHMARK_LOOKUP_COUNT: usize = 4_096;
const BENCHMARK_ITERATIONS: usize = 32;
const BENCHMARK_SAMPLES: usize = 17;

#[test]
fn runtime99d_batch_particle_identity_hash_index_preserves_match_policy() {
    let mut extract = ParticleExtract::default();
    extract.sprites = vec![sprite(1, 0), sprite(1, 0), sprite(2, 10), sprite(2, 11)];
    extract.previous_sprites = vec![
        previous_sprite(1, 0),
        previous_sprite(1, 0),
        previous_sprite(2, 10),
        previous_sprite(2, 11),
        previous_sprite(9, 90),
    ];

    assert_eq!(
        extract.anonymous_stream_ambiguity_entities(),
        HashSet::from([1])
    );
    assert_eq!(extract.anonymous_stream_ambiguity_sprite_count(), 2);
    assert_eq!(extract.previous_state_sprite_count(), 2);
    assert_eq!(extract.missing_previous_state_sprite_count(), 2);
}

#[test]
fn runtime99d_batch_particle_identity_hash_index_has_no_order_projection() {
    let source = include_str!("../particle_extract_policy.rs");

    assert!(source.contains("use std::collections::{HashMap, HashSet}"));
    assert!(source.contains("HashMap::with_capacity(self.previous_sprites.len())"));
    assert!(source.contains(") -> HashSet<EntityId>"));
    assert!(source.contains(") -> HashMap<EntityId, usize>"));
    assert!(source.contains("HashMap::with_capacity(sprites.len())"));
    assert!(!source.contains("BTreeMap"));
    assert!(!source.contains("BTreeSet"));
    assert!(!source.contains("sort"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn runtime99d_batch_particle_identity_hash_index_p95() {
    let identities = (0..BENCHMARK_IDENTITY_COUNT as u64)
        .map(|index| {
            (
                index.wrapping_mul(11_400_714_819_323_198_485),
                index.rotate_left(13),
            )
        })
        .collect::<Vec<_>>();
    let lookups = identities
        .iter()
        .rev()
        .take(BENCHMARK_LOOKUP_COUNT)
        .copied()
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&identities, &lookups));
            optimized_samples.push(measure_optimized(&identities, &lookups));
        } else {
            optimized_samples.push(measure_optimized(&identities, &lookups));
            legacy_samples.push(measure_legacy(&identities, &lookups));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples, 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples, 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / legacy_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME99D_PARTICLE_IDENTITY_HASH_INDEX_BENCH_V1 sample_pairs={BENCHMARK_SAMPLES} \
pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 \
iterations={BENCHMARK_ITERATIONS} identities={BENCHMARK_IDENTITY_COUNT} \
lookups={BENCHMARK_LOOKUP_COUNT} legacy_p50_ns={} legacy_p95_ns={} \
optimized_p50_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(70),
        "hash particle identity indexing must reduce build-and-lookup P95 by at least 30%: \
legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

fn sprite(entity: u64, stable_sprite_key: u64) -> RenderParticleSpriteSnapshot {
    RenderParticleSpriteSnapshot {
        entity,
        stable_sprite_key,
        ..RenderParticleSpriteSnapshot::default()
    }
}

fn previous_sprite(entity: u64, stable_sprite_key: u64) -> RenderParticlePreviousSpriteSnapshot {
    RenderParticlePreviousSpriteSnapshot {
        entity,
        stable_sprite_key,
        position: Vec3::ZERO,
        size: 1.0,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        billboard_basis: None,
    }
}

fn measure_legacy(identities: &[(u64, u64)], lookups: &[(u64, u64)]) -> Duration {
    measure_index(identities, lookups, |identities| {
        identities
            .iter()
            .copied()
            .map(|identity| (identity, 1_usize))
            .collect::<BTreeMap<_, _>>()
    })
}

fn measure_optimized(identities: &[(u64, u64)], lookups: &[(u64, u64)]) -> Duration {
    measure_index(identities, lookups, |identities| {
        identities
            .iter()
            .copied()
            .map(|identity| (identity, 1_usize))
            .collect::<HashMap<_, _>>()
    })
}

fn measure_index<M>(
    identities: &[(u64, u64)],
    lookups: &[(u64, u64)],
    mut build: impl FnMut(&[(u64, u64)]) -> M,
) -> Duration
where
    M: IdentityIndex,
{
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..BENCHMARK_ITERATIONS {
        let mut index = build(black_box(identities));
        for identity in lookups {
            if let Some(remaining) = index.remaining_mut(black_box(identity)) {
                checksum ^= *remaining;
                *remaining = remaining.saturating_sub(1);
            }
        }
        black_box(index);
    }
    black_box(checksum);
    started.elapsed()
}

trait IdentityIndex {
    fn remaining_mut(&mut self, identity: &(u64, u64)) -> Option<&mut usize>;
}

impl IdentityIndex for BTreeMap<(u64, u64), usize> {
    fn remaining_mut(&mut self, identity: &(u64, u64)) -> Option<&mut usize> {
        self.get_mut(identity)
    }
}

impl IdentityIndex for HashMap<(u64, u64), usize> {
    fn remaining_mut(&mut self, identity: &(u64, u64)) -> Option<&mut usize> {
        self.get_mut(identity)
    }
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let rank = samples.len().saturating_mul(percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}
