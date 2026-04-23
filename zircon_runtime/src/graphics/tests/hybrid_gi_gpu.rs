use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe,
    RenderHybridGiTraceRegion, RenderMeshSnapshot, RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::scene::world::World;

use crate::{
    runtime::HybridGiRuntimeState,
    types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareUpdateRequest, HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
        HybridGiResolveRuntime, HybridGiScenePrepareFrame, ViewportRenderFrame,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
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
            probe(200, true, 64, Vec3::new(0.25, 0.5, -0.75), 0.65),
            probe(500, true, 32, Vec3::new(-0.5, 0.25, 0.5), 0.4),
            probe(300, false, 128, Vec3::new(1.0, -0.25, 0.75), 0.9),
        ],
        vec![
            trace_region(40, Vec3::new(0.5, 0.25, -0.5), 0.75, 0.8),
            trace_region(50, Vec3::new(-0.25, 0.75, 0.5), 0.4, 0.35),
        ],
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
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
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
                }),
            ),
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
            (
                200,
                expected_gpu_irradiance(
                    &probe(200, true, 64, Vec3::new(0.25, 0.5, -0.75), 0.65),
                    0,
                    [255, 64, 32],
                    &[
                        trace_region(40, Vec3::new(0.5, 0.25, -0.5), 0.75, 0.8),
                        trace_region(50, Vec3::new(-0.25, 0.75, 0.5), 0.4, 0.35),
                    ],
                    &[40, 50],
                    1,
                    false,
                ),
            ),
            (
                500,
                expected_gpu_irradiance(
                    &probe(500, true, 32, Vec3::new(-0.5, 0.25, 0.5), 0.4),
                    1,
                    [32, 96, 255],
                    &[
                        trace_region(40, Vec3::new(0.5, 0.25, -0.5), 0.75, 0.8),
                        trace_region(50, Vec3::new(-0.25, 0.75, 0.5), 0.4, 0.35),
                    ],
                    &[40, 50],
                    1,
                    false,
                ),
            ),
            (
                300,
                expected_gpu_irradiance(
                    &probe(300, false, 128, Vec3::new(1.0, -0.25, 0.75), 0.9),
                    2,
                    [0, 0, 0],
                    &[
                        trace_region(40, Vec3::new(0.5, 0.25, -0.5), 0.75, 0.8),
                        trace_region(50, Vec3::new(-0.25, 0.75, 0.5), 0.4, 0.35),
                    ],
                    &[40, 50],
                    1,
                    true,
                ),
            ),
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
        vec![
            probe(200, true, 64, Vec3::new(-0.75, 0.125, -0.25), 0.55),
            probe(300, false, 128, Vec3::new(0.75, 0.5, 0.375), 0.8),
        ],
        vec![
            trace_region(40, Vec3::new(0.125, 0.25, 0.5), 0.35, 0.6),
            trace_region(50, Vec3::new(-0.5, 0.75, -0.125), 0.9, 0.95),
        ],
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
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
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
                }),
            ),
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
        vec![(
            200,
            expected_gpu_irradiance(
                &probe(200, true, 64, Vec3::new(-0.75, 0.125, -0.25), 0.55),
                0,
                [255, 64, 32],
                &[
                    trace_region(40, Vec3::new(0.125, 0.25, 0.5), 0.35, 0.6),
                    trace_region(50, Vec3::new(-0.5, 0.75, -0.125), 0.9, 0.95),
                ],
                &[40, 50],
                2,
                false,
            ),
        )]
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_probe_or_trace_scene_data_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let base_probe = probe(200, true, 64, Vec3::new(0.0, 0.25, -0.5), 0.6);
    let moved_probe = probe(200, true, 64, Vec3::new(1.0, 0.25, -0.5), 0.6);
    let base_region = trace_region(40, Vec3::new(0.25, 0.5, 0.0), 0.7, 0.5);
    let moved_region = trace_region(40, Vec3::new(-0.75, 0.5, 0.0), 1.1, 0.9);

    let baseline = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(viewport_size, 1, 1, vec![base_probe], vec![base_region]),
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [96, 96, 96],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    let moved = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(viewport_size, 1, 1, vec![moved_probe], vec![moved_region]),
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [96, 96, 96],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    assert_ne!(
        baseline, moved,
        "expected Hybrid GI GPU irradiance updates to change when probe/trace scene data changes"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_previous_irradiance_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 64, Vec3::new(0.0, 0.25, -0.5), 0.6)],
        vec![trace_region(40, Vec3::new(0.25, 0.5, 0.0), 0.7, 0.5)],
    );

    let warm = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        extract.clone(),
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [220, 48, 32],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    let cool = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        extract,
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [32, 96, 220],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    assert_ne!(
        warm, cool,
        "expected Hybrid GI GPU irradiance updates to change when previous radiance-cache history changes"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_without_scheduled_trace_regions_keeps_resident_history_and_zeroes_pending_updates(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        2,
        1,
        vec![
            probe(200, true, 64, Vec3::new(0.0, 0.25, -0.5), 0.6),
            probe(300, false, 128, Vec3::new(0.25, 0.125, -0.25), 0.7),
        ],
        vec![trace_region(40, Vec3::new(0.125, 0.25, 0.0), 0.7, 0.5)],
    );

    let readback = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        extract,
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [220, 96, 48],
            }],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 128,
                generation: 12,
            }],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    assert_eq!(
        readback,
        vec![(200, [220, 96, 48]), (300, [0, 0, 0])],
        "expected Hybrid GI GPU updates to depend on scheduled trace regions: without any trace work, resident probes should keep previous history and pending probes should not synthesize new radiance"
    );
}

