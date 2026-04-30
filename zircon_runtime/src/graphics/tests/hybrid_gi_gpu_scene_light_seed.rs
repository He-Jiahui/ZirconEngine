use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    RenderPointLightSnapshot, RenderSceneSnapshot, RenderSpotLightSnapshot,
    RenderWorldSnapshotHandle,
};
use crate::core::math::{UVec2, Vec3};
use crate::graphics::tests::plugin_render_feature_fixtures::hybrid_gi_render_feature_descriptor;
use crate::scene::world::World;
use crate::{
    types::{HybridGiPrepareFrame, HybridGiPrepareUpdateRequest, ViewportRenderFrame},
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

fn compile_hybrid_gi_pipeline(extract: &RenderFrameExtract) -> CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            extract,
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(
                    RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
                )
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap()
}

fn hybrid_gi_scene_renderer() -> SceneRenderer {
    SceneRenderer::new_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [hybrid_gi_render_feature_descriptor()],
    )
    .unwrap()
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_point_and_spot_light_seed_changes() {
    let mut renderer = hybrid_gi_scene_renderer();
    let viewport_size = UVec2::new(96, 64);
    let pending_probe = probe(300, Vec3::new(0.05, 0.0, 0.0), 0.85);
    let trace_region = trace_region(40, Vec3::ZERO, 0.8, 0.9);
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 32,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let red_point = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract_with_split_lights(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region],
            vec![point_light(
                100,
                Vec3::new(1.0, 0.1, 0.1),
                4.0,
                Vec3::new(0.0, 0.0, 1.5),
                5.0,
            )],
            Vec::new(),
        ),
        prepare.clone(),
    );
    let blue_spot = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract_with_split_lights(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region],
            Vec::new(),
            vec![spot_light(
                200,
                Vec3::new(0.0, 0.0, 1.75),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.1, 0.2, 1.0),
                5.0,
                5.0,
                0.2,
                0.6,
            )],
        ),
        prepare,
    );

    let red_rgb = red_point
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("red point-light probe irradiance");
    let blue_rgb = blue_spot
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("blue spot-light probe irradiance");

    assert!(
        red_rgb[0] > blue_rgb[0],
        "expected point lights to contribute to the GPU scene-light seed red channel; red_point={red_rgb:?}, blue_spot={blue_rgb:?}"
    );
    assert!(
        blue_rgb[2] > red_rgb[2],
        "expected spot lights to contribute to the GPU scene-light seed blue channel; red_point={red_rgb:?}, blue_spot={blue_rgb:?}"
    );
}

fn render_hybrid_gi_gpu_readback(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
) -> Vec<(u32, [u8; 3])> {
    let compiled = compile_hybrid_gi_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare)),
            &compiled,
            None,
        )
        .unwrap();

    renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback")
        .probe_irradiance_rgb()
        .to_vec()
}

fn build_extract_with_split_lights(
    viewport_size: UVec2,
    probe_budget: u32,
    tracing_budget: u32,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
    point_lights: Vec<RenderPointLightSnapshot>,
    spot_lights: Vec<RenderSpotLightSnapshot>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.point_lights = point_lights;
    snapshot.scene.spot_lights = spot_lights;
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget,
        tracing_budget,
        probes,
        trace_regions,
    });
    extract
}

fn probe(probe_id: u32, position: Vec3, radius: f32) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position,
        radius,
        parent_probe_id: None,
        resident: false,
        ray_budget: 128,
    }
}

fn trace_region(
    region_id: u32,
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_coverage: f32,
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity: 1,
        region_id,
        bounds_center,
        bounds_radius,
        screen_coverage,
        rt_lighting_rgb: [0, 0, 0],
    }
}

fn point_light(
    node_id: u64,
    color: Vec3,
    intensity: f32,
    position: Vec3,
    range: f32,
) -> RenderPointLightSnapshot {
    RenderPointLightSnapshot {
        node_id,
        position,
        color,
        intensity,
        range,
    }
}

fn spot_light(
    node_id: u64,
    position: Vec3,
    direction: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
    inner_angle_radians: f32,
    outer_angle_radians: f32,
) -> RenderSpotLightSnapshot {
    RenderSpotLightSnapshot {
        node_id,
        position,
        direction,
        color,
        intensity,
        range,
        inner_angle_radians,
        outer_angle_radians,
    }
}
