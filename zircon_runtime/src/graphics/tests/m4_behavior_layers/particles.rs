use super::super::plugin_render_feature_fixtures::particle_render_feature_descriptor;
use super::RenderFixture;
use crate::core::framework::render::{
    RenderLayerSet, RenderParticleSpriteSnapshot, RenderQualityProfile,
};
use crate::core::math::{Vec2, Vec3, Vec4};
use crate::graphics::{RenderPassExecutionContext, RenderPassExecutorRegistration};

#[test]
fn particle_rendering_draws_billboard_sprites_in_transparent_stage() {
    let fixture = RenderFixture::new("graphics_m4_particles", [0.1, 0.1, 0.1, 1.0]);
    let extract = fixture.frame_extract(Vec::new(), Vec::new(), |extract| {
        extract.particles.emitters = vec![42];
        extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
            entity: 42,
            stable_sprite_key: 0,
            position: Vec3::ZERO,
            size: 0.9,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: 0.0,
            sort_order: 0,
            color: Vec4::new(1.0, 0.48, 0.12, 0.8),
            intensity: 1.0,
            depth_test: true,
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material: None,
            texture: None,
        }];
    });

    let particle_server = fixture.server_with_render_features(
        [particle_render_feature_descriptor()],
        [RenderPassExecutorRegistration::new(
            "particle.transparent",
            particle_transparent_billboard_executor,
        )],
    );
    let particle_frame = fixture.render_extract(
        &particle_server,
        extract.clone(),
        RenderQualityProfile::new("particle-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let no_particle_frame = fixture.render_extract(
        &particle_server,
        extract,
        RenderQualityProfile::new("particle-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_particle_rendering(false),
    );

    let particle_pixels = warm_pixels(&particle_frame.rgba);
    let no_particle_pixels = warm_pixels(&no_particle_frame.rgba);
    assert!(
        particle_pixels > no_particle_pixels + 96,
        "expected particle rendering to add visible billboard pixels; particle={particle_pixels}, disabled={no_particle_pixels}"
    );
}

#[test]
fn particle_shader_preserves_sprite_alpha_for_transparent_blending() {
    let shader = include_str!("../../scene/scene_renderer/particle/shaders/particle.wgsl");

    assert!(
        shader.contains("return input.color;"),
        "particle shader should preserve vertex alpha instead of forcing opaque output"
    );
}

fn particle_transparent_billboard_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    context
        .require_gpu()?
        .record_particle_billboards_to_resources("scene-color", "scene-depth")
}

fn warm_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| {
            // Particle sprites preserve transparent alpha, so visible warm pixels are not opaque.
            pixel[3] >= 64 && pixel[0] > 28 && pixel[0] > pixel[1] && pixel[1] > pixel[2]
        })
        .count()
}
