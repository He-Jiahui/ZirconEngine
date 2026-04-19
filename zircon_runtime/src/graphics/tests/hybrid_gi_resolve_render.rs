use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::math::{UVec2, Vec3};
use crate::scene::world::World;

use crate::{
    runtime::HybridGiRuntimeState,
    types::{
        EditorOrRuntimeFrame, HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiResolveRuntime,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
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
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size),
            &compiled,
            None,
        )
        .unwrap();
    let resolved = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
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
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes(
        viewport_size,
        vec![probe(200, true, 128, Vec3::new(-0.9, 0.0, 0.0), 2.0)],
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

    let baseline = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size),
            &compiled,
            None,
        )
        .unwrap();
    let resolved = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
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
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
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

    let left_supported = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size).with_hybrid_gi_prepare(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();
    let compiled = RenderPipelineAsset::default_forward_plus()
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

    let flat = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(
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
            &EditorOrRuntimeFrame::from_extract(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let probe = probe(200, true, 128, Vec3::ZERO, 2.6);
    let compiled = RenderPipelineAsset::default_forward_plus()
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
            &EditorOrRuntimeFrame::from_extract(
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
            &EditorOrRuntimeFrame::from_extract(
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
fn hybrid_gi_resolve_changes_when_resident_ancestor_is_reached_through_nonresident_hierarchy_gap() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();
    let compiled = RenderPipelineAsset::default_forward_plus()
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

    let flat = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(
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
            &EditorOrRuntimeFrame::from_extract(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();
    let compiled = RenderPipelineAsset::default_forward_plus()
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

    let flat = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(
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
            &EditorOrRuntimeFrame::from_extract(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = Vec::new();

    let compiled = RenderPipelineAsset::default_forward_plus()
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

    let low_budget = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(
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
            &EditorOrRuntimeFrame::from_extract(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = vec![trace_region_with_bounds_and_rt_lighting(
        40,
        Vec3::new(-0.85, 0.0, 0.0),
        0.75,
        0.95,
        [255, 64, 32],
    )];

    let compiled = RenderPipelineAsset::default_forward_plus()
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

    let low_budget = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(
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
            &EditorOrRuntimeFrame::from_extract(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let trace_regions = vec![trace_region_with_bounds_and_rt_lighting(
        40,
        Vec3::new(-0.85, 0.0, 0.0),
        0.75,
        0.95,
        [255, 64, 32],
    )];
    let compiled = RenderPipelineAsset::default_forward_plus()
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

    let flat = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(
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
            &EditorOrRuntimeFrame::from_extract(
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
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
            irradiance_rgb: [160, 160, 160],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                    probe_rt_lighting_rgb: std::collections::BTreeMap::from([(200, [240, 96, 48])]),
                    ..Default::default()
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                    probe_rt_lighting_rgb: std::collections::BTreeMap::from([(200, [48, 96, 240])]),
                    ..Default::default()
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
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

    let warm = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                    probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                        200,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                    )]),
                    probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([
                        (
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.95, 0.28, 0.12], 0.55),
                        ),
                    ]),
                    ..Default::default()
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                    probe_hierarchy_resolve_weight_q8: std::collections::BTreeMap::from([(
                        200,
                        HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                    )]),
                    probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([
                        (
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.28, 0.95], 0.55),
                        ),
                    ]),
                    ..Default::default()
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![
            probe(100, false, 96, Vec3::ZERO, 1.8),
            probe_with_parent(200, 100, true, 128, Vec3::ZERO, 1.8),
        ],
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
            irradiance_rgb: [128, 128, 128],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                    probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([
                        (
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.95, 0.3, 0.12], 0.5),
                        ),
                    ]),
                    ..Default::default()
                })),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                    probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([
                        (
                            200,
                            HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.3, 0.95], 0.5),
                        ),
                    ]),
                    ..Default::default()
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
        "expected runtime-host GI resolve to consume hierarchy RT-lighting continuation from runtime/GPU history even when the current frame has no scheduled trace regions; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected runtime-host GI resolve to consume hierarchy RT-lighting continuation from runtime/GPU history even when the current frame has no scheduled trace regions; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_descendant_scene_driven_runtime_irradiance_for_parent_probe_after_schedule_clears(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(100, true, 96, Vec3::ZERO, 1.8)],
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
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(warm_runtime)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(100, true, 96, Vec3::ZERO, 1.8)],
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
            &EditorOrRuntimeFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_resolve_runtime(Some(warm_runtime)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
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
    snapshot.scene.lights.clear();
    snapshot.preview.clear_color = crate::core::math::Vec4::ZERO;
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
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

fn descendant_scene_driven_parent_irradiance_runtime_for_resolve(
    child_irradiance_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
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

fn descendant_scene_driven_parent_rt_runtime_for_resolve(
    child_rt_lighting_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
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
