use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::math::{UVec2, Vec3};
use crate::scene::world::World;
use crate::test_support::render_feature_fixtures::hybrid_gi_render_feature_descriptor;

use crate::{
    runtime::HybridGiRuntimeState,
    types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareVoxelCell,
        HybridGiPrepareVoxelClipmap, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
        ViewportRenderFrame,
    },
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

fn hybrid_gi_scene_renderer(asset_manager: Arc<ProjectAssetManager>) -> SceneRenderer {
    SceneRenderer::new_with_plugin_render_features(
        asset_manager,
        [hybrid_gi_render_feature_descriptor()],
    )
    .unwrap()
}

#[test]
fn hybrid_gi_resolve_adds_radiance_cache_indirect_light_when_feature_enabled() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let baseline = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size),
            &compiled,
            None,
        )
        .unwrap();
    let resolved = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
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
                }),
            ),
            &compiled,
            None,
        )
        .unwrap();

    let baseline_luma = average_region_luma(&baseline.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let resolved_luma = average_region_luma(&resolved.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    assert!(
        resolved_luma > baseline_luma + 0.5,
        "expected hybrid GI resolve to brighten the probe-influenced screen region when resident probes are available; baseline_luma={baseline_luma:.2}, resolved_luma={resolved_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_prepare_probe_irradiance_colors() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
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
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
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
                }),
            ),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    assert!(
        warm_red > cool_red + 1.0,
        "expected warm GI irradiance to increase red output in the probe-influenced region; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 1.0,
        "expected cool GI irradiance to increase blue output in the probe-influenced region; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_localizes_indirect_light_by_probe_screen_position() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes(
        viewport_size,
        vec![probe(200, true, 128, Vec3::new(-0.9, 0.0, 0.0), 2.0)],
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let baseline = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size),
            &compiled,
            None,
        )
        .unwrap();
    let resolved = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 72, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            None,
        )
        .unwrap();

    let left_red_delta =
        average_half_channel_delta(&baseline.rgba, &resolved.rgba, viewport_size, 0, Half::Left);
    let right_red_delta = average_half_channel_delta(
        &baseline.rgba,
        &resolved.rgba,
        viewport_size,
        0,
        Half::Right,
    );

    assert!(
        left_red_delta > right_red_delta + 0.6,
        "expected left-side warm probe to add more red indirect light on the left half; left_delta={left_red_delta:.2}, right_delta={right_red_delta:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_localizes_trace_region_boost_by_screen_position() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 3.5)],
        vec![trace_region_with_bounds(
            40,
            Vec3::new(-1.25, 0.0, 0.0),
            1.75,
            0.9,
        )],
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let baseline = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [176, 188, 208],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let boosted = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [176, 188, 208],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            None,
        )
        .unwrap();

    let left_luma_delta =
        average_half_luma_delta(&baseline.rgba, &boosted.rgba, viewport_size, Half::Left);
    let right_luma_delta =
        average_half_luma_delta(&baseline.rgba, &boosted.rgba, viewport_size, Half::Right);

    assert!(
        left_luma_delta > right_luma_delta + 0.3,
        "expected a left-side trace region to concentrate Hybrid GI boost on the left half; left_delta={left_luma_delta:.2}, right_delta={right_luma_delta:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_prefers_screen_probe_irradiance_supported_by_scheduled_trace_regions() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(200, true, 128, Vec3::new(-0.6, 0.0, 0.0), 2.1),
            probe(500, true, 128, Vec3::new(0.6, 0.0, 0.0), 2.1),
        ],
        vec![
            trace_region_with_bounds(40, Vec3::new(-0.6, 0.0, 0.0), 1.25, 0.95),
            trace_region_with_bounds(50, Vec3::new(0.6, 0.0, 0.0), 1.25, 0.95),
        ],
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let left_supported = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [255, 80, 40],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 128,
                            irradiance_rgb: [40, 96, 255],
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
    let right_supported = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [255, 80, 40],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 128,
                            irradiance_rgb: [40, 96, 255],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![50],
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            None,
        )
        .unwrap();

    let center_left_red = average_region_channel(
        &left_supported.rgba,
        viewport_size,
        0,
        0.35,
        0.65,
        0.25,
        0.75,
    );
    let center_left_blue = average_region_channel(
        &left_supported.rgba,
        viewport_size,
        2,
        0.35,
        0.65,
        0.25,
        0.75,
    );
    let center_right_red = average_region_channel(
        &right_supported.rgba,
        viewport_size,
        0,
        0.35,
        0.65,
        0.25,
        0.75,
    );
    let center_right_blue = average_region_channel(
        &right_supported.rgba,
        viewport_size,
        2,
        0.35,
        0.65,
        0.25,
        0.75,
    );

    assert!(
        center_left_red > center_right_red + 0.6,
        "expected left-side scheduled trace support to bias center resolve toward the warm left probe; left_red={center_left_red:.2}, right_red={center_right_red:.2}"
    );
    assert!(
        center_right_blue > center_left_blue + 0.6,
        "expected right-side scheduled trace support to bias center resolve toward the cool right probe; left_blue={center_left_blue:.2}, right_blue={center_right_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_changes_when_parent_child_hierarchy_links_overlapping_probes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![
                    probe(200, true, 128, Vec3::ZERO, 2.2),
                    probe(300, true, 128, Vec3::ZERO, 2.2),
                ],
                trace_regions.clone(),
            ),
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
        .unwrap();

    let flat = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 128, Vec3::ZERO, 2.2),
                        probe(300, true, 128, Vec3::ZERO, 2.2),
                    ],
                    trace_regions.clone(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 80, 40],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [40, 96, 255],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();
    let hierarchical = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 128, Vec3::ZERO, 2.2),
                        probe_with_parent(300, 200, true, 128, Vec3::ZERO, 2.2),
                    ],
                    trace_regions,
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 80, 40],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [40, 96, 255],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();

    let flat_red = average_region_channel(&flat.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let flat_blue = average_region_channel(&flat.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let hierarchical_red =
        average_region_channel(&hierarchical.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let hierarchical_blue =
        average_region_channel(&hierarchical.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);

    assert!(
        hierarchical_blue > flat_blue + 0.6,
        "expected overlapping child probe to gain more blue resolve weight when linked to a resident parent; flat_blue={flat_blue:.2}, hierarchical_blue={hierarchical_blue:.2}"
    );
    assert!(
        flat_red > hierarchical_red + 0.6,
        "expected overlapping parent probe to lose relative red dominance once the child probe becomes hierarchy-linked; flat_red={flat_red:.2}, hierarchical_red={hierarchical_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_changes_when_trace_region_rt_lighting_tint_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let probe = probe(200, true, 128, Vec3::ZERO, 2.6);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![probe],
                vec![trace_region_with_bounds_and_rt_lighting(
                    40,
                    Vec3::ZERO,
                    1.35,
                    0.95,
                    [255, 64, 32],
                )],
            ),
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
        .unwrap();

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe],
                    vec![trace_region_with_bounds_and_rt_lighting(
                        40,
                        Vec3::ZERO,
                        1.35,
                        0.95,
                        [255, 64, 32],
                    )],
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [160, 160, 160],
                }],
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
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe],
                    vec![trace_region_with_bounds_and_rt_lighting(
                        40,
                        Vec3::ZERO,
                        1.35,
                        0.95,
                        [32, 96, 255],
                    )],
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [160, 160, 160],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![40],
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.6,
        "expected warm RT-lighting trace region tint to increase red indirect light during resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.6,
        "expected cool RT-lighting trace region tint to increase blue indirect light during resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_frame_ignores_trace_region_boost() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 3.5)],
        vec![trace_region_with_bounds(
            40,
            Vec3::new(-1.25, 0.0, 0.0),
            1.75,
            0.9,
        )],
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.78, 0.48, 0.26], Vec3::ZERO, 1.0);

    let without_trace = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let with_trace = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let left_luma_delta = average_half_luma_delta(
        &without_trace.rgba,
        &with_trace.rgba,
        viewport_size,
        Half::Left,
    );
    let right_luma_delta = average_half_luma_delta(
        &without_trace.rgba,
        &with_trace.rgba,
        viewport_size,
        Half::Right,
    );

    assert!(
        left_luma_delta < 0.15,
        "expected scene-driven GI frames to stop using authored trace-region boost as a direct final-composite weight source; left_delta={left_luma_delta:.2}"
    );
    assert!(
        right_luma_delta < 0.15,
        "expected scene-driven GI frames to stay stable outside legacy trace-region boost even when scheduled trace ids remain for fixture/runtime scaffolding; right_delta={right_luma_delta:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_frame_ignores_trace_region_rt_lighting_tint_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let probe = probe(200, true, 128, Vec3::ZERO, 2.6);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![probe],
                vec![trace_region_with_bounds_and_rt_lighting(
                    40,
                    Vec3::ZERO,
                    1.35,
                    0.95,
                    [255, 64, 32],
                )],
            ),
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
        .unwrap();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.78, 0.48, 0.26], Vec3::ZERO, 1.0);

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe],
                    vec![trace_region_with_bounds_and_rt_lighting(
                        40,
                        Vec3::ZERO,
                        1.35,
                        0.95,
                        [255, 64, 32],
                    )],
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [160, 160, 160],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![40],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe],
                    vec![trace_region_with_bounds_and_rt_lighting(
                        40,
                        Vec3::ZERO,
                        1.35,
                        0.95,
                        [32, 96, 255],
                    )],
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [160, 160, 160],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: vec![40],
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let warm_luma = average_region_luma(&warm.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);

    assert!(
        warm_luma > 1.0,
        "expected the scene-driven GI fixture itself to contribute visible indirect light so the trace-region-tint check is meaningful; warm_luma={warm_luma:.2}"
    );
    assert!(
        (warm_red - cool_red).abs() < 0.2,
        "expected authored trace-region RT tint to stop recoloring final scene-driven GI resolve once voxel/surface scene truth is present; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        (warm_blue - cool_blue).abs() < 0.2,
        "expected scene-driven GI resolve to stay stable across legacy trace-region tint changes once the new scene representation path is active; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_stripped_scene_prepare_runtime_truth_ignores_trace_region_rt_lighting_tint_changes(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let probe = probe(200, true, 128, Vec3::ZERO, 2.6);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![probe],
                vec![trace_region_with_bounds_and_rt_lighting(
                    40,
                    Vec3::ZERO,
                    1.35,
                    0.95,
                    [255, 64, 32],
                )],
            ),
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
        .unwrap();
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [160, 160, 160],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.62, 0.62, 0.62], 0.58),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.52, 0.52, 0.52], 0.42),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([200]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([200]))
        .build();

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe],
                    vec![trace_region_with_bounds_and_rt_lighting(
                        40,
                        Vec3::ZERO,
                        1.35,
                        0.95,
                        [255, 64, 32],
                    )],
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(prepare.clone()))
            .with_hybrid_gi_resolve_runtime(Some(runtime.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe],
                    vec![trace_region_with_bounds_and_rt_lighting(
                        40,
                        Vec3::ZERO,
                        1.35,
                        0.95,
                        [32, 96, 255],
                    )],
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(prepare))
            .with_hybrid_gi_resolve_runtime(Some(runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let warm_luma = average_region_luma(&warm.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);

    assert!(
        warm_luma > 1.0,
        "expected stripped-scene-prepare runtime scene truth to contribute visible indirect light so the trace-region-tint check is meaningful; warm_luma={warm_luma:.2}"
    );
    assert!(
        (warm_red - cool_red).abs() < 0.2,
        "expected stripped-scene-prepare runtime scene truth to ignore authored trace-region RT tint changes once runtime scene truth already owns the current probe; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        (warm_blue - cool_blue).abs() < 0.2,
        "expected stripped-scene-prepare runtime scene truth to keep current GI stable across legacy trace-region tint changes; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_frame_ignores_prepare_probe_irradiance_tint_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 2.6)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.78, 0.48, 0.26], Vec3::ZERO, 1.0);

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 96, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [48, 96, 255],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let warm_luma = average_region_luma(&warm.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);

    assert!(
        warm_luma > 1.0,
        "expected the scene-driven GI fixture itself to contribute visible indirect light so the authored-probe-tint check is meaningful; warm_luma={warm_luma:.2}"
    );
    assert!(
        (warm_red - cool_red).abs() < 0.2,
        "expected scene-driven GI resolve to stop recoloring final output from authored resident-probe irradiance once voxel/surface truth is present; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        (warm_blue - cool_blue).abs() < 0.2,
        "expected scene-driven GI resolve to stay stable across authored resident-probe irradiance tint changes once the new scene representation path is active; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_frame_ignores_prepare_probe_screen_position_changes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.78, 0.48, 0.26], Vec3::ZERO, 1.0);
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![probe(200, true, 128, Vec3::new(-0.9, 0.0, 0.0), 2.2)],
                Vec::new(),
            ),
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
        .unwrap();

    let left = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe(200, true, 128, Vec3::new(-0.9, 0.0, 0.0), 2.2)],
                    Vec::new(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [160, 160, 160],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let right = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![probe(200, true, 128, Vec3::new(0.9, 0.0, 0.0), 2.2)],
                    Vec::new(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 128,
                    irradiance_rgb: [160, 160, 160],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let left_luma = average_region_luma(&left.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);
    let left_half_delta =
        average_half_luma_delta(&left.rgba, &right.rgba, viewport_size, Half::Left);
    let right_half_delta =
        average_half_luma_delta(&left.rgba, &right.rgba, viewport_size, Half::Right);

    assert!(
        left_luma > 1.0,
        "expected the scene-driven GI fixture itself to contribute visible indirect light so the authored-probe-position check is meaningful; left_luma={left_luma:.2}"
    );
    assert!(
        left_half_delta < 0.15,
        "expected scene-driven GI resolve to stop shifting final left-half energy when only authored resident-probe screen position moves; left_delta={left_half_delta:.2}"
    );
    assert!(
        right_half_delta < 0.15,
        "expected scene-driven GI resolve to stay spatially stable when only authored resident-probe screen position changes once the new scene representation path is active; right_delta={right_half_delta:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_frame_localizes_from_scene_prepare_bounds_instead_of_probe_position(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 2.2)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let left_scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::new(-1.8, 0.0, 0.0),
            bounds_radius: 0.75,
            atlas_sample_rgba: [224, 112, 64, 255],
            capture_sample_rgba: [240, 96, 48, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };
    let right_scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::new(1.8, 0.0, 0.0),
            bounds_radius: 0.75,
            atlas_sample_rgba: [224, 112, 64, 255],
            capture_sample_rgba: [240, 96, 48, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };

    let left = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(left_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let right = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(right_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let left_frame_left_luma = average_half_luma(&left.rgba, viewport_size, Half::Left);
    let left_frame_right_luma = average_half_luma(&left.rgba, viewport_size, Half::Right);
    let right_frame_left_luma = average_half_luma(&right.rgba, viewport_size, Half::Left);
    let right_frame_right_luma = average_half_luma(&right.rgba, viewport_size, Half::Right);

    assert!(
        left_frame_left_luma > left_frame_right_luma + 0.2,
        "expected left-shifted scene_prepare bounds to bias scene-driven final GI toward the left half instead of staying locked to authored probe screen position; left_frame_left={left_frame_left_luma:.2}, left_frame_right={left_frame_right_luma:.2}"
    );
    assert!(
        right_frame_right_luma > right_frame_left_luma + 0.2,
        "expected right-shifted scene_prepare bounds to bias scene-driven final GI toward the right half instead of staying locked to authored probe screen position; right_frame_left={right_frame_left_luma:.2}, right_frame_right={right_frame_right_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_frame_ignores_unmatched_prepare_probe_slots() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 2.2)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.78, 0.48, 0.26], Vec3::ZERO, 1.0);

    let single = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let extra_unmatched = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [160, 160, 160],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 999,
                            slot: 1,
                            ray_budget: 24,
                            irradiance_rgb: [255, 32, 32],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let single_center_luma =
        average_region_luma(&single.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);
    let extra_center_luma =
        average_region_luma(&extra_unmatched.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);
    let left_half_delta = average_half_luma_delta(
        &single.rgba,
        &extra_unmatched.rgba,
        viewport_size,
        Half::Left,
    );
    let right_half_delta = average_half_luma_delta(
        &single.rgba,
        &extra_unmatched.rgba,
        viewport_size,
        Half::Right,
    );

    assert!(
        single_center_luma > 1.0,
        "expected the scene-driven GI fixture itself to contribute visible indirect light so the unmatched-slot check is meaningful; single_center_luma={single_center_luma:.2}"
    );
    assert!(
        (single_center_luma - extra_center_luma).abs() < 0.2,
        "expected scene-driven GI resolve to ignore extra resident probe slots that have no authored probe source instead of dimming the final composite through probe-count/container semantics; single_center_luma={single_center_luma:.2}, extra_center_luma={extra_center_luma:.2}"
    );
    assert!(
        left_half_delta < 0.15,
        "expected scene-driven GI resolve to stay spatially stable on the left half when only an unmatched resident probe slot is added for compatibility fixtures; left_delta={left_half_delta:.2}"
    );
    assert!(
        right_half_delta < 0.15,
        "expected scene-driven GI resolve to stay spatially stable on the right half when only an unmatched resident probe slot is added for compatibility fixtures; right_delta={right_half_delta:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_frame_ignores_authored_parent_child_links() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![
                    probe(200, true, 128, Vec3::ZERO, 2.2),
                    probe(300, true, 128, Vec3::ZERO, 2.2),
                ],
                trace_regions.clone(),
            ),
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
        .unwrap();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.78, 0.48, 0.26], Vec3::ZERO, 1.0);

    let flat = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 128, Vec3::ZERO, 2.2),
                        probe(300, true, 128, Vec3::ZERO, 2.2),
                    ],
                    trace_regions.clone(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let hierarchical = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 128, Vec3::ZERO, 2.2),
                        probe_with_parent(300, 200, true, 128, Vec3::ZERO, 2.2),
                    ],
                    trace_regions,
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let flat_center_luma = average_region_luma(&flat.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);
    let hierarchical_center_luma =
        average_region_luma(&hierarchical.rgba, viewport_size, 0.35, 0.65, 0.25, 0.75);
    let left_half_delta =
        average_half_luma_delta(&flat.rgba, &hierarchical.rgba, viewport_size, Half::Left);
    let right_half_delta =
        average_half_luma_delta(&flat.rgba, &hierarchical.rgba, viewport_size, Half::Right);

    assert!(
        flat_center_luma > 1.0,
        "expected the scene-driven GI fixture itself to contribute visible indirect light so the hierarchy-link check is meaningful; flat_center_luma={flat_center_luma:.2}"
    );
    assert!(
        (flat_center_luma - hierarchical_center_luma).abs() < 0.2,
        "expected scene-driven GI resolve to ignore authored parent-child links when current scene truth is unchanged, instead of changing final intensity through resolve-weight fallback; flat_center_luma={flat_center_luma:.2}, hierarchical_center_luma={hierarchical_center_luma:.2}"
    );
    assert!(
        left_half_delta < 0.15,
        "expected scene-driven GI resolve to stay spatially stable on the left half when only authored parent-child links change; left_delta={left_half_delta:.2}"
    );
    assert!(
        right_half_delta < 0.15,
        "expected scene-driven GI resolve to stay spatially stable on the right half when only authored parent-child links change; right_delta={right_half_delta:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_inherited_runtime_truth_keeps_current_gi_when_only_ancestor_depth_changes(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.68),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.62),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([100]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([100]))
        .build();

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_center_luma =
        average_region_luma(&stable.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let changed_center_luma =
        average_region_luma(&changed.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_center_luma > 1.0,
        "expected the inherited scene-driven runtime GI fixture itself to contribute visible indirect light so the ancestor-depth check is meaningful; stable_center_luma={stable_center_luma:.2}"
    );
    assert!(
        (stable_center_luma - changed_center_luma).abs() < 0.2,
        "expected scene-driven inherited runtime truth to keep current GI materially aligned even when only an intermediate authored ancestor node is inserted, instead of letting ancestor depth attenuate the same runtime scene truth; stable_center_luma={stable_center_luma:.2}, changed_center_luma={changed_center_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_inherited_runtime_truth_keeps_scene_prepare_mix_when_only_ancestor_depth_changes(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.68),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.62),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([100]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([100]))
        .build();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.14, 0.38, 0.92], Vec3::ZERO, 1.0);

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_red = average_region_channel(&stable.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let changed_red =
        average_region_channel(&changed.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let stable_blue =
        average_region_channel(&stable.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let changed_blue =
        average_region_channel(&changed.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_red > 1.0 || stable_blue > 1.0,
        "expected the inherited-runtime plus scene-prepare GI fixture itself to contribute visible indirect light so the ancestor-depth mix check is meaningful; stable_red={stable_red:.2}, stable_blue={stable_blue:.2}"
    );
    assert!(
        (stable_red - changed_red).abs() < 0.05,
        "expected scene-driven inherited runtime truth to keep the same red mix against scene-prepare fallback even when only an intermediate authored ancestor node is inserted, instead of letting ancestor depth change the runtime-vs-scene-prepare blend; stable_red={stable_red:.2}, changed_red={changed_red:.2}"
    );
    assert!(
        (stable_blue - changed_blue).abs() < 0.05,
        "expected scene-driven inherited runtime truth to keep the same blue mix against scene-prepare fallback even when only an intermediate authored ancestor node is inserted, instead of letting ancestor depth change the runtime-vs-scene-prepare blend; stable_blue={stable_blue:.2}, changed_blue={changed_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_inherited_runtime_truth_ignores_scene_prepare_surface_cache_tint()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.68),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.62),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([100]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([100]))
        .build();
    let warm_scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            atlas_sample_rgba: [224, 112, 64, 255],
            capture_sample_rgba: [240, 96, 48, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };
    let cool_scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            atlas_sample_rgba: [64, 112, 224, 255],
            capture_sample_rgba: [48, 96, 240, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > 1.0 || warm_blue > 1.0,
        "expected the inherited scene-driven runtime fixture itself to contribute visible indirect light so the surface-cache tint check is meaningful; warm_red={warm_red:.2}, warm_blue={warm_blue:.2}"
    );
    assert!(
        (warm_red - cool_red).abs() < 0.05,
        "expected scene-driven inherited runtime truth to keep current red GI stable when only the current surface-cache page tint changes, instead of reblending page tint into inherited runtime truth; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        (warm_blue - cool_blue).abs() < 0.05,
        "expected scene-driven inherited runtime truth to keep current blue GI stable when only the current surface-cache page tint changes, instead of reblending page tint into inherited runtime truth; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_ignores_scene_prepare_surface_cache_tint(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.46),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.44),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([200]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([200]))
        .build();
    let warm_scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            atlas_sample_rgba: [224, 112, 64, 255],
            capture_sample_rgba: [240, 96, 48, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };
    let cool_scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            atlas_sample_rgba: [64, 112, 224, 255],
            capture_sample_rgba: [48, 96, 240, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > 1.0 || warm_blue > 1.0,
        "expected the descendant scene-driven runtime fixture itself to contribute visible indirect light so the surface-cache tint check is meaningful; warm_red={warm_red:.2}, warm_blue={warm_blue:.2}"
    );
    assert!(
        (warm_red - cool_red).abs() < 0.05,
        "expected scene-driven descendant runtime truth to keep current red GI stable when only the current surface-cache page tint changes, instead of reblending page tint into descendant runtime truth; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        (warm_blue - cool_blue).abs() < 0.05,
        "expected scene-driven descendant runtime truth to keep current blue GI stable when only the current surface-cache page tint changes, instead of reblending page tint into descendant runtime truth; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_inherited_runtime_truth_ignores_reachable_continuation_weight_from_inserted_ancestor(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
            150,
            HybridGiResolveRuntime::pack_resolve_weight_q8(2.4),
        )]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.68),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            100,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.62),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([100]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([100]))
        .build();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.18, 0.34, 0.82], Vec3::ZERO, 1.0);

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_center_luma =
        average_region_luma(&stable.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let changed_center_luma =
        average_region_luma(&changed.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_center_luma > 1.0,
        "expected the inherited scene-driven runtime GI fixture itself to contribute visible indirect light so the inserted-ancestor continuation-weight check is meaningful; stable_center_luma={stable_center_luma:.2}"
    );
    assert!(
        (stable_center_luma - changed_center_luma).abs() < 0.2,
        "expected scene-driven inherited runtime truth to ignore continuation-only hierarchy resolve weight that only becomes reachable after inserting an authored ancestor node, instead of changing current GI intensity when the underlying scene truth stayed fixed; stable_center_luma={stable_center_luma:.2}, changed_center_luma={changed_center_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_inherited_runtime_truth_ignores_reachable_continuation_rgb_from_inserted_ancestor(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([
            (
                100,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.68),
            ),
            (
                150,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.16, 0.34, 0.92], 0.72),
            ),
        ]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([
            (
                100,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.62),
            ),
            (
                150,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.18, 0.36, 0.9], 0.68),
            ),
        ]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([100]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([100]))
        .build();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.18, 0.34, 0.82], Vec3::ZERO, 1.0);

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_red = average_region_channel(&stable.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let changed_red =
        average_region_channel(&changed.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let stable_blue =
        average_region_channel(&stable.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let changed_blue =
        average_region_channel(&changed.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_red > 1.0 || stable_blue > 1.0,
        "expected the inherited scene-driven runtime GI fixture itself to contribute visible indirect light so the inserted-ancestor continuation-RGB check is meaningful; stable_red={stable_red:.2}, stable_blue={stable_blue:.2}"
    );
    assert!(
        (stable_red - changed_red).abs() < 0.05,
        "expected scene-driven inherited runtime truth to ignore continuation-only hierarchy irradiance/RT RGB that only becomes reachable after inserting an authored ancestor node, instead of shifting current red GI while the underlying scene truth stayed fixed; stable_red={stable_red:.2}, changed_red={changed_red:.2}"
    );
    assert!(
        (stable_blue - changed_blue).abs() < 0.05,
        "expected scene-driven inherited runtime truth to ignore continuation-only hierarchy irradiance/RT RGB that only becomes reachable after inserting an authored ancestor node, instead of shifting current blue GI while the underlying scene truth stayed fixed; stable_blue={stable_blue:.2}, changed_blue={changed_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_ignores_reachable_continuation_weight_from_inserted_descendant(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.2),
            probe_with_parent(200, 150, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([(
            150,
            HybridGiResolveRuntime::pack_resolve_weight_q8(2.4),
        )]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.46),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.44),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([200]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([200]))
        .build();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.18, 0.34, 0.82], Vec3::ZERO, 1.0);

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_center_luma =
        average_region_luma(&stable.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let changed_center_luma =
        average_region_luma(&changed.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_center_luma > 1.0,
        "expected the descendant scene-driven runtime GI fixture itself to contribute visible indirect light so the inserted-descendant continuation-weight check is meaningful; stable_center_luma={stable_center_luma:.2}"
    );
    assert!(
        (stable_center_luma - changed_center_luma).abs() < 0.2,
        "expected scene-driven descendant runtime truth to ignore continuation-only hierarchy resolve weight that only becomes reachable after inserting an authored descendant node, instead of changing current GI intensity when the underlying leaf scene truth stayed fixed; stable_center_luma={stable_center_luma:.2}, changed_center_luma={changed_center_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_ignores_reachable_continuation_rgb_from_inserted_descendant(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.2),
            probe_with_parent(200, 150, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([
            (
                150,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.16, 0.34, 0.92], 0.72),
            ),
            (
                200,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.46),
            ),
        ]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([
            (
                150,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.18, 0.36, 0.9], 0.68),
            ),
            (
                200,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.44),
            ),
        ]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([200]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([200]))
        .build();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.18, 0.34, 0.82], Vec3::ZERO, 1.0);

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_red = average_region_channel(&stable.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let changed_red =
        average_region_channel(&changed.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let stable_blue =
        average_region_channel(&stable.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let changed_blue =
        average_region_channel(&changed.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_red > 1.0 || stable_blue > 1.0,
        "expected the descendant scene-driven runtime GI fixture itself to contribute visible indirect light so the inserted-descendant continuation-RGB check is meaningful; stable_red={stable_red:.2}, stable_blue={stable_blue:.2}"
    );
    assert!(
        (stable_red - changed_red).abs() < 0.05,
        "expected scene-driven descendant runtime truth to ignore continuation-only hierarchy irradiance/RT RGB that only becomes reachable after inserting an authored descendant node, instead of shifting current red GI while the underlying leaf scene truth stayed fixed; stable_red={stable_red:.2}, changed_red={changed_red:.2}"
    );
    assert!(
        (stable_blue - changed_blue).abs() < 0.05,
        "expected scene-driven descendant runtime truth to ignore continuation-only hierarchy irradiance/RT RGB that only becomes reachable after inserting an authored descendant node, instead of shifting current blue GI while the underlying leaf scene truth stayed fixed; stable_blue={stable_blue:.2}, changed_blue={changed_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_current_gi_when_only_descendant_depth_changes(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.2),
            probe_with_parent(200, 150, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.46),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.44),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([200]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([200]))
        .build();

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_center_luma =
        average_region_luma(&stable.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let changed_center_luma =
        average_region_luma(&changed.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_center_luma > 1.0,
        "expected the descendant scene-driven runtime GI fixture itself to contribute visible indirect light so the descendant-depth check is meaningful; stable_center_luma={stable_center_luma:.2}"
    );
    assert!(
        (stable_center_luma - changed_center_luma).abs() < 0.2,
        "expected scene-driven descendant runtime truth to keep current GI materially aligned even when only an intermediate authored descendant node is inserted, instead of letting descendant depth attenuate the same leaf scene truth; stable_center_luma={stable_center_luma:.2}, changed_center_luma={changed_center_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_scene_driven_descendant_runtime_truth_keeps_scene_prepare_mix_when_only_descendant_depth_changes(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let stable_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let changed_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.2),
            probe_with_parent(200, 150, false, 96, Vec3::ZERO, 1.2),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&stable_extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };
    let runtime = HybridGiResolveRuntime::fixture()
        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
        .with_probe_hierarchy_irradiance_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.32, 0.14], 0.46),
        )]))
        .with_probe_hierarchy_rt_lighting_rgb_and_weight(std::collections::BTreeMap::from([(
            200,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.3, 0.16], 0.44),
        )]))
        .with_probe_scene_driven_hierarchy_irradiance_ids(std::collections::BTreeSet::from([200]))
        .with_probe_scene_driven_hierarchy_rt_lighting_ids(std::collections::BTreeSet::from([200]))
        .build();
    let scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.14, 0.38, 0.92], Vec3::ZERO, 1.0);

    let stable = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(stable_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(runtime.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let changed = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(runtime))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let stable_red = average_region_channel(&stable.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let changed_red =
        average_region_channel(&changed.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let stable_blue =
        average_region_channel(&stable.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let changed_blue =
        average_region_channel(&changed.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        stable_red > 1.0 || stable_blue > 1.0,
        "expected the descendant-runtime plus scene-prepare GI fixture itself to contribute visible indirect light so the descendant-depth mix check is meaningful; stable_red={stable_red:.2}, stable_blue={stable_blue:.2}"
    );
    assert!(
        (stable_red - changed_red).abs() < 0.05,
        "expected scene-driven descendant runtime truth to keep the same red mix against scene-prepare fallback even when only an intermediate authored descendant node is inserted, instead of letting descendant depth change the runtime-vs-scene-prepare blend; stable_red={stable_red:.2}, changed_red={changed_red:.2}"
    );
    assert!(
        (stable_blue - changed_blue).abs() < 0.05,
        "expected scene-driven descendant runtime truth to keep the same blue mix against scene-prepare fallback even when only an intermediate authored descendant node is inserted, instead of letting descendant depth change the runtime-vs-scene-prepare blend; stable_blue={stable_blue:.2}, changed_blue={changed_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_changes_when_resident_ancestor_is_reached_through_nonresident_hierarchy_gap() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![
                    probe(200, true, 128, Vec3::ZERO, 2.2),
                    probe(250, false, 96, Vec3::ZERO, 2.2),
                    probe(300, true, 128, Vec3::ZERO, 2.2),
                ],
                trace_regions.clone(),
            ),
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
        .unwrap();

    let flat = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 128, Vec3::ZERO, 2.2),
                        probe(250, false, 96, Vec3::ZERO, 2.2),
                        probe(300, true, 128, Vec3::ZERO, 2.2),
                    ],
                    trace_regions.clone(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 80, 40],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [40, 96, 255],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();
    let hierarchical = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 128, Vec3::ZERO, 2.2),
                        probe_with_parent(250, 200, false, 96, Vec3::ZERO, 2.2),
                        probe_with_parent(300, 250, true, 128, Vec3::ZERO, 2.2),
                    ],
                    trace_regions,
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 80, 40],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [40, 96, 255],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();

    let flat_red = average_region_channel(&flat.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let flat_blue = average_region_channel(&flat.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);
    let hierarchical_red =
        average_region_channel(&hierarchical.rgba, viewport_size, 0, 0.35, 0.65, 0.25, 0.75);
    let hierarchical_blue =
        average_region_channel(&hierarchical.rgba, viewport_size, 2, 0.35, 0.65, 0.25, 0.75);

    assert!(
        hierarchical_blue > flat_blue + 0.6,
        "expected a resident child probe to gain more blue resolve weight when it reaches a resident ancestor through a nonresident hierarchy gap; flat_blue={flat_blue:.2}, hierarchical_blue={hierarchical_blue:.2}"
    );
    assert!(
        flat_red > hierarchical_red + 0.6,
        "expected the ancestor probe to lose relative red dominance once the resident child is linked through the nonresident hierarchy chain; flat_red={flat_red:.2}, hierarchical_red={hierarchical_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_inherits_farther_resident_ancestor_irradiance_beyond_nearest_resident_parent()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![
                    probe(100, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                    probe(200, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                    probe(250, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                    probe(300, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                ],
                trace_regions.clone(),
            ),
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
        .unwrap();

    let flat = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(100, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                        probe(200, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                        probe(250, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                        probe(300, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                    ],
                    trace_regions.clone(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 100,
                        slot: 0,
                        ray_budget: 96,
                        irradiance_rgb: [255, 80, 40],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 1,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 2,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();
    let hierarchical = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(100, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                        probe_with_parent(200, 100, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                        probe_with_parent(250, 200, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                        probe_with_parent(300, 250, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                    ],
                    trace_regions,
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 100,
                        slot: 0,
                        ray_budget: 96,
                        irradiance_rgb: [255, 80, 40],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 1,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 2,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();

    let flat_chroma = average_region_channel(&flat.rgba, viewport_size, 0, 0.55, 0.9, 0.25, 0.75)
        - average_region_channel(&flat.rgba, viewport_size, 2, 0.55, 0.9, 0.25, 0.75);
    let hierarchical_chroma =
        average_region_channel(&hierarchical.rgba, viewport_size, 0, 0.55, 0.9, 0.25, 0.75)
            - average_region_channel(&hierarchical.rgba, viewport_size, 2, 0.55, 0.9, 0.25, 0.75);

    assert!(
        hierarchical_chroma > flat_chroma + 0.6,
        "expected a resident child probe to inherit warmer farther-ancestor irradiance beyond the nearest resident parent when linked through the hierarchy chain; flat_chroma={flat_chroma:.2}, hierarchical_chroma={hierarchical_chroma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_increases_child_intensity_when_farther_resident_ancestor_has_more_budget() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();

    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![
                    probe(100, true, 1, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                    probe_with_parent(200, 100, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                    probe_with_parent(250, 200, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                    probe_with_parent(300, 250, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                ],
                trace_regions.clone(),
            ),
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
        .unwrap();

    let low_budget = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(100, true, 1, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                        probe_with_parent(200, 100, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                        probe_with_parent(250, 200, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                        probe_with_parent(300, 250, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                    ],
                    trace_regions.clone(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 100,
                        slot: 0,
                        ray_budget: 1,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 1,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 2,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();
    let high_budget = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(100, true, 255, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                        probe_with_parent(200, 100, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                        probe_with_parent(250, 200, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                        probe_with_parent(300, 250, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                    ],
                    trace_regions,
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 100,
                        slot: 0,
                        ray_budget: 255,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 1,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 2,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
                    },
                ],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            })),
            &compiled,
            None,
        )
        .unwrap();

    let low_budget_luma =
        average_region_luma(&low_budget.rgba, viewport_size, 0.55, 0.9, 0.25, 0.75);
    let high_budget_luma =
        average_region_luma(&high_budget.rgba, viewport_size, 0.55, 0.9, 0.25, 0.75);

    assert!(
        high_budget_luma > low_budget_luma + 0.35,
        "expected a farther resident ancestor with more ray budget to increase the child probe's final resolve intensity instead of only changing lineage color continuation; low_budget_luma={low_budget_luma:.2}, high_budget_luma={high_budget_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_strengthens_farther_ancestor_rt_tint_when_budget_increases() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = vec![trace_region_with_bounds_and_rt_lighting(
        40,
        Vec3::new(-0.85, 0.0, 0.0),
        0.75,
        0.95,
        [255, 64, 32],
    )];

    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![
                    probe(100, true, 1, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                    probe_with_parent(200, 100, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                    probe_with_parent(250, 200, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                    probe_with_parent(300, 250, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                ],
                trace_regions.clone(),
            ),
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
        .unwrap();

    let low_budget = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(100, true, 1, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                        probe_with_parent(200, 100, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                        probe_with_parent(250, 200, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                        probe_with_parent(300, 250, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                    ],
                    trace_regions.clone(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 100,
                        slot: 0,
                        ray_budget: 1,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 1,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 2,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
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
    let high_budget = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(100, true, 255, Vec3::new(-0.85, 0.0, 0.0), 0.65),
                        probe_with_parent(200, 100, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                        probe_with_parent(250, 200, false, 96, Vec3::new(0.2, 0.0, 0.0), 0.9),
                        probe_with_parent(300, 250, true, 128, Vec3::new(0.65, 0.0, 0.0), 1.1),
                    ],
                    trace_regions,
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 100,
                        slot: 0,
                        ray_budget: 255,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 1,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 2,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
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

    let low_budget_chroma =
        average_region_channel(&low_budget.rgba, viewport_size, 0, 0.55, 0.9, 0.25, 0.75)
            - average_region_channel(&low_budget.rgba, viewport_size, 2, 0.55, 0.9, 0.25, 0.75);
    let high_budget_chroma =
        average_region_channel(&high_budget.rgba, viewport_size, 0, 0.55, 0.9, 0.25, 0.75)
            - average_region_channel(&high_budget.rgba, viewport_size, 2, 0.55, 0.9, 0.25, 0.75);

    assert!(
        high_budget_chroma > low_budget_chroma + 0.35,
        "expected farther-ancestor RT tint inheritance to strengthen when the resident ancestor has more ray budget instead of staying flat; low_budget_chroma={low_budget_chroma:.2}, high_budget_chroma={high_budget_chroma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_inherits_rt_lighting_tint_through_nonresident_hierarchy_gap() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = vec![trace_region_with_bounds_and_rt_lighting(
        40,
        Vec3::new(-0.85, 0.0, 0.0),
        0.75,
        0.95,
        [255, 64, 32],
    )];
    let compiled = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            &build_extract_with_probes_and_trace_regions(
                viewport_size,
                vec![
                    probe(200, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.8),
                    probe(250, false, 96, Vec3::new(-0.25, 0.0, 0.0), 0.9),
                    probe(300, true, 128, Vec3::new(0.3, 0.0, 0.0), 1.4),
                ],
                trace_regions.clone(),
            ),
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
        .unwrap();

    let flat = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.8),
                        probe(250, false, 96, Vec3::new(-0.25, 0.0, 0.0), 0.9),
                        probe(300, true, 128, Vec3::new(0.3, 0.0, 0.0), 1.4),
                    ],
                    trace_regions.clone(),
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
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
    let hierarchical = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(
                build_extract_with_probes_and_trace_regions(
                    viewport_size,
                    vec![
                        probe(200, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.8),
                        probe_with_parent(250, 200, false, 96, Vec3::new(-0.25, 0.0, 0.0), 0.9),
                        probe_with_parent(300, 250, true, 128, Vec3::new(0.3, 0.0, 0.0), 1.4),
                    ],
                    trace_regions,
                ),
                viewport_size,
            )
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![
                    HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 96,
                        irradiance_rgb: [160, 160, 160],
                    },
                    HybridGiPrepareProbe {
                        probe_id: 300,
                        slot: 1,
                        ray_budget: 128,
                        irradiance_rgb: [160, 160, 160],
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

    let flat_chroma = average_region_channel(&flat.rgba, viewport_size, 0, 0.45, 0.75, 0.25, 0.75)
        - average_region_channel(&flat.rgba, viewport_size, 2, 0.45, 0.75, 0.25, 0.75);
    let hierarchical_chroma =
        average_region_channel(&hierarchical.rgba, viewport_size, 0, 0.45, 0.75, 0.25, 0.75)
            - average_region_channel(&hierarchical.rgba, viewport_size, 2, 0.45, 0.75, 0.25, 0.75);

    assert!(
        hierarchical_chroma > flat_chroma + 0.6,
        "expected a child probe to inherit warmer RT-lighting resolve tint through a nonresident hierarchy gap when only its resident ancestor is directly covered by the scheduled trace region; flat_chroma={flat_chroma:.2}, hierarchical_chroma={hierarchical_chroma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_gpu_trace_lighting_source_without_current_trace_schedule() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [160, 160, 160],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_rt_lighting_rgb(std::collections::BTreeMap::from([(
                            200,
                            [240, 96, 48],
                        )]))
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_rt_lighting_rgb(std::collections::BTreeMap::from([(
                            200,
                            [48, 96, 240],
                        )]))
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected runtime-host GI resolve to consume GPU-produced warm trace-lighting history even when the current frame has no scheduled trace regions; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected runtime-host GI resolve to consume GPU-produced cool trace-lighting history even when the current frame has no scheduled trace regions; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_hierarchy_irradiance_and_weight_without_current_ancestor_prepare()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(2.0)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                200,
                                HybridGiResolveRuntime::pack_rgb_and_weight(
                                    [0.95, 0.28, 0.12],
                                    0.55,
                                ),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(2.0)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                200,
                                HybridGiResolveRuntime::pack_rgb_and_weight(
                                    [0.12, 0.28, 0.95],
                                    0.55,
                                ),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.6,
        "expected runtime-host GI resolve to consume runtime hierarchy irradiance and weight even when the current frame no longer carries resident ancestor prepare data; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.6,
        "expected runtime-host GI resolve to consume runtime hierarchy irradiance and weight even when the current frame no longer carries resident ancestor prepare data; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_hierarchy_rt_lighting_without_current_trace_schedule() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                200,
                                HybridGiResolveRuntime::pack_rgb_and_weight([0.95, 0.3, 0.12], 0.5),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                200,
                                HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.3, 0.95], 0.5),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected runtime-host GI resolve to consume hierarchy RT-lighting continuation from runtime/GPU history even when the current frame has no scheduled trace regions; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected runtime-host GI resolve to consume hierarchy RT-lighting continuation from runtime/GPU history even when the current frame has no scheduled trace regions; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_scene_prepare_voxel_fallback_without_current_trace_schedule() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 4.0,
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: Vec::new(),
                    voxel_clipmaps: vec![clipmap.clone()],
                    voxel_cells: vec![
                        HybridGiPrepareVoxelCell {
                            clipmap_id: 7,
                            cell_index: 20,
                            occupancy_count: 4,
                            dominant_card_id: 0,
                            radiance_present: false,
                            radiance_rgb: [0, 0, 0],
                        },
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
                            cell_index: 24,
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
                    ],
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: Vec::new(),
                    voxel_clipmaps: vec![clipmap],
                    voxel_cells: vec![
                        HybridGiPrepareVoxelCell {
                            clipmap_id: 7,
                            cell_index: 38,
                            occupancy_count: 4,
                            dominant_card_id: 0,
                            radiance_present: false,
                            radiance_rgb: [0, 0, 0],
                        },
                        HybridGiPrepareVoxelCell {
                            clipmap_id: 7,
                            cell_index: 39,
                            occupancy_count: 4,
                            dominant_card_id: 0,
                            radiance_present: false,
                            radiance_rgb: [0, 0, 0],
                        },
                        HybridGiPrepareVoxelCell {
                            clipmap_id: 7,
                            cell_index: 42,
                            occupancy_count: 4,
                            dominant_card_id: 0,
                            radiance_present: false,
                            radiance_rgb: [0, 0, 0],
                        },
                        HybridGiPrepareVoxelCell {
                            clipmap_id: 7,
                            cell_index: 43,
                            occupancy_count: 4,
                            dominant_card_id: 0,
                            radiance_present: false,
                            radiance_rgb: [0, 0, 0],
                        },
                    ],
                })),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected GI resolve to use scene_prepare voxel fallback when no trace regions are scheduled, producing warmer indirect light for warm-side voxel cells; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected GI resolve to use scene_prepare voxel fallback when no trace regions are scheduled, producing cooler indirect light for cool-side voxel cells; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_scene_prepare_voxel_clipmap_fallback_without_runtime_voxel_cells() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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

    let near = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: Vec::new(),
                    voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                        clipmap_id: 7,
                        center: Vec3::new(0.0, 0.0, 0.2),
                        half_extent: 4.0,
                    }],
                    voxel_cells: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let far = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: Vec::new(),
                    voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                        clipmap_id: 7,
                        center: Vec3::new(9.0, 9.0, 9.0),
                        half_extent: 4.0,
                    }],
                    voxel_cells: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let near_luma = average_region_luma(&near.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let far_luma = average_region_luma(&far.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        near_luma > far_luma + 0.5,
        "expected GI resolve to use coarse scene_prepare voxel clipmap fallback when runtime voxel cells are absent, so a nearby clipmap still adds more indirect light than a far clipmap; near_luma={near_luma:.2}, far_luma={far_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_scene_voxel_tint_when_layout_stays_fixed() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let warm_scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([1.0, 0.18, 0.06], Vec3::ZERO, 2.0);
    let cool_scene_prepare =
        runtime_voxel_scene_prepare_from_tinted_mesh([0.06, 0.18, 1.0], Vec3::ZERO, 2.0);

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
        "expected warm/cool fixtures to keep identical runtime voxel layout so this regression only checks shading authority"
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

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected scene-driven runtime voxel fallback to preserve warm mesh tint when voxel layout stays fixed instead of collapsing both outputs to the same spatial heuristic; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected scene-driven runtime voxel fallback to preserve cool mesh tint when voxel layout stays fixed instead of collapsing both outputs to the same spatial heuristic; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_scene_voxel_point_light_seed_when_layout_and_tint_stay_fixed() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let warm_scene_prepare = runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
        [1.0, 1.0, 1.0],
        Vec3::ZERO,
        2.0,
        &[],
        &[test_point_light(
            10,
            Vec3::new(0.0, 0.0, 0.35),
            Vec3::new(1.0, 0.12, 0.05),
            4.0,
            3.0,
        )],
        &[],
    );
    let cool_scene_prepare = runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
        [1.0, 1.0, 1.0],
        Vec3::ZERO,
        2.0,
        &[],
        &[test_point_light(
            10,
            Vec3::new(0.0, 0.0, 0.35),
            Vec3::new(0.05, 0.12, 1.0),
            4.0,
            3.0,
        )],
        &[],
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
        "expected warm/cool point-light fixtures to keep identical runtime voxel layout so this regression only checks direct-light seed authority"
    );
    assert_ne!(
        warm_scene_prepare
            .voxel_cells
            .iter()
            .map(|cell| (cell.clipmap_id, cell.cell_index, cell.radiance_rgb))
            .collect::<Vec<_>>(),
        cool_scene_prepare
            .voxel_cells
            .iter()
            .map(|cell| (cell.clipmap_id, cell.cell_index, cell.radiance_rgb))
            .collect::<Vec<_>>(),
        "expected runtime voxel cell radiance to change with split-light direct seed even when layout and mesh tint stay fixed"
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

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected scene-driven runtime voxel fallback to preserve warm point-light seed when voxel layout and mesh tint stay fixed; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected scene-driven runtime voxel fallback to preserve cool point-light seed when voxel layout and mesh tint stay fixed; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_scene_voxel_owner_card_capture_seed_when_layout_and_owner_stay_fixed(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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
    let base_card_capture_request = HybridGiPrepareCardCaptureRequest {
        card_id: 11,
        page_id: 22,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::new(20.0, 20.0, 20.0),
        bounds_radius: 0.25,
    };
    let voxel_layout = vec![
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 20,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
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
            cell_index: 24,
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
    ];

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: vec![base_card_capture_request.clone()],
                    surface_cache_page_contents: Vec::new(),
                    voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                        clipmap_id: 7,
                        center: Vec3::ZERO,
                        half_extent: 4.0,
                    }],
                    voxel_cells: voxel_layout.clone(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                        capture_slot_id: 31,
                        ..base_card_capture_request
                    }],
                    surface_cache_page_contents: Vec::new(),
                    voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                        clipmap_id: 7,
                        center: Vec3::ZERO,
                        half_extent: 4.0,
                    }],
                    voxel_cells: voxel_layout,
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_capture = warm_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected warm capture slot sample");
    let cool_capture = cool_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 31)
                .map(|sample| sample.1)
        })
        .expect("expected cool capture slot sample");
    assert_ne!(
        warm_capture, cool_capture,
        "expected capture-slot readback to change when only the matched owner request seed changes"
    );

    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        (cool_blue - warm_blue).abs() > 0.1,
        "expected scene-driven runtime voxel fallback to reuse the matched card-capture seed when radiance is absent, so changing only that card seed still changes final GI resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}, warm_capture={warm_capture:?}, cool_capture={cool_capture:?}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_persisted_surface_cache_page_sample_when_layout_and_owner_stay_fixed() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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
    let base_page_content = HybridGiPrepareSurfaceCachePageContent {
        page_id: 11,
        owner_card_id: 11,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::ZERO,
        bounds_radius: 0.25,
        atlas_sample_rgba: [224, 112, 64, 255],
        capture_sample_rgba: [240, 96, 48, 255],
    };
    let voxel_layout = vec![
        HybridGiPrepareVoxelCell {
            clipmap_id: 7,
            cell_index: 20,
            occupancy_count: 4,
            dominant_card_id: 11,
            radiance_present: false,
            radiance_rgb: [0, 0, 0],
        },
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
            cell_index: 24,
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
    ];

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: vec![base_page_content],
                    voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                        clipmap_id: 7,
                        center: Vec3::ZERO,
                        half_extent: 4.0,
                    }],
                    voxel_cells: voxel_layout.clone(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                        atlas_sample_rgba: [64, 112, 224, 255],
                        capture_sample_rgba: [48, 96, 240, 255],
                        ..base_page_content
                    }],
                    voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                        clipmap_id: 7,
                        center: Vec3::ZERO,
                        half_extent: 4.0,
                    }],
                    voxel_cells: voxel_layout,
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_capture = warm_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected warm persisted capture slot sample");
    let cool_capture = cool_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected cool persisted capture slot sample");
    assert_ne!(
        warm_capture, cool_capture,
        "expected persisted clean-frame surface-cache samples to survive scene_prepare readback when there is no current card-capture request"
    );

    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        (cool_blue - warm_blue).abs() > 0.1,
        "expected final GI resolve to reuse persisted clean-frame surface-cache page samples when runtime voxel radiance is absent and owner/layout stay fixed; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}, warm_capture={warm_capture:?}, cool_capture={cool_capture:?}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_persisted_surface_cache_page_sample_without_runtime_voxel_support() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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
    let base_page_content = HybridGiPrepareSurfaceCachePageContent {
        page_id: 11,
        owner_card_id: 11,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::ZERO,
        bounds_radius: 0.6,
        atlas_sample_rgba: [224, 112, 64, 255],
        capture_sample_rgba: [240, 96, 48, 255],
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: vec![base_page_content],
                    voxel_clipmaps: Vec::new(),
                    voxel_cells: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let warm_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: Vec::new(),
                    surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                        atlas_sample_rgba: [64, 112, 224, 255],
                        capture_sample_rgba: [48, 96, 240, 255],
                        ..base_page_content
                    }],
                    voxel_clipmaps: Vec::new(),
                    voxel_cells: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let warm_capture = warm_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected warm persisted capture slot sample without runtime voxel support");
    let cool_capture = cool_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected cool persisted capture slot sample without runtime voxel support");
    assert_ne!(
        warm_capture, cool_capture,
        "expected persisted clean-frame surface-cache samples to survive scene_prepare readback even when runtime voxel support is absent"
    );

    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        (cool_blue - warm_blue).abs() > 0.1,
        "expected final GI resolve to reuse persisted clean-frame surface-cache page samples even when runtime voxel support is absent; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}, warm_capture={warm_capture:?}, cool_capture={cool_capture:?}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_scene_voxel_radiance_rehydrated_from_persisted_page_sample_on_clean_frame(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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
            .map(|cell| (cell.clipmap_id, cell.cell_index, cell.radiance_present, cell.radiance_rgb))
            .collect::<Vec<_>>(),
        cool_scene_prepare
            .voxel_cells
            .iter()
            .map(|cell| (cell.clipmap_id, cell.cell_index, cell.radiance_present, cell.radiance_rgb))
            .collect::<Vec<_>>(),
        "expected persisted page samples to rehydrate different runtime voxel radiance while layout stays fixed"
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

    let warm = renderer
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
    let cool = renderer
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
        .scene_prepare_resources()
        .expect("expected warm scene-prepare resource snapshot");
    let cool_scene_prepare_resources = cool_readback
        .scene_prepare_resources()
        .expect("expected cool scene-prepare resource snapshot");
    assert!(
        warm_scene_prepare_resources
            .capture_slot_rgba_samples()
            .is_empty()
            && warm_scene_prepare_resources
                .atlas_slot_rgba_samples()
                .is_empty()
            && cool_scene_prepare_resources
                .capture_slot_rgba_samples()
                .is_empty()
            && cool_scene_prepare_resources
                .atlas_slot_rgba_samples()
                .is_empty(),
        "expected this clean-frame regression to remove persisted surface-cache page-content fallback from the renderer input, so the resolve difference must come from runtime voxel radiance instead of owner-card capture resources"
    );

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected clean-frame persisted page samples to rehydrate warm runtime voxel radiance even after owner-card surface-cache page fallback is removed from the renderer input; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected clean-frame persisted page samples to rehydrate cool runtime voxel radiance even after owner-card surface-cache page fallback is removed from the renderer input; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_changes_when_runtime_scene_voxel_owner_matches_scene_card_capture_material_seed_with_fixed_layout(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let probes = vec![probe(200, true, 128, Vec3::ZERO, 1.8)];
    let offscreen_transform =
        crate::core::math::Transform::from_translation(Vec3::new(10_000.0, 10_000.0, 10_000.0));
    let directional_lights = vec![
        crate::core::framework::render::RenderDirectionalLightSnapshot {
            node_id: 1,
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 2.0,
        },
    ];
    let default_material_extract = {
        let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
        snapshot.scene.meshes = vec![crate::core::framework::render::RenderMeshSnapshot {
            node_id: 11,
            transform: offscreen_transform,
            model: crate::core::resource::ResourceHandle::<crate::core::resource::ModelMarker>::new(
                crate::core::resource::ResourceId::from_stable_label("builtin://cube"),
            ),
            material:
                crate::core::resource::ResourceHandle::<crate::core::resource::MaterialMarker>::new(
                    crate::core::resource::ResourceId::from_stable_label(
                        "builtin://material/default",
                    ),
                ),
            tint: crate::core::math::Vec4::ONE,
            mobility: crate::core::framework::scene::Mobility::Static,
            render_layer_mask: u32::MAX,
        }];
        snapshot.scene.directional_lights = directional_lights.clone();
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
            probe_budget: 1,
            tracing_budget: 0,
            probes: probes.clone(),
            trace_regions: Vec::new(),
        });
        extract
    };
    let missing_material_extract = {
        let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
        snapshot.scene.meshes = vec![crate::core::framework::render::RenderMeshSnapshot {
            node_id: 11,
            transform: offscreen_transform,
            model: crate::core::resource::ResourceHandle::<crate::core::resource::ModelMarker>::new(
                crate::core::resource::ResourceId::from_stable_label("builtin://cube"),
            ),
            material:
                crate::core::resource::ResourceHandle::<crate::core::resource::MaterialMarker>::new(
                    crate::core::resource::ResourceId::from_stable_label(
                        "builtin://missing-material",
                    ),
                ),
            tint: crate::core::math::Vec4::ONE,
            mobility: crate::core::framework::scene::Mobility::Static,
            render_layer_mask: u32::MAX,
        }];
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
            probe_budget: 1,
            tracing_budget: 0,
            probes,
            trace_regions: Vec::new(),
        });
        extract
    };
    let compiled = compile_hybrid_gi_pipeline(&default_material_extract);

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
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
            card_id: 11,
            page_id: 22,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::new(20.0, 20.0, 20.0),
            bounds_radius: 0.25,
        }],
        surface_cache_page_contents: Vec::new(),
        voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 4.0,
        }],
        voxel_cells: vec![
            HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 20,
                occupancy_count: 4,
                dominant_card_id: 11,
                radiance_present: false,
                radiance_rgb: [0, 0, 0],
            },
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
                cell_index: 24,
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
        ],
    };

    let default_material = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(default_material_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let default_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();
    let missing_material = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(missing_material_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let missing_readback = renderer.take_last_hybrid_gi_gpu_readback().unwrap();

    let default_capture = default_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected default-material capture slot sample");
    let missing_capture = missing_readback
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected missing-material capture slot sample");
    assert_ne!(
        default_capture, missing_capture,
        "expected scene-prepare capture samples to differ when only scene material truth changes while request layout stays fixed"
    );

    let default_green = average_region_channel(
        &default_material.rgba,
        viewport_size,
        1,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let missing_green = average_region_channel(
        &missing_material.rgba,
        viewport_size,
        1,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        (default_green - missing_green).abs() > 0.1,
        "expected final resolve to change when the authoritative scene card-capture material seed changes while scene-prepare layout, voxel owner, and radiance stay fixed; default_green={default_green:.2}, missing_green={missing_green:.2}, default_capture={default_capture:?}, missing_capture={missing_capture:?}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_descendant_scene_driven_runtime_irradiance_for_parent_probe_after_schedule_clears(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(100, true, 96, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm_runtime = descendant_scene_driven_parent_irradiance_runtime_for_resolve([240, 96, 48]);
    let cool_runtime = descendant_scene_driven_parent_irradiance_runtime_for_resolve([48, 96, 240]);

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(warm_runtime)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(cool_runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected render resolve to keep consuming descendant-driven runtime irradiance for the parent merge-back probe after the child trace schedule clears, instead of dropping the parent back to the same flat/no-hierarchy output; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected render resolve to keep consuming descendant-driven runtime irradiance for the parent merge-back probe after the child trace schedule clears, instead of dropping the parent back to the same flat/no-hierarchy output; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_descendant_scene_driven_runtime_rt_for_parent_probe_after_schedule_clears(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(100, true, 96, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm_runtime = descendant_scene_driven_parent_rt_runtime_for_resolve([240, 96, 48]);
    let cool_runtime = descendant_scene_driven_parent_rt_runtime_for_resolve([48, 96, 240]);

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(warm_runtime)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(cool_runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected render resolve to keep consuming descendant-driven runtime RT-lighting for the parent merge-back probe after the child trace schedule clears, instead of dropping the parent back to the same flat/no-hierarchy output; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected render resolve to keep consuming descendant-driven runtime RT-lighting for the parent merge-back probe after the child trace schedule clears, instead of dropping the parent back to the same flat/no-hierarchy output; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gathers_requested_descendant_runtime_irradiance_when_parent_exact_entry_is_missing(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm_runtime =
        descendant_scene_driven_parent_irradiance_runtime_without_parent_exact_for_resolve([
            240, 96, 48,
        ]);
    let cool_runtime =
        descendant_scene_driven_parent_irradiance_runtime_without_parent_exact_for_resolve([
            48, 96, 240,
        ]);

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(warm_runtime)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(cool_runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected final resolve to keep gathering requested descendant runtime irradiance for a resident parent probe when the parent exact entry is missing, instead of collapsing both paths to the same flat output; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected final resolve to keep gathering requested descendant runtime irradiance for a resident parent probe when the parent exact entry is missing, instead of collapsing both paths to the same flat output; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gathers_requested_descendant_runtime_rt_when_parent_exact_entry_is_missing() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm_runtime =
        descendant_scene_driven_parent_rt_runtime_without_parent_exact_for_resolve([240, 96, 48]);
    let cool_runtime =
        descendant_scene_driven_parent_rt_runtime_without_parent_exact_for_resolve([48, 96, 240]);

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(warm_runtime)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(cool_runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected final resolve to keep gathering requested descendant runtime RT-lighting for a resident parent probe when the parent exact entry is missing, instead of collapsing both paths to the same flat output; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected final resolve to keep gathering requested descendant runtime RT-lighting for a resident parent probe when the parent exact entry is missing, instead of collapsing both paths to the same flat output; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gathers_requested_descendant_runtime_resolve_weight_when_parent_exact_entry_is_missing(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [176, 176, 176],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let supported_runtime =
        descendant_scene_driven_parent_resolve_runtime_without_parent_exact_for_resolve(true);
    let flat_runtime =
        descendant_scene_driven_parent_resolve_runtime_without_parent_exact_for_resolve(false);

    let supported = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(supported_runtime)),
            &compiled,
            None,
        )
        .unwrap();
    let flat = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(flat_runtime)),
            &compiled,
            None,
        )
        .unwrap();

    let supported_luma =
        average_region_luma(&supported.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let flat_luma = average_region_luma(&flat.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        supported_luma > flat_luma + 0.5,
        "expected final resolve to keep gathering requested descendant runtime resolve-weight for a resident parent probe when the parent exact entry is missing, instead of collapsing back to the same flat intensity; supported_luma={supported_luma:.2}, flat_luma={flat_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gathers_runtime_grandparent_irradiance_when_exact_probe_entry_is_missing() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                100,
                                HybridGiResolveRuntime::pack_rgb_and_weight(
                                    [0.95, 0.28, 0.12],
                                    0.68,
                                ),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                100,
                                HybridGiResolveRuntime::pack_rgb_and_weight(
                                    [0.12, 0.28, 0.95],
                                    0.68,
                                ),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected final GI resolve to keep consuming runtime grandparent irradiance for a resident child probe when the exact probe runtime entry is missing, instead of collapsing both paths to the same flat output; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected final GI resolve to keep consuming runtime grandparent irradiance for a resident child probe when the exact probe runtime entry is missing, instead of collapsing both paths to the same flat output; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gathers_runtime_grandparent_rt_lighting_when_exact_probe_entry_is_missing() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                100,
                                HybridGiResolveRuntime::pack_rgb_and_weight(
                                    [0.95, 0.3, 0.12],
                                    0.62,
                                ),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([(
                                100,
                                HybridGiResolveRuntime::pack_rgb_and_weight(
                                    [0.12, 0.3, 0.95],
                                    0.62,
                                ),
                            )]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected final GI resolve to keep consuming runtime grandparent RT-lighting for a resident child probe when the exact probe runtime entry is missing, instead of collapsing both paths to the same flat output; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected final GI resolve to keep consuming runtime grandparent RT-lighting for a resident child probe when the exact probe runtime entry is missing, instead of collapsing both paths to the same flat output; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_ignores_zero_weight_exact_irradiance_entry_and_keeps_runtime_ancestor_gather()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.95, 0.28, 0.12],
                                        0.68,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.05, 0.05, 0.05],
                                        0.0,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.12, 0.28, 0.95],
                                        0.68,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.05, 0.05, 0.05],
                                        0.0,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected a zero-weight exact hierarchy irradiance entry to stop shadowing a stronger runtime ancestor gather in final resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected a zero-weight exact hierarchy irradiance entry to stop shadowing a stronger runtime ancestor gather in final resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_ignores_zero_weight_exact_rt_entry_and_keeps_runtime_ancestor_gather() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.95, 0.3, 0.12],
                                        0.62,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.05, 0.05, 0.05],
                                        0.0,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.12, 0.3, 0.95],
                                        0.62,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.05, 0.05, 0.05],
                                        0.0,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected a zero-weight exact hierarchy RT entry to stop shadowing a stronger runtime ancestor gather in final resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected a zero-weight exact hierarchy RT entry to stop shadowing a stronger runtime ancestor gather in final resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_blends_nonzero_exact_irradiance_entry_with_runtime_ancestor_gather() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

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

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(2.0)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.95, 0.28, 0.12],
                                        0.68,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.18,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(2.0)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.12, 0.28, 0.95],
                                        0.68,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.18,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected a non-zero exact hierarchy irradiance entry to keep blending with runtime ancestor gather instead of fully shadowing it in final resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected a non-zero exact hierarchy irradiance entry to keep blending with runtime ancestor gather instead of fully shadowing it in final resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_blends_nonzero_exact_rt_entry_with_runtime_ancestor_gather() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.95, 0.3, 0.12],
                                        0.62,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.18,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.12, 0.3, 0.95],
                                        0.62,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.18,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected a non-zero exact hierarchy RT entry to keep blending with runtime ancestor gather instead of fully shadowing it in final resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected a non-zero exact hierarchy RT entry to keep blending with runtime ancestor gather instead of fully shadowing it in final resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_blends_nonzero_exact_resolve_weight_with_runtime_ancestor_gather() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [176, 176, 176],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let strong = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(2.4)),
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(0.6)),
                        ]))
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let weak = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(0.6)),
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(0.6)),
                        ]))
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let strong_luma = average_region_luma(&strong.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let weak_luma = average_region_luma(&weak.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        strong_luma > weak_luma + 0.5,
        "expected a non-zero exact hierarchy resolve-weight entry to keep blending with stronger runtime ancestor continuation instead of fully shadowing it in final resolve; strong_luma={strong_luma:.2}, weak_luma={weak_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_blends_nonzero_exact_irradiance_entry_with_requested_descendant_runtime() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(1.2)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.12,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.95, 0.28, 0.12],
                                        0.68,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(1.2)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.12,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.12, 0.28, 0.95],
                                        0.68,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected a non-zero exact parent hierarchy irradiance entry to keep blending with requested descendant runtime continuation instead of fully shadowing it in final resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected a non-zero exact parent hierarchy irradiance entry to keep blending with requested descendant runtime continuation instead of fully shadowing it in final resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_blends_nonzero_exact_rt_entry_with_requested_descendant_runtime() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.12,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.95, 0.3, 0.12],
                                        0.62,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
                        .with_probe_hierarchy_rt_lighting_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.5, 0.5, 0.5],
                                        0.12,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.12, 0.3, 0.95],
                                        0.62,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected a non-zero exact parent hierarchy RT entry to keep blending with requested descendant runtime continuation instead of fully shadowing it in final resolve; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected a non-zero exact parent hierarchy RT entry to keep blending with requested descendant runtime continuation instead of fully shadowing it in final resolve; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_blends_nonzero_exact_resolve_weight_with_requested_descendant_runtime() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, true, 128, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 100,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [176, 176, 176],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let strong = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(0.6)),
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(2.4)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.82, 0.82, 0.82],
                                        0.18,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.82, 0.82, 0.82],
                                        0.68,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let weak = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(200, 100)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(0.6)),
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(0.6)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.82, 0.82, 0.82],
                                        0.18,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.82, 0.82, 0.82],
                                        0.68,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let strong_luma = average_region_luma(&strong.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let weak_luma = average_region_luma(&weak.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        strong_luma > weak_luma + 0.5,
        "expected a non-zero exact parent hierarchy resolve-weight entry to keep blending with requested descendant runtime continuation instead of fully shadowing it in final resolve; strong_luma={strong_luma:.2}, weak_luma={weak_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_gathers_runtime_grandparent_resolve_weight_when_leaf_entry_is_zeroed() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(150, 100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 150, true, 128, Vec3::ZERO, 1.8),
        ],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let high_weight = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(2.4)),
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(0.0)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.82, 0.82, 0.82],
                                        0.68,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.05, 0.05, 0.05],
                                        0.0,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();
    let low_weight = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_parent_probes(runtime_parent_topology([(150, 100), (200, 150)]))
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (100, HybridGiResolveRuntime::pack_resolve_weight_q8(0.6)),
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(0.0)),
                        ]))
                        .with_probe_hierarchy_irradiance_rgb_and_weight(
                            std::collections::BTreeMap::from([
                                (
                                    100,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.82, 0.82, 0.82],
                                        0.68,
                                    ),
                                ),
                                (
                                    200,
                                    HybridGiResolveRuntime::pack_rgb_and_weight(
                                        [0.05, 0.05, 0.05],
                                        0.0,
                                    ),
                                ),
                            ]),
                        )
                        .build(),
                )),
            &compiled,
            None,
        )
        .unwrap();

    let high_luma = average_region_luma(&high_weight.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);
    let low_luma = average_region_luma(&low_weight.rgba, viewport_size, 0.25, 0.75, 0.25, 0.75);

    assert!(
        high_luma > low_luma + 0.4,
        "expected final resolve to keep gathering runtime grandparent resolve weight for a resident child probe even when the leaf entry is zeroed, instead of flattening both outputs to the same intensity; high_luma={high_luma:.2}, low_luma={low_luma:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_reuses_global_illumination_history_when_scene_history_resolve_is_disabled() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(viewport_size);
    let compiled = compile_hybrid_gi_pipeline(&extract);
    let history_handle = crate::FrameHistoryHandle::new(1);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [255, 96, 48],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 64,
                            irradiance_rgb: [224, 72, 32],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    let with_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [0, 0, 0],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 64,
                            irradiance_rgb: [0, 0, 0],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            Some(history_handle),
        )
        .unwrap();
    let without_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [0, 0, 0],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 64,
                            irradiance_rgb: [0, 0, 0],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            None,
        )
        .unwrap();

    let with_history_red =
        average_region_channel(&with_history.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let without_history_red = average_region_channel(
        &without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        with_history_red > without_history_red + 0.5,
        "expected GlobalIllumination frame history to keep warm indirect-light energy on the second frame even when generic scene-color history resolve stays disabled; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_global_illumination_history_when_probe_support_moves_off_pixel() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let left_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::new(-0.9, 0.0, 0.0), 1.8)],
        Vec::new(),
    );
    let right_extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::new(0.9, 0.0, 0.0), 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&left_extract);
    let history_handle = crate::FrameHistoryHandle::new(7);
    let warm_prepare = Some(HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [255, 96, 48],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    });

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(left_extract, viewport_size)
                .with_hybrid_gi_prepare(warm_prepare.clone()),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    let moved_with_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(right_extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(warm_prepare.clone()),
            &compiled,
            Some(history_handle),
        )
        .unwrap();
    let moved_without_history = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(right_extract, viewport_size)
                .with_hybrid_gi_prepare(warm_prepare),
            &compiled,
            None,
        )
        .unwrap();

    let with_history_left_red =
        average_half_channel(&moved_with_history.rgba, viewport_size, 0, Half::Left);
    let without_history_left_red =
        average_half_channel(&moved_without_history.rgba, viewport_size, 0, Half::Left);

    assert!(
        with_history_left_red < without_history_left_red + 0.5,
        "expected GI history to reject stale left-side indirect-light contribution once probe support moves to the right half instead of leaving a visible screen-space ghost; with_history_left_red={with_history_left_red:.2}, without_history_left_red={without_history_left_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_preserves_more_history_when_hierarchy_resolve_weight_is_stronger() {
    let high_weight = render_hybrid_gi_history_with_second_frame_resolve_weight(2.4);
    let low_weight = render_hybrid_gi_history_with_second_frame_resolve_weight(0.6);
    let viewport_size = UVec2::new(high_weight.width, high_weight.height);

    let high_red =
        average_region_channel(&high_weight.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let low_red =
        average_region_channel(&low_weight.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);

    assert!(
        high_red > low_red + 0.5,
        "expected stronger hierarchy resolve weight to preserve more GI history on the second frame under identical screen support, instead of treating temporal accumulation as purely flat screen-space blend; high_red={high_red:.2}, low_red={low_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_global_illumination_history_when_probe_identity_changes_at_same_support(
) {
    let with_history = render_hybrid_gi_history_with_changed_probe_identity(true);
    let without_history = render_hybrid_gi_history_with_changed_probe_identity(false);
    let viewport_size = UVec2::new(with_history.width, with_history.height);

    let with_history_red =
        average_region_channel(&with_history.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let without_history_red = average_region_channel(
        &without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        with_history_red < without_history_red + 0.5,
        "expected GI history to reset when the dominant probe identity changes even if screen support stays the same, instead of preserving stale radiance from a different lineage; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_global_illumination_history_when_deeper_probe_lineage_changes() {
    let with_history = render_hybrid_gi_history_with_changed_deeper_probe_lineage(true);
    let without_history = render_hybrid_gi_history_with_changed_deeper_probe_lineage(false);
    let viewport_size = UVec2::new(with_history.width, with_history.height);

    let with_history_red =
        average_region_channel(&with_history.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let without_history_red = average_region_channel(
        &without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        with_history_red < without_history_red + 0.5,
        "expected GI history to reset when an unchanged child probe reconnects to a different ancestor lineage at the same screen support, instead of preserving stale radiance from the previous hierarchy; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_ignores_non_dominant_probe_confidence_when_blending_history() {
    let polluted = render_hybrid_gi_history_with_non_dominant_confidence_pollution(true);
    let clean = render_hybrid_gi_history_with_non_dominant_confidence_pollution(false);
    let viewport_size = UVec2::new(polluted.width, polluted.height);

    let polluted_red =
        average_region_channel(&polluted.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let clean_red = average_region_channel(&clean.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);

    assert!(
        polluted_red < clean_red + 0.5,
        "expected a high-confidence probe that does not cover the current pixel to avoid inflating GI history reuse for the dominant probe, instead of leaking unrelated hierarchy confidence into the temporal blend; polluted_red={polluted_red:.2}, clean_red={clean_red:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_global_illumination_history_when_overlapping_dominant_probe_changes_by_resolve_weight(
) {
    let with_history =
        render_hybrid_gi_history_with_overlapping_dominant_probe_changed_by_resolve_weight(true);
    let without_history =
        render_hybrid_gi_history_with_overlapping_dominant_probe_changed_by_resolve_weight(false);
    let viewport_size = UVec2::new(with_history.width, with_history.height);

    let with_history_red =
        average_region_channel(&with_history.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let without_history_red = average_region_channel(
        &without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let with_history_blue =
        average_region_channel(&with_history.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let without_history_blue = average_region_channel(
        &without_history.rgba,
        viewport_size,
        2,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        without_history_blue > 1.0,
        "expected the second frame without history to be visibly driven by the new cool dominant overlapping probe so the resolve-weight history test is meaningful; without_history_blue={without_history_blue:.2}"
    );
    assert!(
        with_history_red < without_history_red + 0.5,
        "expected GI history to reject stale warm indirect light once an overlapping cool probe becomes the dominant contributor purely through higher hierarchy resolve weight, instead of keeping the previous probe's signature just because falloff and budget stayed tied; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
    assert!(
        with_history_blue > without_history_blue - 0.5,
        "expected GI history to stay aligned with the new cool dominant overlapping probe when resolve-weight dominance changes at identical screen support, instead of suppressing the current blue contribution with stale warm history; with_history_blue={with_history_blue:.2}, without_history_blue={without_history_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_rejects_global_illumination_history_when_overlapping_dominant_probe_changes_by_trace_support(
) {
    let with_history =
        render_hybrid_gi_history_with_overlapping_dominant_probe_changed_by_trace_support(true);
    let without_history =
        render_hybrid_gi_history_with_overlapping_dominant_probe_changed_by_trace_support(false);
    let viewport_size = UVec2::new(with_history.width, with_history.height);

    let with_history_red =
        average_region_channel(&with_history.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let without_history_red = average_region_channel(
        &without_history.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let with_history_blue =
        average_region_channel(&with_history.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let without_history_blue = average_region_channel(
        &without_history.rgba,
        viewport_size,
        2,
        0.25,
        0.75,
        0.25,
        0.75,
    );

    assert!(
        without_history_blue > 1.0,
        "expected the second frame without history to be visibly driven by the new cool dominant overlapping probe so the trace-support history test is meaningful; without_history_blue={without_history_blue:.2}"
    );
    assert!(
        with_history_red < without_history_red + 0.5,
        "expected GI history to reject stale warm indirect light once an overlapping cool probe becomes the dominant contributor through trace-support weighting, instead of keeping the previous probe's signature just because falloff and budget stayed tied; with_history_red={with_history_red:.2}, without_history_red={without_history_red:.2}"
    );
    assert!(
        with_history_blue > without_history_blue - 0.5,
        "expected GI history to stay aligned with the new cool dominant overlapping probe when trace-support dominance changes at identical falloff and budget, instead of suppressing the current blue contribution with stale warm history; with_history_blue={with_history_blue:.2}, without_history_blue={without_history_blue:.2}"
    );
}

fn build_extract(viewport_size: UVec2) -> RenderFrameExtract {
    build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(200, true, 128, Vec3::ZERO, 1.0),
            probe(500, true, 64, Vec3::ZERO, 1.0),
        ],
        vec![trace_region(40)],
    )
}

fn runtime_parent_topology<const N: usize>(
    edges: [(u32, u32); N],
) -> std::collections::BTreeMap<u32, u32> {
    edges.into_iter().collect()
}

fn build_extract_with_probes(
    viewport_size: UVec2,
    probes: Vec<RenderHybridGiProbe>,
) -> RenderFrameExtract {
    build_extract_with_probes_and_trace_regions(viewport_size, probes, vec![trace_region(40)])
}

fn build_extract_with_probes_and_trace_regions(
    viewport_size: UVec2,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
    snapshot.scene.directional_lights.clear();
    snapshot.preview.clear_color = crate::core::math::Vec4::ZERO;
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
        probe_budget: 2,
        tracing_budget: 1,
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

fn trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
    trace_region_with_bounds(region_id, crate::core::math::Vec3::ZERO, 1.0, 1.0)
}

fn trace_region_with_bounds(
    region_id: u32,
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_coverage: f32,
) -> RenderHybridGiTraceRegion {
    trace_region_with_bounds_and_rt_lighting(
        region_id,
        bounds_center,
        bounds_radius,
        screen_coverage,
        [0, 0, 0],
    )
}

fn trace_region_with_bounds_and_rt_lighting(
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

fn runtime_voxel_scene_prepare_from_tinted_mesh(
    tint_rgb: [f32; 3],
    translation: Vec3,
    uniform_scale: f32,
) -> HybridGiScenePrepareFrame {
    runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
        tint_rgb,
        translation,
        uniform_scale,
        &[],
        &[],
        &[],
    )
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
        &[crate::core::framework::render::RenderMeshSnapshot {
            node_id: 11,
            transform: crate::core::math::Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: crate::core::resource::ResourceHandle::<crate::core::resource::ModelMarker>::new(
                crate::core::resource::ResourceId::from_stable_label("res://models/card.obj"),
            ),
            material:
                crate::core::resource::ResourceHandle::<crate::core::resource::MaterialMarker>::new(
                    crate::core::resource::ResourceId::from_stable_label(
                        "res://materials/runtime-voxel-persisted-page.mat",
                    ),
                ),
            tint: crate::core::math::Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: crate::core::framework::scene::Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        &[],
        &[],
        &[],
    );
    let mut scene_prepare_resources =
        crate::hybrid_gi::renderer::HybridGiScenePrepareResourcesSnapshot::new(
            1,
            Vec::new(),
            vec![0],
            vec![0],
            0,
            0,
            (0, 0),
            (0, 0),
            0,
        );
    scene_prepare_resources.store_texture_slot_rgba_samples(
        vec![(0, persisted_capture_rgba)],
        vec![(0, persisted_capture_rgba)],
    );
    runtime.apply_scene_prepare_resources_for_test(&scene_prepare_resources);
    runtime.register_scene_extract(
        Some(&extract),
        &[crate::core::framework::render::RenderMeshSnapshot {
            node_id: 11,
            transform: crate::core::math::Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: crate::core::resource::ResourceHandle::<crate::core::resource::ModelMarker>::new(
                crate::core::resource::ResourceId::from_stable_label("res://models/card.obj"),
            ),
            material:
                crate::core::resource::ResourceHandle::<crate::core::resource::MaterialMarker>::new(
                    crate::core::resource::ResourceId::from_stable_label(
                        "res://materials/runtime-voxel-persisted-page.mat",
                    ),
                ),
            tint: crate::core::math::Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: crate::core::framework::scene::Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        &[],
        &[],
        &[],
    );

    runtime.build_scene_prepare_frame()
}

fn runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
    tint_rgb: [f32; 3],
    translation: Vec3,
    uniform_scale: f32,
    directional_lights: &[crate::core::framework::render::RenderDirectionalLightSnapshot],
    point_lights: &[crate::core::framework::render::RenderPointLightSnapshot],
    spot_lights: &[crate::core::framework::render::RenderSpotLightSnapshot],
) -> HybridGiScenePrepareFrame {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 0,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };

    runtime.register_scene_extract(
        Some(&extract),
        &[crate::core::framework::render::RenderMeshSnapshot {
            node_id: 11,
            transform: crate::core::math::Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: crate::core::resource::ResourceHandle::<crate::core::resource::ModelMarker>::new(
                crate::core::resource::ResourceId::from_stable_label("res://models/card.obj"),
            ),
            material:
                crate::core::resource::ResourceHandle::<crate::core::resource::MaterialMarker>::new(
                    crate::core::resource::ResourceId::from_stable_label(
                        "res://materials/runtime-voxel-tint.mat",
                    ),
                ),
            tint: crate::core::math::Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: crate::core::framework::scene::Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        directional_lights,
        point_lights,
        spot_lights,
    );

    runtime.build_scene_prepare_frame()
}

fn test_point_light(
    node_id: u64,
    position: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
) -> crate::core::framework::render::RenderPointLightSnapshot {
    crate::core::framework::render::RenderPointLightSnapshot {
        node_id,
        position,
        color,
        intensity,
        range,
    }
}

fn descendant_scene_driven_parent_irradiance_runtime_for_resolve(
    child_irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(100, false, 96, Vec3::ZERO, 1.4),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        trace_regions: vec![trace_region_with_bounds(
            40,
            Vec3::new(0.7, 0.0, 0.0),
            1.1,
            0.95,
        )],
    };

    runtime.register_extract(Some(&extract));
    runtime.ingest_plan(
        80,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.complete_gpu_updates([], [40], &[(200, child_irradiance_rgb)], &[], &[]);
    runtime.ingest_plan(
        81,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.build_resolve_runtime()
}

fn descendant_scene_driven_parent_irradiance_runtime_without_parent_exact_for_resolve(
    child_irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    let mut runtime =
        descendant_scene_driven_parent_irradiance_runtime_for_resolve(child_irradiance_rgb);
    runtime.remove_hierarchy_irradiance_for_test(100);
    runtime
}

fn descendant_scene_driven_parent_rt_runtime_for_resolve(
    child_rt_lighting_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(100, false, 96, Vec3::ZERO, 1.4),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        trace_regions: vec![trace_region_with_bounds(
            40,
            Vec3::new(0.7, 0.0, 0.0),
            1.1,
            0.95,
        )],
    };

    runtime.register_extract(Some(&extract));
    runtime.ingest_plan(
        82,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.complete_gpu_updates([], [40], &[], &[(200, child_rt_lighting_rgb)], &[]);
    runtime.ingest_plan(
        83,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.build_resolve_runtime()
}

fn descendant_scene_driven_parent_rt_runtime_without_parent_exact_for_resolve(
    child_rt_lighting_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    let mut runtime = descendant_scene_driven_parent_rt_runtime_for_resolve(child_rt_lighting_rgb);
    runtime.remove_hierarchy_rt_lighting_for_test(100);
    runtime
}

fn descendant_scene_driven_parent_resolve_runtime_without_parent_exact_for_resolve(
    include_requested_descendant_support: bool,
) -> HybridGiResolveRuntime {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(100, false, 96, Vec3::ZERO, 1.4),
            probe_with_parent(200, 100, false, 88, Vec3::new(0.7, 0.0, 0.0), 1.0),
        ],
        trace_regions: vec![trace_region_with_bounds(
            40,
            Vec3::new(0.7, 0.0, 0.0),
            1.1,
            0.95,
        )],
    };

    runtime.register_extract(Some(&extract));
    runtime.ingest_plan(
        84,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: include_requested_descendant_support
                .then_some(vec![200])
                .unwrap_or_default(),
            dirty_requested_probe_ids: include_requested_descendant_support
                .then_some(vec![200])
                .unwrap_or_default(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.ingest_plan(
        85,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut resolve_runtime = runtime.build_resolve_runtime();
    resolve_runtime.remove_hierarchy_resolve_weight_for_test(100);
    resolve_runtime
}

fn render_hybrid_gi_history_with_second_frame_resolve_weight(
    hierarchy_resolve_weight: f32,
) -> crate::types::ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract =
        build_extract_with_probes(viewport_size, vec![probe(200, true, 128, Vec3::ZERO, 1.1)]);
    let compiled = compile_hybrid_gi_pipeline(&extract);
    let history_handle = crate::FrameHistoryHandle::new(11);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 96, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [0, 0, 0],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (
                                200,
                                HybridGiResolveRuntime::pack_resolve_weight_q8(
                                    hierarchy_resolve_weight,
                                ),
                            ),
                        ]))
                        .build(),
                )),
            &compiled,
            Some(history_handle),
        )
        .unwrap()
}

fn render_hybrid_gi_history_with_changed_probe_identity(
    keep_history: bool,
) -> crate::types::ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let seed_extract =
        build_extract_with_probes(viewport_size, vec![probe(200, true, 128, Vec3::ZERO, 1.1)]);
    let changed_extract =
        build_extract_with_probes(viewport_size, vec![probe(500, true, 128, Vec3::ZERO, 1.1)]);
    let compiled = compile_hybrid_gi_pipeline(&seed_extract);
    let history_handle = crate::FrameHistoryHandle::new(12);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(seed_extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 96, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 500,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [0, 0, 0],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            keep_history.then_some(history_handle),
        )
        .unwrap()
}

fn render_hybrid_gi_history_with_changed_deeper_probe_lineage(
    keep_history: bool,
) -> crate::types::ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let seed_extract = build_extract_with_probes(
        viewport_size,
        vec![
            probe_with_parent(700, 500, true, 128, Vec3::ZERO, 1.1),
            probe_with_parent(500, 200, true, 128, Vec3::ZERO, 1.1),
            probe(200, true, 128, Vec3::ZERO, 1.1),
        ],
    );
    let changed_extract = build_extract_with_probes(
        viewport_size,
        vec![
            probe_with_parent(700, 500, true, 128, Vec3::ZERO, 1.1),
            probe_with_parent(500, 300, true, 128, Vec3::ZERO, 1.1),
            probe(300, true, 128, Vec3::ZERO, 1.1),
        ],
    );
    let compiled = compile_hybrid_gi_pipeline(&seed_extract);
    let history_handle = crate::FrameHistoryHandle::new(13);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(seed_extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 700,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 96, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(changed_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 700,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [0, 0, 0],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            keep_history.then_some(history_handle),
        )
        .unwrap()
}

fn render_hybrid_gi_history_with_non_dominant_confidence_pollution(
    include_remote_high_confidence_probe: bool,
) -> crate::types::ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let seed_extract =
        build_extract_with_probes(viewport_size, vec![probe(200, true, 128, Vec3::ZERO, 1.1)]);
    let mut second_frame_probes = vec![probe(200, true, 128, Vec3::ZERO, 1.1)];
    if include_remote_high_confidence_probe {
        second_frame_probes.push(probe(500, true, 128, Vec3::new(5.0, 0.0, 0.0), 0.25));
    }
    let second_extract = build_extract_with_probes(viewport_size, second_frame_probes);
    let compiled = compile_hybrid_gi_pipeline(&seed_extract);
    let history_handle = crate::FrameHistoryHandle::new(14);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(seed_extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 96, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    let mut probe_hierarchy_resolve_weight_q8 = std::collections::BTreeMap::from([(
        200,
        HybridGiResolveRuntime::pack_resolve_weight_q8(0.6),
    )]);
    if include_remote_high_confidence_probe {
        probe_hierarchy_resolve_weight_q8
            .insert(500, HybridGiResolveRuntime::pack_resolve_weight_q8(2.4));
    }

    let mut resident_probes = vec![HybridGiPrepareProbe {
        probe_id: 200,
        slot: 0,
        ray_budget: 128,
        irradiance_rgb: [0, 0, 0],
    }];
    if include_remote_high_confidence_probe {
        resident_probes.push(HybridGiPrepareProbe {
            probe_id: 500,
            slot: 1,
            ray_budget: 128,
            irradiance_rgb: [0, 0, 0],
        });
    }

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(second_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes,
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_hierarchy_resolve_weight_q8(probe_hierarchy_resolve_weight_q8)
                        .build(),
                )),
            &compiled,
            Some(history_handle),
        )
        .unwrap()
}

fn render_hybrid_gi_history_with_overlapping_dominant_probe_changed_by_resolve_weight(
    keep_history: bool,
) -> crate::types::ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let seed_extract =
        build_extract_with_probes(viewport_size, vec![probe(200, true, 128, Vec3::ZERO, 1.1)]);
    let second_extract = build_extract_with_probes(
        viewport_size,
        vec![
            probe(200, true, 128, Vec3::ZERO, 1.1),
            probe(500, true, 128, Vec3::ZERO, 1.1),
        ],
    );
    let compiled = compile_hybrid_gi_pipeline(&seed_extract);
    let history_handle = crate::FrameHistoryHandle::new(15);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(seed_extract, viewport_size).with_hybrid_gi_prepare(
                Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 96, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            Some(history_handle),
        )
        .unwrap();

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(second_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![
                        HybridGiPrepareProbe {
                            probe_id: 200,
                            slot: 0,
                            ray_budget: 128,
                            irradiance_rgb: [0, 0, 0],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 128,
                            irradiance_rgb: [48, 96, 255],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_resolve_runtime(Some(
                    HybridGiResolveRuntime::fixture()
                        .with_probe_hierarchy_resolve_weight_q8(std::collections::BTreeMap::from([
                            (200, HybridGiResolveRuntime::pack_resolve_weight_q8(1.0)),
                            (500, HybridGiResolveRuntime::pack_resolve_weight_q8(2.4)),
                        ]))
                        .build(),
                )),
            &compiled,
            keep_history.then_some(history_handle),
        )
        .unwrap()
}

fn render_hybrid_gi_history_with_overlapping_dominant_probe_changed_by_trace_support(
    keep_history: bool,
) -> crate::types::ViewportFrame {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(200, true, 128, Vec3::new(-0.22, 0.0, 0.0), 1.05),
            probe(500, true, 128, Vec3::new(0.22, 0.0, 0.0), 1.05),
        ],
        vec![trace_region_with_bounds(
            40,
            Vec3::new(0.22, 0.0, 0.0),
            0.8,
            0.95,
        )],
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);
    let history_handle = crate::FrameHistoryHandle::new(18);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 128,
                        irradiance_rgb: [255, 96, 48],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: Vec::new(),
                    evictable_probe_ids: Vec::new(),
                })),
            &compiled,
            Some(history_handle),
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
                            ray_budget: 128,
                            irradiance_rgb: [255, 96, 48],
                        },
                        HybridGiPrepareProbe {
                            probe_id: 500,
                            slot: 1,
                            ray_budget: 128,
                            irradiance_rgb: [48, 96, 255],
                        },
                    ],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                }),
            ),
            &compiled,
            keep_history.then_some(history_handle),
        )
        .unwrap()
}