#[test]
fn hybrid_gi_gpu_trace_lighting_readback_uses_runtime_hierarchy_rt_lighting_after_schedule_clears()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe_with_parent(
            200,
            100,
            true,
            128,
            Vec3::new(0.0, 0.0, 0.0),
            0.85,
        )],
        vec![trace_region(40, Vec3::new(0.0, 0.0, 0.0), 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(HybridGiResolveRuntime {
            probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                200,
                HybridGiResolveRuntime::pack_resolve_weight_q8(1.85),
            )]),
            probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
                200,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.95, 0.32, 0.12], 0.6),
            )]),
            ..Default::default()
        }),
    );
    let cool = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(HybridGiResolveRuntime {
            probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                200,
                HybridGiResolveRuntime::pack_resolve_weight_q8(1.85),
            )]),
            probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
                200,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.32, 0.95], 0.6),
            )]),
            ..Default::default()
        }),
    );

    let warm_trace = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("warm trace-lighting probe");
    let cool_trace = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("cool trace-lighting probe");

    assert!(
        warm_trace[0] > cool_trace[0] + 20,
        "expected GPU prepare to keep consuming warm hierarchy RT-lighting continuation from runtime after the current trace schedule clears, instead of collapsing to the same flat source; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
    assert!(
        cool_trace[2] > warm_trace[2] + 20,
        "expected GPU prepare to keep consuming cool hierarchy RT-lighting continuation from runtime after the current trace schedule clears, instead of collapsing to the same flat source; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_concentrates_radiance_on_probes_near_scheduled_trace_regions()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let readback = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            2,
            1,
            vec![
                probe(200, true, 96, Vec3::new(0.0, 0.0, 0.0), 0.9),
                probe(500, true, 96, Vec3::new(4.0, 4.0, 4.0), 0.9),
            ],
            vec![trace_region(40, Vec3::new(0.1, 0.0, 0.0), 0.85, 0.9)],
        ),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 96,
                    irradiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareProbe {
                    probe_id: 500,
                    slot: 1,
                    ray_budget: 96,
                    irradiance_rgb: [0, 0, 0],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let near_luma = readback
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| average_rgb_luma(*rgb))
        .expect("near probe irradiance");
    let far_luma = readback
        .iter()
        .find(|(probe_id, _)| *probe_id == 500)
        .map(|(_, rgb)| average_rgb_luma(*rgb))
        .expect("far probe irradiance");

    assert!(
        near_luma > far_luma + 20.0,
        "expected traced radiance to concentrate on probes near the scheduled trace region; near_luma={near_luma:.2}, far_luma={far_luma:.2}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_normalizes_multi_region_radiance_instead_of_additive_saturation(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let pending_probe = probe(300, false, 96, Vec3::new(0.0, 0.0, 0.0), 0.85);
    let left_region = trace_region(40, Vec3::new(-0.1, 0.0, 0.0), 0.7, 0.85);
    let right_region = trace_region(50, Vec3::new(0.1, 0.0, 0.0), 0.7, 0.85);

    let left_only = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            1,
            2,
            vec![pending_probe],
            vec![left_region, right_region],
        ),
        HybridGiPrepareFrame {
            resident_probes: Vec::new(),
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 96,
                generation: 20,
            }],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    let right_only = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            1,
            2,
            vec![pending_probe],
            vec![left_region, right_region],
        ),
        HybridGiPrepareFrame {
            resident_probes: Vec::new(),
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 96,
                generation: 21,
            }],
            scheduled_trace_region_ids: vec![50],
            evictable_probe_ids: Vec::new(),
        },
    );
    let combined = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            1,
            2,
            vec![pending_probe],
            vec![left_region, right_region],
        ),
        HybridGiPrepareFrame {
            resident_probes: Vec::new(),
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 96,
                generation: 22,
            }],
            scheduled_trace_region_ids: vec![40, 50],
            evictable_probe_ids: Vec::new(),
        },
    );

    let left_luma = average_rgb_luma(left_only[0].1);
    let right_luma = average_rgb_luma(right_only[0].1);
    let combined_luma = average_rgb_luma(combined[0].1);
    let min_single = left_luma.min(right_luma);
    let max_single = left_luma.max(right_luma);

    assert!(
        combined_luma >= min_single - 5.0,
        "expected combined multi-region radiance to stay near the single-region band instead of collapsing; left={left_luma:.2}, right={right_luma:.2}, combined={combined_luma:.2}"
    );
    assert!(
        combined_luma <= max_single + 5.0,
        "expected combined multi-region radiance to normalize blend energy instead of additive saturation; left={left_luma:.2}, right={right_luma:.2}, combined={combined_luma:.2}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_directional_light_color_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let pending_probe = probe(300, false, 128, Vec3::new(0.05, 0.0, 0.0), 0.85);
    let trace_region = trace_region(40, Vec3::ZERO, 0.8, 0.9);
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 30,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let warm = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract_with_lights(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region],
            vec![directional_light(Vec3::new(1.0, 0.45, 0.2), 3.0)],
        ),
        prepare.clone(),
    );
    let cool = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract_with_lights(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region],
            vec![directional_light(Vec3::new(0.2, 0.45, 1.0), 3.0)],
        ),
        prepare,
    );
    let warm_rgb = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm probe irradiance");
    let cool_rgb = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool probe irradiance");

    assert!(
        warm_rgb[0] > cool_rgb[0],
        "expected warm directional light tint to increase Hybrid GI red output; warm={warm_rgb:?}, cool={cool_rgb:?}"
    );
    assert!(
        cool_rgb[2] > warm_rgb[2],
        "expected cool directional light tint to increase Hybrid GI blue output; warm={warm_rgb:?}, cool={cool_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_uses_trace_region_rt_lighting_when_present() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let pending_probe = probe(300, false, 128, Vec3::new(0.05, 0.0, 0.0), 0.85);
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 33,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let warm = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                0.8,
                0.9,
                [255, 48, 24],
            )],
        ),
        prepare.clone(),
    );
    let cool = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                0.8,
                0.9,
                [24, 48, 255],
            )],
        ),
        prepare,
    );
    let warm_rgb = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm rt-lit probe irradiance");
    let cool_rgb = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool rt-lit probe irradiance");

    assert!(
        warm_rgb[0] > cool_rgb[0],
        "expected trace-region RT lighting to bias Hybrid GI red output; warm={warm_rgb:?}, cool={cool_rgb:?}"
    );
    assert!(
        cool_rgb[2] > warm_rgb[2],
        "expected trace-region RT lighting to bias Hybrid GI blue output; warm={warm_rgb:?}, cool={cool_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_directional_light_intensity_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let pending_probe = probe(300, false, 128, Vec3::new(0.05, 0.0, 0.0), 0.85);
    let trace_region = trace_region(40, Vec3::ZERO, 0.8, 0.9);
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 31,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let dim = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract_with_lights(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region],
            vec![directional_light(Vec3::new(1.0, 0.5, 0.25), 0.2)],
        ),
        prepare.clone(),
    );
    let bright = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract_with_lights(
            viewport_size,
            1,
            1,
            vec![pending_probe],
            vec![trace_region],
            vec![directional_light(Vec3::new(1.0, 0.5, 0.25), 1.0)],
        ),
        prepare,
    );
    let dim_luma = average_rgb_luma(
        dim.iter()
            .find(|(probe_id, _)| *probe_id == 300)
            .map(|(_, rgb)| *rgb)
            .expect("dim probe irradiance"),
    );
    let bright_luma = average_rgb_luma(
        bright
            .iter()
            .find(|(probe_id, _)| *probe_id == 300)
            .map(|(_, rgb)| *rgb)
            .expect("bright probe irradiance"),
    );

    assert!(
        bright_luma > dim_luma + 8.0,
        "expected stronger directional light intensity to increase Hybrid GI radiance energy; dim={dim_luma:.2}, bright={bright_luma:.2}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_gathers_radiance_from_nearby_resident_probes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let pending_probe = probe(300, false, 128, Vec3::ZERO, 0.85);
    let trace_region = trace_region(40, Vec3::ZERO, 0.8, 0.9);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [240, 80, 40],
            },
            HybridGiPrepareProbe {
                probe_id: 500,
                slot: 1,
                ray_budget: 96,
                irradiance_rgb: [40, 80, 240],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 32,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let warm_near = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(0.05, 0.0, 0.0), 0.85),
                pending_probe,
                probe(500, true, 96, Vec3::new(4.0, 4.0, 4.0), 0.85),
            ],
            vec![trace_region],
        ),
        prepare.clone(),
    );
    let cool_near = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(4.0, 4.0, 4.0), 0.85),
                pending_probe,
                probe(500, true, 96, Vec3::new(0.05, 0.0, 0.0), 0.85),
            ],
            vec![trace_region],
        ),
        prepare,
    );
    let warm_near_rgb = warm_near
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm-near pending probe irradiance");
    let cool_near_rgb = cool_near
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool-near pending probe irradiance");

    assert!(
        warm_near_rgb[0] > cool_near_rgb[0],
        "expected nearby warm resident probes to bias gathered pending radiance toward red; warm_near={warm_near_rgb:?}, cool_near={cool_near_rgb:?}"
    );
    assert!(
        cool_near_rgb[2] > warm_near_rgb[2],
        "expected nearby cool resident probes to bias gathered pending radiance toward blue; warm_near={warm_near_rgb:?}, cool_near={cool_near_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_prefers_hierarchy_parent_probe_radiance() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)];
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [255, 80, 40],
            },
            HybridGiPrepareProbe {
                probe_id: 500,
                slot: 1,
                ray_budget: 96,
                irradiance_rgb: [40, 96, 255],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 34,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let warm_parent = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                probe_with_parent(300, 200, false, 128, Vec3::ZERO, 0.85),
                probe(500, true, 96, Vec3::new(0.2, 0.0, 0.0), 0.85),
            ],
            trace_regions.clone(),
        ),
        prepare.clone(),
    );
    let cool_parent = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                probe_with_parent(300, 500, false, 128, Vec3::ZERO, 0.85),
                probe(500, true, 96, Vec3::new(0.2, 0.0, 0.0), 0.85),
            ],
            trace_regions,
        ),
        prepare,
    );
    let warm_parent_rgb = warm_parent
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm-parent pending probe irradiance");
    let cool_parent_rgb = cool_parent
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool-parent pending probe irradiance");

    assert!(
        warm_parent_rgb[0] > cool_parent_rgb[0] + 12,
        "expected a pending child probe to inherit more red radiance when its hierarchy parent is the warm resident probe; warm_parent={warm_parent_rgb:?}, cool_parent={cool_parent_rgb:?}"
    );
    assert!(
        cool_parent_rgb[2] > warm_parent_rgb[2] + 12,
        "expected a pending child probe to inherit more blue radiance when its hierarchy parent is the cool resident probe; warm_parent={warm_parent_rgb:?}, cool_parent={cool_parent_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_scene_card_capture_requests_move_near_or_far_from_probe(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let near = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                card_id: 11,
                page_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(0.05, 0.0, 0.0),
                bounds_radius: 0.9,
            }],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );
    let far = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                card_id: 11,
                page_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(6.0, 6.0, 6.0),
                bounds_radius: 0.9,
            }],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    let near_luma = average_rgb_luma(
        near.iter()
            .find(|(probe_id, _)| *probe_id == 200)
            .map(|(_, rgb)| *rgb)
            .expect("near scene-card probe irradiance"),
    );
    let far_luma = average_rgb_luma(
        far.iter()
            .find(|(probe_id, _)| *probe_id == 200)
            .map(|(_, rgb)| *rgb)
            .expect("far scene-card probe irradiance"),
    );

    assert!(
        near_luma > far_luma + 6.0,
        "expected near scene card-capture descriptors to bias Hybrid GI irradiance more than far descriptors; near_luma={near_luma:.2}, far_luma={far_luma:.2}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_clipmaps_move_near_or_far_from_probe()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let near = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::new(0.0, 0.0, 0.1),
                half_extent: 4.0,
            }],
            voxel_cells: Vec::new(),
        },
    );
    let far = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::new(8.0, 8.0, 8.0),
                half_extent: 4.0,
            }],
            voxel_cells: Vec::new(),
        },
    );

    let near_luma = average_rgb_luma(
        near.iter()
            .find(|(probe_id, _)| *probe_id == 200)
            .map(|(_, rgb)| *rgb)
            .expect("near scene-voxel probe irradiance"),
    );
    let far_luma = average_rgb_luma(
        far.iter()
            .find(|(probe_id, _)| *probe_id == 200)
            .map(|(_, rgb)| *rgb)
            .expect("far scene-voxel probe irradiance"),
    );

    assert!(
        near_luma > far_luma + 6.0,
        "expected near scene voxel clipmaps to bias Hybrid GI irradiance more than far clipmaps; near_luma={near_luma:.2}, far_luma={far_luma:.2}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_cells_move_near_or_far_from_probe() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 4.0,
    };

    let near = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![clipmap.clone()],
            voxel_cells: vec![
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 21,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 22,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 25,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 26,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
            ],
        },
    );
    let far = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![clipmap],
            voxel_cells: vec![
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 58,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 59,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 62,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 63,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
            ],
        },
    );

    let near_luma = average_rgb_luma(
        near.iter()
            .find(|(probe_id, _)| *probe_id == 200)
            .map(|(_, rgb)| *rgb)
            .expect("near scene-voxel-cell probe irradiance"),
    );
    let far_luma = average_rgb_luma(
        far.iter()
            .find(|(probe_id, _)| *probe_id == 200)
            .map(|(_, rgb)| *rgb)
            .expect("far scene-voxel-cell probe irradiance"),
    );

    assert!(
        near_luma > far_luma + 6.0,
        "expected near scene voxel cells to bias Hybrid GI irradiance more than far voxel cells when clipmap truth stays fixed; near_luma={near_luma:.2}, far_luma={far_luma:.2}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_radiance_changes_with_fixed_layout(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 4.0,
    };
    let voxel_layout = vec![
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 21,
            occupancy_count: 4,
            dominant_card_id: 0,
            radiance_present: true,
            radiance_rgb: [240, 96, 48],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 22,
            occupancy_count: 4,
            dominant_card_id: 0,
            radiance_present: true,
            radiance_rgb: [240, 96, 48],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 25,
            occupancy_count: 4,
            dominant_card_id: 0,
            radiance_present: true,
            radiance_rgb: [240, 96, 48],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 26,
            occupancy_count: 4,
            dominant_card_id: 0,
            radiance_present: true,
            radiance_rgb: [240, 96, 48],
        },
    ];

    let warm = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![clipmap],
            voxel_cells: voxel_layout.clone(),
        },
    );
    let cool = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: voxel_layout
                .into_iter()
                .map(|cell| HybridGiPrepareVoxelCell {
                    radiance_rgb: [48, 112, 240],
                    ..cell
                })
                .collect(),
        },
    );

    assert_ne!(
        warm, cool,
        "expected Hybrid GI GPU irradiance updates to change when runtime scene voxel radiance changes while voxel layout stays fixed"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_uses_runtime_scene_voxel_radiance_rehydrated_from_persisted_page_sample_on_clean_frame(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
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
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let warm_scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh_with_persisted_page_sample(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            [240, 96, 48, 255],
        );
    let cool_scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh_with_persisted_page_sample(
            [1.0, 1.0, 1.0],
            Vec3::ZERO,
            2.0,
            [48, 96, 240, 255],
        );

    assert!(
        warm_scene_prepare.card_capture_requests.is_empty()
            && cool_scene_prepare.card_capture_requests.is_empty(),
        "expected the runtime fixture to represent a clean frame with no pending card recapture requests"
    );
    assert_eq!(
        warm_scene_prepare.voxel_clipmaps,
        cool_scene_prepare.voxel_clipmaps
    );
    assert_eq!(
        warm_scene_prepare
            .voxel_cells
            .iter()
            .map(|cell| (cell.clipmap_id, cell.cell_index, cell.occupancy_count))
            .collect::<Vec<_>>(),
        cool_scene_prepare
            .voxel_cells
            .iter()
            .map(|cell| (cell.clipmap_id, cell.cell_index, cell.occupancy_count))
            .collect::<Vec<_>>(),
        "expected warm/cool persisted-page fixtures to keep identical runtime voxel layout so this regression only checks radiance authority"
    );
    assert_ne!(
        warm_scene_prepare
            .voxel_cells
            .iter()
            .map(|cell| {
                (
                    cell.clipmap_id,
                    cell.cell_index,
                    cell.radiance_present,
                    cell.radiance_rgb,
                )
            })
            .collect::<Vec<_>>(),
        cool_scene_prepare
            .voxel_cells
            .iter()
            .map(|cell| {
                (
                    cell.clipmap_id,
                    cell.cell_index,
                    cell.radiance_present,
                    cell.radiance_rgb,
                )
            })
            .collect::<Vec<_>>(),
        "expected persisted page samples to rehydrate different runtime voxel radiance while layout stays fixed"
    );

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    surface_cache_page_contents: Vec::new(),
                    ..warm_scene_prepare
                })),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    surface_cache_page_contents: Vec::new(),
                    ..cool_scene_prepare
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_scene_prepare_resources = warm_readback
        .scene_prepare_resources
        .expect("expected warm scene-prepare resource snapshot");
    let cool_scene_prepare_resources = cool_readback
        .scene_prepare_resources
        .expect("expected cool scene-prepare resource snapshot");
    assert!(
        warm_scene_prepare_resources.capture_slot_rgba_samples.is_empty()
            && warm_scene_prepare_resources.atlas_slot_rgba_samples.is_empty()
            && cool_scene_prepare_resources.capture_slot_rgba_samples.is_empty()
            && cool_scene_prepare_resources.atlas_slot_rgba_samples.is_empty(),
        "expected this clean-frame regression to remove persisted surface-cache page-content fallback from the renderer input, so the GPU completion difference must come from runtime voxel radiance instead of owner-card capture resources"
    );

    assert_ne!(
        warm_readback.probe_irradiance_rgb,
        cool_readback.probe_irradiance_rgb,
        "expected clean-frame persisted page samples to rehydrate different runtime voxel radiance for GPU completion even after owner-card surface-cache page fallback is removed from the renderer input"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_uses_clean_frame_persisted_surface_cache_page_descriptors_without_dirty_requests_or_runtime_voxels(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let base_page_content = crate::graphics::types::HybridGiPrepareSurfaceCachePageContent {
        page_id: 11,
        owner_card_id: 11,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::ZERO,
        bounds_radius: 2.0,
        atlas_sample_rgba: [224, 112, 64, 255],
        capture_sample_rgba: [240, 96, 48, 255],
    };

    let warm = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![base_page_content],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );
    let cool = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![
                crate::graphics::types::HybridGiPrepareSurfaceCachePageContent {
                    atlas_sample_rgba: [64, 112, 224, 255],
                    capture_sample_rgba: [48, 96, 240, 255],
                    ..base_page_content
                },
            ],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    assert_ne!(
        warm, cool,
        "expected clean-frame persisted surface-cache page samples to stage scene-prepare card descriptors for GPU completion even when there are no dirty card-capture requests or runtime voxel cells"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_ignores_absent_clean_frame_persisted_surface_cache_pages_without_dirty_requests_or_runtime_voxels(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let absent_page_content = crate::graphics::types::HybridGiPrepareSurfaceCachePageContent {
        page_id: 11,
        owner_card_id: 11,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::ZERO,
        bounds_radius: 2.0,
        atlas_sample_rgba: [0, 0, 0, 0],
        capture_sample_rgba: [0, 0, 0, 0],
    };

    let baseline = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );
    let absent = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![absent_page_content],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    assert_eq!(
        baseline, absent,
        "expected absent clean-frame persisted page samples to match the no-page baseline instead of fabricating black GPU completion authority"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_changes_with_fixed_layout(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 4.0,
    };
    let voxel_layout = vec![
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 21,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 22,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 25,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 26,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
    ];

    let owner_a = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![clipmap],
            voxel_cells: voxel_layout.clone(),
        },
    );
    let owner_b = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: voxel_layout
                .into_iter()
                .map(|cell| HybridGiPrepareVoxelCell {
                    dominant_card_id: 22,
                    ..cell
                })
                .collect(),
        },
    );

    assert_ne!(
        owner_a, owner_b,
        "expected Hybrid GI GPU irradiance updates to change when runtime scene voxel dominant owner changes while voxel layout and radiance stay fixed"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_matches_different_card_capture_seed_with_fixed_layout(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 4.0,
    };
    let voxel_layout = vec![
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 21,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 22,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 25,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 26,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
    ];
    let base_card_capture_request = HybridGiPrepareCardCaptureRequest {
        card_id: 11,
        page_id: 22,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::new(20.0, 20.0, 20.0),
        bounds_radius: 0.25,
    };

    let warm = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![base_card_capture_request.clone()],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![clipmap],
            voxel_cells: voxel_layout.clone(),
        },
    );
    let cool = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                capture_slot_id: 17,
                ..base_card_capture_request
            }],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: voxel_layout,
        },
    );

    assert_ne!(
        warm, cool,
        "expected Hybrid GI GPU irradiance updates to change when runtime voxel owner matches a different scene card-capture seed while voxel layout, owner id, and radiance stay fixed"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_preserves_explicit_black_runtime_voxel_radiance_with_fixed_layout(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![probe(200, true, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [0, 0, 0],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 4.0,
    };
    let base_card_capture_request = HybridGiPrepareCardCaptureRequest {
        card_id: 11,
        page_id: 22,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::new(20.0, 20.0, 20.0),
        bounds_radius: 0.25,
    };
    let explicit_black = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![base_card_capture_request.clone()],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![clipmap],
            voxel_cells: runtime_owner_voxel_cells_with_presence(true),
        },
    );
    let absent = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![base_card_capture_request],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: runtime_owner_voxel_cells_with_presence(false),
        },
    );

    let explicit_black_luma = average_rgb_luma(
        explicit_black
            .first()
            .map(|(_, rgb)| *rgb)
            .expect("expected probe irradiance readback"),
    );
    let absent_luma = average_rgb_luma(
        absent
            .first()
            .map(|(_, rgb)| *rgb)
            .expect("expected probe irradiance readback"),
    );

    assert!(
        absent_luma > explicit_black_luma + 8.0,
        "expected explicit-black runtime voxel radiance to stay authoritative through GPU completion instead of collapsing to owner-card fallback; explicit_black_luma={explicit_black_luma:.2}, absent_luma={absent_luma:.2}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_changes_when_scene_card_capture_material_seed_changes_with_fixed_layout(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let probes = vec![probe(200, true, 96, Vec3::ZERO, 0.85)];
    let trace_regions = vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)];
    let lights = vec![directional_light(Vec3::ONE, 2.0)];
    let default_material_extract = build_extract_with_scene_and_lights(
        viewport_size,
        1,
        1,
        probes.clone(),
        trace_regions.clone(),
        vec![mesh_with_material_and_tint(
            11,
            "builtin://material/default",
            Vec4::ONE,
        )],
        lights.clone(),
    );
    let missing_material_extract = build_extract_with_scene_and_lights(
        viewport_size,
        1,
        1,
        probes,
        trace_regions,
        vec![mesh_with_material_and_tint(
            11,
            "builtin://missing-material",
            Vec4::ONE,
        )],
        lights,
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let request = HybridGiPrepareCardCaptureRequest {
        card_id: 11,
        page_id: 22,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::new(0.05, 0.0, 0.0),
        bounds_radius: 0.9,
    };

    let default_material = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        default_material_extract,
        prepare.clone(),
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![request.clone()],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );
    let missing_material = render_hybrid_gi_gpu_readback_with_scene_prepare(
        &mut renderer,
        viewport_size,
        missing_material_extract,
        prepare,
        HybridGiScenePrepareFrame {
            card_capture_requests: vec![request],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    assert_ne!(
        default_material, missing_material,
        "expected Hybrid GI GPU irradiance updates to change when real scene card-capture material seed changes while request layout and scene card identity stay fixed"
    );
}

fn build_extract(
    viewport_size: UVec2,
    probe_budget: u32,
    tracing_budget: u32,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
) -> RenderFrameExtract {
    build_extract_with_lights(
        viewport_size,
        probe_budget,
        tracing_budget,
        probes,
        trace_regions,
        Vec::new(),
    )
}

fn build_extract_with_lights(
    viewport_size: UVec2,
    probe_budget: u32,
    tracing_budget: u32,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
    directional_lights: Vec<RenderDirectionalLightSnapshot>,
) -> RenderFrameExtract {
    build_extract_with_scene_and_lights(
        viewport_size,
        probe_budget,
        tracing_budget,
        probes,
        trace_regions,
        Vec::new(),
        directional_lights,
    )
}

fn build_extract_with_scene_and_lights(
    viewport_size: UVec2,
    probe_budget: u32,
    tracing_budget: u32,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
    meshes: Vec<RenderMeshSnapshot>,
    directional_lights: Vec<RenderDirectionalLightSnapshot>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes = meshes;
    snapshot.scene.directional_lights = directional_lights;
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

fn probe(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    position: Vec3,
    radius: f32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position,
        radius,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn probe_with_parent(
    probe_id: u32,
    parent_probe_id: u32,
    resident: bool,
    ray_budget: u32,
    position: Vec3,
    radius: f32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        parent_probe_id: Some(parent_probe_id),
        ..probe(probe_id, resident, ray_budget, position, radius)
    }
}

fn trace_region(
    region_id: u32,
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_coverage: f32,
) -> RenderHybridGiTraceRegion {
    trace_region_with_rt_lighting(
        region_id,
        bounds_center,
        bounds_radius,
        screen_coverage,
        [0, 0, 0],
    )
}

fn trace_region_with_rt_lighting(
    region_id: u32,
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_coverage: f32,
    rt_lighting_rgb: [u8; 3],
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity: 1,
        region_id,
        bounds_center,
        bounds_radius,
        screen_coverage,
        rt_lighting_rgb,
    }
}

fn directional_light(color: Vec3, intensity: f32) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id: 1,
        direction: Vec3::new(0.0, -1.0, 0.0),
        color,
        intensity,
    }
}

