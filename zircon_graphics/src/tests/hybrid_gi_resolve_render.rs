use std::sync::Arc;

use zircon_asset::ProjectAssetManager;
use zircon_math::UVec2;
use zircon_scene::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, World,
};

use crate::{
    types::{EditorOrRuntimeFrame, HybridGiPrepareFrame, HybridGiPrepareProbe},
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
    ViewportState,
};

#[test]
fn hybrid_gi_resolve_adds_radiance_cache_indirect_light_when_feature_enabled() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
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

    let baseline = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), ViewportState::new(viewport_size)),
            &compiled,
            None,
        )
        .unwrap();
    let resolved = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(viewport_size))
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [96, 120, 160],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 64,
                            irradiance_rgb: [72, 96, 136],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert!(
        average_luma(&resolved.rgba) > average_luma(&baseline.rgba) + 3.0,
        "expected hybrid GI resolve to brighten the frame when resident probes are available"
    );
}

#[test]
fn hybrid_gi_resolve_uses_prepare_probe_irradiance_colors() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
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

    let warm = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), ViewportState::new(viewport_size))
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [255, 96, 32],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 64,
                            irradiance_rgb: [224, 72, 24],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, ViewportState::new(viewport_size))
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [24, 96, 255],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 64,
                            irradiance_rgb: [32, 72, 224],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    assert!(
        average_channel(&warm.rgba, 0) > average_channel(&cool.rgba, 0) + 6.0,
        "expected warm GI irradiance to increase red output"
    );
    assert!(
        average_channel(&cool.rgba, 2) > average_channel(&warm.rgba, 2) + 6.0,
        "expected cool GI irradiance to increase blue output"
    );
}

fn build_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
    snapshot.scene.lights.clear();
    snapshot.preview.clear_color = zircon_math::Vec4::ZERO;
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![probe(200, true, 128), probe(500, true, 64)],
        trace_regions: vec![trace_region(40)],
    });
    extract
}

fn probe(probe_id: u32, resident: bool, ray_budget: u32) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position: zircon_math::Vec3::ZERO,
        radius: 1.0,
        resident,
        ray_budget,
    }
}

fn trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity: 1,
        region_id,
        bounds_center: zircon_math::Vec3::ZERO,
        bounds_radius: 1.0,
        screen_coverage: 1.0,
    }
}

fn average_luma(rgba: &[u8]) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    rgba.chunks_exact(4)
        .map(|pixel| (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0)
        .sum::<f32>()
        / (rgba.len() as f32 / 4.0)
}

fn average_channel(rgba: &[u8], channel: usize) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    rgba.chunks_exact(4)
        .map(|pixel| pixel[channel] as f32)
        .sum::<f32>()
        / (rgba.len() as f32 / 4.0)
}
