use serde_json::json;

use super::*;

#[test]
fn render_particle_extract_scans_dynamic_component_owners_instead_of_all_entities() {
    let source = include_str!("../render_particles.rs");
    let collect = source
        .split("pub(super) fn collect_render_particles")
        .nth(1)
        .and_then(|source| source.split("#[derive(Clone, Debug, PartialEq)]").next())
        .expect("read render particle collection body");

    assert!(
        collect.contains("self.dynamic_components.keys().copied().collect::<Vec<_>>()")
            && collect.contains("dynamic_component_entities.sort_unstable();")
            && collect.contains("for entity in dynamic_component_entities")
            && collect.contains("let Some(components) = self.dynamic_components.get(&entity)")
            && collect.contains("components.get(component_id)")
            && !collect.contains("for entity in self.entities.iter().copied()")
            && !collect.contains("self.dynamic_component(entity, component_id)")
            && !collect.contains("emitters.sort_unstable();")
            && !collect.contains("bounds.sort_by_key"),
        "render particle extraction must scan dynamic-component owners instead of probing every world entity"
    );
}

#[test]
fn optimization_wave_20260825vw_runtime26_particle_extract_reuses_frame_storage() {
    let source = include_str!("../render_particles.rs");
    let collect = source
        .split("pub(super) fn collect_render_particles")
        .nth(1)
        .and_then(|source| source.split("#[derive(Clone, Debug, PartialEq)]").next())
        .expect("read render particle collection body");

    assert!(collect.contains("let sprite_start = sprites.len();"));
    assert!(collect.contains("let entity_sprites = &sprites[sprite_start..];"));
    assert!(collect.contains("std::array::from_fn(|_| None)"));
    assert!(!collect.contains("let mut entity_sprites = Vec::new();"));
    assert!(!collect.contains("let mut entity_gpu_bounds = Vec::new();"));
    assert!(!collect.contains("sprites.extend(entity_sprites);"));
}

fn particle_scratch_checksum(
    checksum: u64,
    entity: EntityId,
    sprites: &[RenderParticleSpriteSnapshot],
    bounds: &RenderParticleBoundsSnapshot,
) -> u64 {
    checksum.rotate_left(7)
        ^ entity
        ^ sprites.len() as u64
        ^ sprites[0].stable_sprite_key
        ^ u64::from(bounds.radius.to_bits())
}

fn legacy_particle_extract_scratch_workload(
    particle: &serde_json::Value,
    entity_count: usize,
) -> u64 {
    let mut sprites = Vec::with_capacity(entity_count);
    let mut checksum = 0_u64;
    for entity in 0..entity_count as u64 {
        let mut entity_sprites = Vec::new();
        let mut entity_gpu_bounds = Vec::new();
        collect_particle_sprites_from_value(entity, u32::MAX, particle, &mut entity_sprites);
        entity_gpu_bounds.push(RenderParticleBoundsSnapshot {
            entity,
            center: Vec3::new(1.0, 2.0, 3.0),
            radius: 0.5,
        });
        std::hint::black_box((&entity_sprites, &entity_gpu_bounds));
        checksum =
            particle_scratch_checksum(checksum, entity, &entity_sprites, &entity_gpu_bounds[0]);
        sprites.extend(entity_sprites);
    }
    std::hint::black_box(sprites.len() as u64 ^ checksum)
}

fn optimized_particle_extract_scratch_workload(
    particle: &serde_json::Value,
    entity_count: usize,
) -> u64 {
    let mut sprites = Vec::with_capacity(entity_count);
    let mut checksum = 0_u64;
    for entity in 0..entity_count as u64 {
        let sprite_start = sprites.len();
        let gpu_bounds = [
            Some(RenderParticleBoundsSnapshot {
                entity,
                center: Vec3::new(1.0, 2.0, 3.0),
                radius: 0.5,
            }),
            None,
        ];
        collect_particle_sprites_from_value(entity, u32::MAX, particle, &mut sprites);
        let entity_sprites = &sprites[sprite_start..];
        std::hint::black_box((entity_sprites, &gpu_bounds));
        checksum = particle_scratch_checksum(
            checksum,
            entity,
            entity_sprites,
            gpu_bounds[0].as_ref().expect("benchmark bound"),
        );
    }
    std::hint::black_box(sprites.len() as u64 ^ checksum)
}