fn mesh_with_material_and_tint(node_id: u64, material_uri: &str, tint: Vec4) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        transform: Transform::identity(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            material_uri,
        )),
        tint,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }
}

fn render_hybrid_gi_gpu_readback(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
) -> Vec<(u32, [u8; 3])> {
    render_hybrid_gi_gpu_full_readback_with_runtime(renderer, viewport_size, extract, prepare, None)
        .0
}

fn render_hybrid_gi_gpu_readback_with_scene_prepare(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    scene_prepare: HybridGiScenePrepareFrame,
) -> Vec<(u32, [u8; 3])> {
    render_hybrid_gi_gpu_full_readback_with_runtime_and_scene_prepare(
        renderer,
        viewport_size,
        extract,
        prepare,
        None,
        Some(scene_prepare),
    )
    .0
}

fn render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    runtime: Option<HybridGiResolveRuntime>,
) -> Vec<(u32, [u8; 3])> {
    render_hybrid_gi_gpu_full_readback_with_runtime(
        renderer,
        viewport_size,
        extract,
        prepare,
        runtime,
    )
    .1
}

fn render_hybrid_gi_gpu_full_readback_with_runtime(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    runtime: Option<HybridGiResolveRuntime>,
) -> (Vec<(u32, [u8; 3])>, Vec<(u32, [u8; 3])>) {
    render_hybrid_gi_gpu_full_readback_with_runtime_and_scene_prepare(
        renderer,
        viewport_size,
        extract,
        prepare,
        runtime,
        None,
    )
}

