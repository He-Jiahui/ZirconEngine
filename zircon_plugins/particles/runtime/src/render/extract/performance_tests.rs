use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderParticleBoundsSnapshot, RenderParticleGpuFrameExtract, RenderParticleSpriteSnapshot,
};
use zircon_runtime::core::math::{Real, Vec3};

use crate::{
    ParticleEmitterHandle, ParticleEmitterState, ParticleRuntimeSnapshot, ParticleSimulationBackend,
};

use super::{build_bounds, build_gpu_frame};

const EMITTER_COUNT: usize = 4_096;
const SPRITE_COUNT: usize = 65_536;
const BOUNDS_ENTITY_COUNT: usize = 1_024;
const GPU_FRAME_ITERATIONS: usize = 128;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

#[test]
fn optimization_batch_20260830da_extract_helpers_preserve_projection_contracts() {
    let snapshot = ParticleRuntimeSnapshot {
        emitters: vec![
            emitter(1, 9, ParticleSimulationBackend::Cpu),
            emitter(2, 7, ParticleSimulationBackend::Gpu),
            emitter(3, 11, ParticleSimulationBackend::Gpu),
        ],
        ..ParticleRuntimeSnapshot::default()
    };

    assert_eq!(build_gpu_frame(&snapshot), legacy_gpu_frame(&snapshot));

    let sprites = vec![
        sprite(7, Vec3::new(-2.0, 1.0, 3.0), 2.0),
        sprite(3, Vec3::new(4.0, 0.0, -1.0), 4.0),
        sprite(7, Vec3::new(2.0, 5.0, -3.0), 1.0),
    ];
    let optimized = build_bounds(&sprites, 2);

    assert_eq!(optimized, legacy_bounds(&sprites));
    assert_eq!(
        optimized
            .iter()
            .map(|bounds| bounds.entity)
            .collect::<Vec<_>>(),
        vec![3, 7]
    );
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830da_gpu_frame_skips_filtered_reference_staging() {
    let snapshot = benchmark_snapshot();
    let (legacy_p95, optimized_p95) = paired_p95(
        || {
            for _ in 0..GPU_FRAME_ITERATIONS {
                black_box(legacy_gpu_frame(black_box(&snapshot)));
            }
        },
        || {
            for _ in 0..GPU_FRAME_ITERATIONS {
                black_box(build_gpu_frame(black_box(&snapshot)));
            }
        },
    );
    let improvement = improvement_percent(legacy_p95, optimized_p95);

    println!(
        "PERF_RESULT task=runtime171_particle_gpu_frame_single_pass emitters={EMITTER_COUNT} iterations={GPU_FRAME_ITERATIONS} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}"
    );
    assert!(
        improvement >= REQUIRED_IMPROVEMENT_PERCENT,
        "GPU frame extraction must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

#[test]
#[ignore = "release performance contract"]
fn optimization_batch_20260830da_bounds_hash_aggregation_beats_tree_updates() {
    let sprites = benchmark_sprites();
    let (legacy_p95, optimized_p95) = paired_p95(
        || black_box(legacy_bounds(black_box(&sprites))),
        || black_box(build_bounds(black_box(&sprites), BOUNDS_ENTITY_COUNT)),
    );
    let improvement = improvement_percent(legacy_p95, optimized_p95);

    println!(
        "PERF_RESULT task=runtime171_particle_bounds_hash_aggregation sprites={SPRITE_COUNT} entities={BOUNDS_ENTITY_COUNT} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}"
    );
    assert_eq!(
        build_bounds(&sprites, BOUNDS_ENTITY_COUNT),
        legacy_bounds(&sprites)
    );
    assert!(
        improvement >= REQUIRED_IMPROVEMENT_PERCENT,
        "particle bounds aggregation must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn emitter(
    entity: u64,
    live_particles: usize,
    backend: ParticleSimulationBackend,
) -> ParticleEmitterState {
    ParticleEmitterState {
        handle: ParticleEmitterHandle::new(entity + 1),
        emitter_id: format!("emitter-{entity}"),
        entity,
        live_particles,
        allocated_particles: live_particles,
        playing: true,
        backend,
        fallback_to_cpu: false,
    }
}

fn sprite(entity: u64, position: Vec3, size: Real) -> RenderParticleSpriteSnapshot {
    RenderParticleSpriteSnapshot {
        entity,
        position,
        size,
        ..RenderParticleSpriteSnapshot::default()
    }
}

fn benchmark_snapshot() -> ParticleRuntimeSnapshot {
    ParticleRuntimeSnapshot {
        emitters: (0..EMITTER_COUNT)
            .map(|index| {
                emitter(
                    index as u64,
                    index % 257,
                    if index % 3 == 0 {
                        ParticleSimulationBackend::Cpu
                    } else {
                        ParticleSimulationBackend::Gpu
                    },
                )
            })
            .collect(),
        ..ParticleRuntimeSnapshot::default()
    }
}

fn benchmark_sprites() -> Vec<RenderParticleSpriteSnapshot> {
    let mut sprites = (0..SPRITE_COUNT)
        .map(|index| {
            sprite(
                (index * 2_653 % BOUNDS_ENTITY_COUNT) as u64,
                Vec3::new(
                    index as Real * 0.01,
                    (index % 173) as Real,
                    (index % 97) as Real,
                ),
                0.5 + (index % 11) as Real * 0.1,
            )
        })
        .collect::<Vec<_>>();
    sprites.sort_unstable_by_key(|sprite| {
        (sprite.entity.wrapping_mul(97) + sprite.position.y as u64) % 1_021
    });
    sprites
}

fn legacy_gpu_frame(snapshot: &ParticleRuntimeSnapshot) -> Option<RenderParticleGpuFrameExtract> {
    let gpu_emitters = snapshot
        .emitters
        .iter()
        .filter(|emitter| emitter.backend == ParticleSimulationBackend::Gpu)
        .collect::<Vec<_>>();
    if gpu_emitters.is_empty() {
        return None;
    }
    let alive_count = gpu_emitters
        .iter()
        .map(|emitter| emitter.live_particles as u32)
        .sum::<u32>();
    Some(RenderParticleGpuFrameExtract {
        alive_count,
        spawned_total: alive_count,
        per_emitter_spawned: gpu_emitters
            .iter()
            .map(|emitter| emitter.live_particles as u32)
            .collect(),
        indirect_draw_args: [6, alive_count, 0, 0],
    })
}

fn legacy_bounds(sprites: &[RenderParticleSpriteSnapshot]) -> Vec<RenderParticleBoundsSnapshot> {
    let mut ranges: BTreeMap<_, (Vec3, Vec3)> = BTreeMap::new();
    for sprite in sprites {
        let half = Vec3::splat(sprite.size.max(0.0) * 0.5);
        let min = sprite.position - half;
        let max = sprite.position + half;
        ranges
            .entry(sprite.entity)
            .and_modify(|(current_min, current_max)| {
                *current_min = current_min.min(min);
                *current_max = current_max.max(max);
            })
            .or_insert((min, max));
    }
    ranges
        .into_iter()
        .map(|(entity, (min, max))| RenderParticleBoundsSnapshot {
            entity,
            center: (min + max) * 0.5,
            radius: (max - (min + max) * 0.5).length(),
        })
        .collect()
}

fn paired_p95(mut legacy: impl FnMut(), mut optimized: impl FnMut()) -> (u128, u128) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    (
        legacy_samples[SAMPLE_PAIRS * 95 / 100],
        optimized_samples[SAMPLE_PAIRS * 95 / 100],
    )
}

fn measure(operation: &mut impl FnMut()) -> u128 {
    let started = Instant::now();
    operation();
    started.elapsed().as_nanos()
}

fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
    legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
}
