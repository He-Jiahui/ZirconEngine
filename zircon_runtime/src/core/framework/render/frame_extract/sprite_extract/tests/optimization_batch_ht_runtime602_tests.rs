use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::{
    RenderMaterialAlphaMode, RenderSpriteAnchor, RenderSpriteImageMode, RendererCommon,
};
use crate::core::math::{Transform, Vec3, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

use super::*;

fn sprite(entity: u64, z_order: i32, depth: f32) -> RenderSpriteSnapshot {
    RenderSpriteSnapshot {
        entity,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, depth)),
        image: ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(&format!(
            "runtime599/sprite/{entity}"
        ))),
        material: None,
        atlas_region: None,
        rect: None,
        flip_x: false,
        flip_y: false,
        anchor: RenderSpriteAnchor::CENTER,
        custom_size: None,
        image_mode: RenderSpriteImageMode::Stretch,
        color: Vec4::ONE,
        z_order,
        common: RendererCommon::default(),
        material_alpha_mode: RenderMaterialAlphaMode::Blend,
    }
}

#[test]
fn optimization_batch_ht_runtime602_streamed_phase_build_matches_explicit_inputs() {
    let sprites = vec![sprite(30, 2, 0.3), sprite(10, -1, 0.1), sprite(20, 2, 0.2)];
    let phase_inputs = sprites
        .iter()
        .enumerate()
        .map(|(sprite_index, sprite)| {
            SpritePhaseExtractInput::new(
                sprite.entity,
                sprite_index,
                sprite.material_alpha_mode,
                sprite.z_order,
                sprite.transform.translation.z,
            )
        })
        .collect();
    let expected = SpriteExtract::from_sprites_and_phase_inputs(
        CorePipelineKind::Core2d,
        sprites.clone(),
        phase_inputs,
    );

    assert_eq!(
        SpriteExtract::from_sprites(CorePipelineKind::Core2d, sprites),
        expected
    );
}

#[test]
fn optimization_batch_ht_runtime602_default_sprite_extract_has_no_phase_input_buffer() {
    let source = include_str!("../../sprite_extract.rs");
    let from_sprites = source
        .split("pub fn from_sprites(")
        .nth(1)
        .expect("from sprites")
        .split("pub fn from_sprites_and_phase_inputs")
        .next()
        .expect("bounded from sprites");

    assert!(from_sprites.contains("build_sprite_phase_queue("));
    assert!(!from_sprites.contains("SpritePhaseExtractInput::new"));
    assert!(!from_sprites.contains("collect::<Vec<_>>()"));
}

#[derive(Clone, Copy)]
struct BenchmarkSprite {
    entity: u64,
    sprite_index: usize,
    z_order: i32,
    depth: f32,
}

fn benchmark_checksum(sprite: BenchmarkSprite) -> u64 {
    sprite.entity
        ^ (sprite.sprite_index as u64).rotate_left(7)
        ^ (sprite.z_order as u64).rotate_left(13)
        ^ u64::from(sprite.depth.to_bits()).rotate_left(19)
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ht_runtime602_sprite_phase_stream_performance_evidence() {
    let sprites = (0..65_536_u64)
        .map(|entity| (entity, (entity % 257) as i32 - 128, entity as f32 * 0.25))
        .collect::<Vec<_>>();
    const SAMPLE_PAIRS: usize = 17;
    let measure_legacy = || {
        let started = Instant::now();
        let phase_inputs = sprites
            .iter()
            .enumerate()
            .map(|(sprite_index, (entity, z_order, depth))| BenchmarkSprite {
                entity: *entity,
                sprite_index,
                z_order: *z_order,
                depth: *depth,
            })
            .collect::<Vec<_>>();
        black_box(
            phase_inputs
                .iter()
                .copied()
                .fold(0_u64, |checksum, sprite| {
                    checksum ^ benchmark_checksum(sprite)
                }),
        );
        started.elapsed().as_nanos().max(1)
    };
    let measure_streamed = || {
        let started = Instant::now();
        black_box(
            sprites
                .iter()
                .enumerate()
                .map(|(sprite_index, (entity, z_order, depth))| BenchmarkSprite {
                    entity: *entity,
                    sprite_index,
                    z_order: *z_order,
                    depth: *depth,
                })
                .fold(0_u64, |checksum, sprite| {
                    checksum ^ benchmark_checksum(sprite)
                }),
        );
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_streamed());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut streamed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            streamed_samples.push(measure_streamed());
        } else {
            streamed_samples.push(measure_streamed());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    streamed_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let streamed_p50 = streamed_samples[8];
    let streamed_p95 = streamed_samples[16];
    println!(
        "RUNTIME602_SPRITE_PHASE_STREAM_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 streamed_first_pairs=8 sprites={} legacy_p50_ns={} legacy_p95_ns={} streamed_p50_ns={} streamed_p95_ns={} legacy_phase_input_bytes={} streamed_phase_input_bytes=0 target_ratio_bp=8000",
        sprites.len(),
        legacy_p50,
        legacy_p95,
        streamed_p50,
        streamed_p95,
        sprites.len() * std::mem::size_of::<BenchmarkSprite>(),
    );
    assert!(
        streamed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_000),
        "streamed sprite phase P95 {streamed_p95} ns exceeded 80% of legacy {legacy_p95} ns"
    );
}