fn render_hybrid_gi_gpu_full_readback_with_runtime_and_scene_prepare(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    runtime: Option<HybridGiResolveRuntime>,
    scene_prepare: Option<HybridGiScenePrepareFrame>,
) -> (Vec<(u32, [u8; 3])>, Vec<(u32, [u8; 3])>) {
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
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(scene_prepare)
                .with_hybrid_gi_resolve_runtime(runtime),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback");
    (
        readback.probe_irradiance_rgb,
        readback.probe_trace_lighting_rgb,
    )
}

fn expected_gpu_irradiance(
    probe: &RenderHybridGiProbe,
    _slot_or_index: u32,
    previous_irradiance_rgb: [u8; 3],
    trace_regions: &[RenderHybridGiTraceRegion],
    scheduled_trace_region_ids: &[u32],
    tracing_budget: u32,
    pending_completion: bool,
) -> [u8; 3] {
    let contribution = traced_contribution_rgb(
        probe,
        trace_regions,
        scheduled_trace_region_ids,
        tracing_budget,
    );
    if pending_completion {
        return contribution;
    }
    if contribution == [0, 0, 0] {
        return previous_irradiance_rgb;
    }

    let weight = temporal_update_weight(probe.ray_budget, tracing_budget);
    [
        blend_channel(previous_irradiance_rgb[0], contribution[0], weight),
        blend_channel(previous_irradiance_rgb[1], contribution[1], weight),
        blend_channel(previous_irradiance_rgb[2], contribution[2], weight),
    ]
}