fn measure_particle_extract_scratch(operation: impl FnOnce() -> u64) -> (u128, u64) {
    let started = std::time::Instant::now();
    let checksum = std::hint::black_box(operation());
    (started.elapsed().as_nanos(), checksum)
}

fn particle_scratch_percentile(mut samples: [u128; 21], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn particle_scratch_reduction_percent(legacy_ns: u128, optimized_ns: u128) -> f64 {
    legacy_ns.saturating_sub(optimized_ns) as f64 * 100.0 / legacy_ns as f64
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260825vw_runtime26_particle_extract_scratch_evidence() {
    const ENTITY_COUNT: usize = 100_000;
    const WARMUP_PAIRS: usize = 4;
    const SAMPLE_PAIRS: usize = 21;
    const TARGET_MILLIS: u128 = 500;
    const MARKER: &str = "RUNTIME26_PARTICLE_EXTRACT_SCRATCH_BENCH_V1";

    let particle = json!({
        "position": [1.0, 2.0, 3.0],
        "size": 0.25,
        "stable_sprite_key": 7
    });
    for _ in 0..WARMUP_PAIRS {
        std::hint::black_box(legacy_particle_extract_scratch_workload(
            &particle,
            ENTITY_COUNT,
        ));
        std::hint::black_box(optimized_particle_extract_scratch_workload(
            &particle,
            ENTITY_COUNT,
        ));
    }

    let mut legacy_ns_raw = [0_u128; SAMPLE_PAIRS];
    let mut optimized_ns_raw = [0_u128; SAMPLE_PAIRS];
    let mut checksum = None;
    for sample_index in 0..SAMPLE_PAIRS {
        let (legacy, optimized) = if sample_index % 2 == 0 {
            (
                measure_particle_extract_scratch(|| {
                    legacy_particle_extract_scratch_workload(&particle, ENTITY_COUNT)
                }),
                measure_particle_extract_scratch(|| {
                    optimized_particle_extract_scratch_workload(&particle, ENTITY_COUNT)
                }),
            )
        } else {
            let optimized = measure_particle_extract_scratch(|| {
                optimized_particle_extract_scratch_workload(&particle, ENTITY_COUNT)
            });
            let legacy = measure_particle_extract_scratch(|| {
                legacy_particle_extract_scratch_workload(&particle, ENTITY_COUNT)
            });
            (legacy, optimized)
        };
        assert_eq!(legacy.1, optimized.1);
        let expected_checksum = *checksum.get_or_insert(legacy.1);
        assert_eq!(expected_checksum, legacy.1);
        legacy_ns_raw[sample_index] = legacy.0;
        optimized_ns_raw[sample_index] = optimized.0;
    }

    let p50_legacy_ns = particle_scratch_percentile(legacy_ns_raw, 50);
    let p50_optimized_ns = particle_scratch_percentile(optimized_ns_raw, 50);
    let p95_legacy_ns = particle_scratch_percentile(legacy_ns_raw, 95);
    let p95_optimized_ns = particle_scratch_percentile(optimized_ns_raw, 95);
    let p50_reduction_percent = particle_scratch_reduction_percent(p50_legacy_ns, p50_optimized_ns);
    let p95_reduction_percent = particle_scratch_reduction_percent(p95_legacy_ns, p95_optimized_ns);
    println!(
        "{MARKER} emitters={ENTITY_COUNT} legacy_transient_buffers={} \
             optimized_transient_buffers=0 reduction_pct=100.00 \
             p50_legacy_ns={p50_legacy_ns} p50_optimized_ns={p50_optimized_ns} \
             p50_reduction_percent={p50_reduction_percent:.4} \
             p95_legacy_ns={p95_legacy_ns} p95_optimized_ns={p95_optimized_ns} \
             p95_reduction_percent={p95_reduction_percent:.4} \
             checksum={} target_ms={TARGET_MILLIS} \
             legacy_ns_raw={legacy_ns_raw:?} optimized_ns_raw={optimized_ns_raw:?}",
        ENTITY_COUNT * 2,
        checksum.expect("samples record a checksum"),
    );

    assert!(p50_reduction_percent >= 15.0);
    assert!(p95_reduction_percent >= 5.0);
    assert!(
        p95_optimized_ns <= TARGET_MILLIS * 1_000_000,
        "{MARKER} p95_optimized_ns={p95_optimized_ns} target_ms={TARGET_MILLIS}"
    );
}

#[test]
fn world_hud_bar_sprites_use_nonzero_stable_keys() {
    let mut sprites = Vec::new();

    collect_world_hud_bar_sprites_from_value(
        42,
        1 << 4,
        &json!({
            "position": [0.0, 1.0, 2.0],
            "ratio": 0.5
        }),
        &mut sprites,
    );

    assert_eq!(stable_sprite_keys(&sprites), vec![1, 2]);
    assert!(sprites
        .iter()
        .all(|sprite| sprite.render_layer_mask.to_scene_schema_v1_mask_lossy() == 1 << 4));
}

#[test]
fn world_hud_bar_array_sprites_use_bar_indexed_stable_keys() {
    let mut sprites = Vec::new();

    collect_world_hud_bar_sprites_from_value(
        42,
        u32::MAX,
        &json!({
            "bars": [
                { "position": [0.0, 1.0, 2.0], "ratio": 0.5 },
                { "position": [0.0, 2.0, 2.0], "ratio": 1.0 }
            ]
        }),
        &mut sprites,
    );

    assert_eq!(stable_sprite_keys(&sprites), vec![1, 2, 3, 4]);
}

#[test]
fn world_hud_bar_sprites_use_overlay_depth_path() {
    let mut sprites = Vec::new();

    collect_world_hud_bar_sprites_from_value(
        42,
        u32::MAX,
        &json!({
            "position": [0.0, 1.0, 2.0],
            "ratio": 0.5
        }),
        &mut sprites,
    );

    assert!(!sprites.is_empty());
    assert!(sprites.iter().all(|sprite| !sprite.depth_test));
}

#[test]
fn authored_particle_sprites_keep_depth_test_path() {
    let sprite = particle_sprite(
        42,
        1 << 5,
        &json!({
            "position": [0.0, 1.0, 2.0],
            "size": 0.25
        }),
    )
    .expect("valid authored particle sprite");

    assert!(sprite.depth_test);
    assert_eq!(
        sprite.render_layer_mask.to_scene_schema_v1_mask_lossy(),
        1 << 5
    );
}

#[test]
fn particle_gpu_frame_contribution_defaults_indirect_args_to_alive_count() {
    let contribution = particle_gpu_frame_contribution(&json!({
        "gpu_frame": {
            "alive_count": 5,
            "spawned_total": 7,
            "per_emitter_spawned": [2, 5],
            "bounds": {
                "center": [1.0, 2.0, 3.0],
                "radius": 4.0
            }
        }
    }))
    .expect("gpu frame should parse");

    assert_eq!(contribution.frame.alive_count, 5);
    assert_eq!(contribution.frame.spawned_total, 7);
    assert_eq!(contribution.frame.per_emitter_spawned, vec![2, 5]);
    assert_eq!(contribution.frame.indirect_draw_args, [6, 5, 0, 0]);
    assert_eq!(
        contribution.bounds,
        Some(RenderParticleBoundsSnapshot {
            entity: 0,
            center: Vec3::new(1.0, 2.0, 3.0),
            radius: 4.0
        })
    );
}

#[test]
fn particle_gpu_frame_builder_aggregates_scene_visible_emitters() {
    let mut builder = ParticleGpuFrameBuilder::default();
    builder.push(RenderParticleGpuFrameExtract {
        alive_count: 2,
        spawned_total: 3,
        per_emitter_spawned: vec![3],
        indirect_draw_args: [6, 2, 0, 0],
    });
    builder.push(RenderParticleGpuFrameExtract {
        alive_count: 4,
        spawned_total: 5,
        per_emitter_spawned: vec![2, 3],
        indirect_draw_args: [6, 4, 0, 0],
    });

    let frame = builder.finish().expect("aggregate gpu frame");

    assert_eq!(frame.alive_count, 6);
    assert_eq!(frame.spawned_total, 8);
    assert_eq!(frame.per_emitter_spawned, vec![3, 2, 3]);
    assert_eq!(frame.indirect_draw_args, [6, 6, 0, 0]);
}

fn stable_sprite_keys(sprites: &[RenderParticleSpriteSnapshot]) -> Vec<u64> {
    sprites
        .iter()
        .map(|sprite| sprite.stable_sprite_key)
        .collect()
}