fn average_region_luma(
    rgba: &[u8],
    viewport_size: UVec2,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
) -> f32 {
    let red = average_region_channel(rgba, viewport_size, 0, x_min, x_max, y_min, y_max);
    let green = average_region_channel(rgba, viewport_size, 1, x_min, x_max, y_min, y_max);
    let blue = average_region_channel(rgba, viewport_size, 2, x_min, x_max, y_min, y_max);
    (red + green + blue) / 3.0
}

fn average_region_channel(
    rgba: &[u8],
    viewport_size: UVec2,
    channel: usize,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let start_x = ((width as f32) * x_min.clamp(0.0, 1.0)).floor() as usize;
    let end_x = ((width as f32) * x_max.clamp(0.0, 1.0)).ceil() as usize;
    let start_y = ((height as f32) * y_min.clamp(0.0, 1.0)).floor() as usize;
    let end_y = ((height as f32) * y_max.clamp(0.0, 1.0)).ceil() as usize;

    let mut total = 0.0;
    let mut count = 0usize;
    for y in start_y.min(height)..end_y.min(height).max(start_y.min(height) + 1) {
        for x in start_x.min(width)..end_x.min(width).max(start_x.min(width) + 1) {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }
    total / count as f32
}

#[derive(Clone, Copy)]
enum Half {
    Left,
    Right,
}

fn average_half_channel(rgba: &[u8], viewport_size: UVec2, channel: usize, half: Half) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let x_range = match half {
        Half::Left => 0..(width / 2).max(1),
        Half::Right => (width / 2)..width,
    };

    let mut total = 0.0;
    let mut count = 0usize;
    for y in 0..height {
        for x in x_range.clone() {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }
    total / count as f32
}

fn average_half_channel_delta(
    baseline_rgba: &[u8],
    resolved_rgba: &[u8],
    viewport_size: UVec2,
    channel: usize,
    half: Half,
) -> f32 {
    average_half_channel(resolved_rgba, viewport_size, channel, half)
        - average_half_channel(baseline_rgba, viewport_size, channel, half)
}

fn average_half_luma(rgba: &[u8], viewport_size: UVec2, half: Half) -> f32 {
    let red = average_half_channel(rgba, viewport_size, 0, half);
    let green = average_half_channel(rgba, viewport_size, 1, half);
    let blue = average_half_channel(rgba, viewport_size, 2, half);
    (red + green + blue) / 3.0
}

fn average_half_luma_delta(
    baseline_rgba: &[u8],
    resolved_rgba: &[u8],
    viewport_size: UVec2,
    half: Half,
) -> f32 {
    average_half_luma(resolved_rgba, viewport_size, half)
        - average_half_luma(baseline_rgba, viewport_size, half)
}