fn temporal_update_weight(ray_budget: u32, tracing_budget: u32) -> u8 {
    let weight = 48_u32
        .saturating_add(ray_budget.min(192) / 2)
        .saturating_add(tracing_budget.min(4) * 12)
        .min(224);
    weight as u8
}

fn blend_channel(previous: u8, contribution: u8, weight: u8) -> u8 {
    let weight = u32::from(weight);
    let inverse = 255_u32.saturating_sub(weight);
    (((u32::from(previous) * inverse) + (u32::from(contribution) * weight) + 127) / 255) as u8
}

fn traced_contribution_rgb(
    probe: &RenderHybridGiProbe,
    trace_regions: &[RenderHybridGiTraceRegion],
    scheduled_trace_region_ids: &[u32],
    tracing_budget: u32,
) -> [u8; 3] {
    let position_x_q = quantized_signed(probe.position.x);
    let position_y_q = quantized_signed(probe.position.y);
    let position_z_q = quantized_signed(probe.position.z);
    let radius_q = quantized_positive(probe.radius, 96.0);
    let mut weighted_rgb = [0_u32; 3];
    let mut total_weight = 0_u32;
    for region in active_trace_regions(trace_regions, scheduled_trace_region_ids, tracing_budget) {
        let center_x_q = quantized_signed(region.bounds_center.x);
        let center_y_q = quantized_signed(region.bounds_center.y);
        let center_z_q = quantized_signed(region.bounds_center.z);
        let region_radius_q = quantized_positive(region.bounds_radius, 96.0);
        let coverage_q = quantized_positive(region.screen_coverage, 128.0);
        let reach = radius_q
            .saturating_add(region_radius_q)
            .saturating_add(1)
            .max(1);
        let max_distance = reach.saturating_mul(3).max(1);
        let distance_to_region = abs_diff_u32(position_x_q, center_x_q)
            .saturating_add(abs_diff_u32(position_y_q, center_y_q))
            .saturating_add(abs_diff_u32(position_z_q, center_z_q));
        if distance_to_region >= max_distance {
            continue;
        }

        let contribution_weight = trace_region_contribution_weight(
            distance_to_region,
            max_distance,
            probe.ray_budget,
            coverage_q,
            tracing_budget,
        );
        let base_rgb = trace_region_base_rgb(region_id_and_quantized(
            region.region_id,
            center_x_q,
            center_y_q,
            center_z_q,
            region_radius_q,
            coverage_q,
        ));
        weighted_rgb = [
            weighted_rgb[0].saturating_add(u32::from(base_rgb[0]) * u32::from(contribution_weight)),
            weighted_rgb[1].saturating_add(u32::from(base_rgb[1]) * u32::from(contribution_weight)),
            weighted_rgb[2].saturating_add(u32::from(base_rgb[2]) * u32::from(contribution_weight)),
        ];
        total_weight = total_weight.saturating_add(u32::from(contribution_weight));
    }

    if total_weight == 0 {
        return [0, 0, 0];
    }

    [
        normalize_weighted_channel(weighted_rgb[0], total_weight),
        normalize_weighted_channel(weighted_rgb[1], total_weight),
        normalize_weighted_channel(weighted_rgb[2], total_weight),
    ]
}

