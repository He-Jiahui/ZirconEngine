use std::sync::Arc;

use zircon_asset::ProjectAssetManager;
use zircon_math::UVec2;
use zircon_scene::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, World,
};

use crate::{
    types::{
        EditorOrRuntimeFrame, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareUpdateRequest,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
    ViewportState,
};

#[test]
fn hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        2,
        1,
        vec![
            probe(200, true, 64),
            probe(500, true, 32),
            probe(300, false, 128),
        ],
        vec![trace_region(40), trace_region(50)],
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
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
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(viewport_size))
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 64,
                            irradiance_rgb: [255, 64, 32],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 32,
                            irradiance_rgb: [32, 96, 255],
                        },
                    ],
                    pending_updates: vec![HybridGiPrepareUpdateRequest {
                        probe_id: 300,
                        ray_budget: 128,
                        generation: 9,
                    }],
                    scheduled_trace_region_ids: vec![40, 50],
                    evictable_probe_ids: vec![500],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback");
    assert_eq!(readback.cache_entries, vec![(200, 0), (500, 1)]);
    assert_eq!(readback.completed_probe_ids, vec![300]);
    assert_eq!(readback.completed_trace_region_ids, vec![40]);
    assert_eq!(
        readback.probe_irradiance_rgb,
        vec![
            (200, expected_gpu_irradiance(200, 0, 64, 2, 1, false)),
            (500, expected_gpu_irradiance(500, 1, 32, 2, 1, false)),
            (300, expected_gpu_irradiance(300, 2, 128, 2, 1, true)),
        ]
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_respects_probe_budget_without_evictable_slots() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        2,
        vec![probe(200, true, 64), probe(300, false, 128)],
        vec![trace_region(40), trace_region(50)],
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination)
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
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(viewport_size))
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 64,
                        irradiance_rgb: [255, 64, 32],
                    }],
                    pending_updates: vec![HybridGiPrepareUpdateRequest {
                        probe_id: 300,
                        ray_budget: 128,
                        generation: 10,
                    }],
                    scheduled_trace_region_ids: vec![40, 50],
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback");
    assert_eq!(readback.cache_entries, vec![(200, 0)]);
    assert_eq!(readback.completed_probe_ids, Vec::<u32>::new());
    assert_eq!(readback.completed_trace_region_ids, vec![40, 50]);
    assert_eq!(
        readback.probe_irradiance_rgb,
        vec![(200, expected_gpu_irradiance(200, 0, 64, 2, 2, false))]
    );
}

fn build_extract(
    viewport_size: UVec2,
    probe_budget: u32,
    tracing_budget: u32,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
    snapshot.scene.lights.clear();
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget,
        tracing_budget,
        probes,
        trace_regions,
    });
    extract
}

fn probe(probe_id: u32, resident: bool, ray_budget: u32) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position: zircon_math::Vec3::ZERO,
        radius: 0.5,
        resident,
        ray_budget,
    }
}

fn trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity: 1,
        region_id,
        bounds_center: zircon_math::Vec3::ZERO,
        bounds_radius: 0.5,
        screen_coverage: 1.0,
    }
}

fn expected_gpu_irradiance(
    probe_id: u32,
    slot_or_index: u32,
    ray_budget: u32,
    trace_region_count: u32,
    tracing_budget: u32,
    pending_completion: bool,
) -> [u8; 3] {
    let pending_bias = if pending_completion { 97 } else { 13 };
    let seed = probe_id
        .wrapping_mul(17)
        .wrapping_add(slot_or_index.wrapping_mul(31))
        .wrapping_add(ray_budget.wrapping_mul(7))
        .wrapping_add(trace_region_count.wrapping_mul(19))
        .wrapping_add(tracing_budget.wrapping_mul(23))
        .wrapping_add(pending_bias);
    [
        channel(seed, 0x1f),
        channel(seed.rotate_left(7), 0x3d),
        channel(seed.rotate_left(13), 0x59),
    ]
}

fn channel(seed: u32, bias: u32) -> u8 {
    let value = 48 + ((seed.wrapping_add(bias)) % 160);
    value as u8
}
