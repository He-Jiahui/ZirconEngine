use super::{
    RenderFixture, average_channel, centered_quad_transform, fullscreen_quad_transform, ring_luma,
};
use crate::core::framework::render::{
    RenderBloomSettings, RenderColorGradingSettings, RenderLayerSet, RenderMeshSnapshot,
    RenderQualityProfile,
};
use crate::core::math::{Vec3, Vec4};
use crate::scene::components::{Mobility, default_render_layer_mask};

#[test]
fn bloom_quality_profile_spreads_bright_pixels_when_enabled() {
    let fixture = RenderFixture::new("graphics_m4_bloom", [1.0, 1.0, 1.0, 1.0]);
    let extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: centered_quad_transform(0.35),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            common: crate::core::framework::render::RendererCommon {
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
                ..Default::default()
            },
        }],
        Vec::new(),
        |extract| {
            extract.post_process.bloom = RenderBloomSettings {
                threshold: 0.55,
                intensity: 1.0,
                radius: 1.0,
            };
        },
    );

    let server = fixture.server();
    let bloom_on = fixture.render_extract(
        &server,
        extract.clone(),
        RenderQualityProfile::new("bloom-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let bloom_off = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("bloom-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_bloom(false),
    );

    let bloom_ring = ring_luma(&bloom_on.rgba, fixture.viewport_size, 0.18, 0.42);
    let no_bloom_ring = ring_luma(&bloom_off.rgba, fixture.viewport_size, 0.18, 0.42);
    assert!(
        bloom_ring > no_bloom_ring + 6.0,
        "expected bloom to brighten neighboring pixels; bloom ring={bloom_ring:.2}, no-bloom ring={no_bloom_ring:.2}"
    );
}

#[test]
fn color_grading_extract_tints_scene_after_post_process() {
    let fixture = RenderFixture::new("graphics_m4_color_grading", [0.72, 0.72, 0.72, 1.0]);
    let extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: fullscreen_quad_transform(),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            common: crate::core::framework::render::RendererCommon {
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
                ..Default::default()
            },
        }],
        Vec::new(),
        |extract| {
            extract.post_process.color_grading = RenderColorGradingSettings {
                exposure: 1.18,
                contrast: 1.08,
                saturation: 0.92,
                gamma: 0.95,
                tint: Vec3::new(1.12, 0.78, 0.58),
            };
        },
    );

    let server = fixture.server();
    let graded = fixture.render_extract(
        &server,
        extract.clone(),
        RenderQualityProfile::new("grade-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let neutral = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("grade-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_color_grading(false),
    );

    let graded_red = average_channel(&graded.rgba, 0);
    let graded_blue = average_channel(&graded.rgba, 2);
    let neutral_red = average_channel(&neutral.rgba, 0);
    let neutral_blue = average_channel(&neutral.rgba, 2);
    assert!(
        (graded_red - graded_blue) > (neutral_red - neutral_blue) + 12.0,
        "expected color grading tint to bias warm channels; graded delta={:.2}, neutral delta={:.2}",
        graded_red - graded_blue,
        neutral_red - neutral_blue
    );
}