fn active_trace_regions<'a>(
    trace_regions: &'a [RenderHybridGiTraceRegion],
    scheduled_trace_region_ids: &[u32],
    tracing_budget: u32,
) -> Vec<&'a RenderHybridGiTraceRegion> {
    scheduled_trace_region_ids
        .iter()
        .filter_map(|region_id| {
            trace_regions
                .iter()
                .find(|region| region.region_id == *region_id)
        })
        .take(tracing_budget as usize)
        .collect()
}

fn abs_diff_u32(a: u32, b: u32) -> u32 {
    if a >= b {
        a - b
    } else {
        b - a
    }
}

fn trace_region_contribution_weight(
    distance_to_region: u32,
    max_distance: u32,
    ray_budget: u32,
    coverage_q: u32,
    tracing_budget: u32,
) -> u8 {
    let proximity = max_distance.saturating_sub(distance_to_region);
    let proximity_weight = ((proximity * 255) / max_distance.max(1)).min(255);
    let trace_strength = (32_u32
        .saturating_add(ray_budget.min(160) / 2)
        .saturating_add(coverage_q.min(160) / 2)
        .saturating_add(tracing_budget.min(4) * 40))
    .min(255);
    (((proximity_weight * trace_strength) + 127) / 255) as u8
}

fn normalize_weighted_channel(weighted_channel: u32, total_weight: u32) -> u8 {
    (((weighted_channel + (total_weight / 2)) / total_weight).min(255)) as u8
}

#[derive(Clone, Copy)]
struct QuantizedTraceRegion {
    region_id: u32,
    center_x_q: u32,
    center_y_q: u32,
    center_z_q: u32,
    radius_q: u32,
    coverage_q: u32,
}

fn region_id_and_quantized(
    region_id: u32,
    center_x_q: u32,
    center_y_q: u32,
    center_z_q: u32,
    radius_q: u32,
    coverage_q: u32,
) -> QuantizedTraceRegion {
    QuantizedTraceRegion {
        region_id,
        center_x_q,
        center_y_q,
        center_z_q,
        radius_q,
        coverage_q,
    }
}

fn trace_region_base_rgb(region: QuantizedTraceRegion) -> [u8; 3] {
    [
        (32 + ((region.region_id * 17 + region.center_x_q + region.coverage_q) % 160)) as u8,
        (32 + ((region.region_id * 11 + region.center_y_q + region.radius_q) % 160)) as u8,
        (32 + ((region.region_id * 7 + region.center_z_q + region.coverage_q * 3) % 160)) as u8,
    ]
}

fn quantized_signed(value: f32) -> u32 {
    ((value * 64.0).round() as i32).wrapping_add(2048) as u32
}

fn quantized_positive(value: f32, scale: f32) -> u32 {
    (value.max(0.0) * scale).round() as u32
}

fn average_rgb_luma(rgb: [u8; 3]) -> f32 {
    (f32::from(rgb[0]) + f32::from(rgb[1]) + f32::from(rgb[2])) / 3.0
}

fn runtime_owner_voxel_cells_with_presence(
    radiance_present: bool,
) -> Vec<HybridGiPrepareVoxelCell> {
    [21_u32, 22, 25, 26]
        .into_iter()
        .map(|cell_index| HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present,
            radiance_rgb: [0, 0, 0],
        })
        .collect()
}

fn runtime_voxel_scene_prepare_from_tinted_mesh_with_persisted_page_sample(
    tint_rgb: [f32; 3],
    translation: Vec3,
    uniform_scale: f32,
    persisted_capture_rgba: [u8; 4],
) -> HybridGiScenePrepareFrame {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 0,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };

    runtime.register_scene_extract(
        Some(&extract),
        &[RenderMeshSnapshot {
            node_id: 11,
            transform: Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/card.obj",
            )),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/runtime-voxel-persisted-page.mat",
            )),
            tint: Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        &[],
        &[],
        &[],
    );
    runtime.apply_scene_prepare_resources(
        &crate::graphics::scene::HybridGiScenePrepareResourcesSnapshot {
            card_capture_request_count: 1,
            voxel_clipmap_ids: Vec::new(),
            occupied_atlas_slots: vec![0],
            occupied_capture_slots: vec![0],
            atlas_slot_rgba_samples: vec![(0, persisted_capture_rgba)],
            capture_slot_rgba_samples: vec![(0, persisted_capture_rgba)],
            voxel_clipmap_rgba_samples: Vec::new(),
            voxel_clipmap_occupancy_masks: Vec::new(),
            voxel_clipmap_cell_rgba_samples: Vec::new(),
            voxel_clipmap_cell_occupancy_counts: Vec::new(),
            voxel_clipmap_cell_dominant_node_ids: Vec::new(),
            voxel_clipmap_cell_dominant_rgba_samples: Vec::new(),
            atlas_slot_count: 0,
            capture_slot_count: 0,
            atlas_texture_extent: (0, 0),
            capture_texture_extent: (0, 0),
            capture_layer_count: 0,
        },
    );
    runtime.register_scene_extract(
        Some(&extract),
        &[RenderMeshSnapshot {
            node_id: 11,
            transform: Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/card.obj",
            )),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/runtime-voxel-persisted-page.mat",
            )),
            tint: Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        &[],
        &[],
        &[],
    );

    runtime.build_scene_prepare_frame()
}
