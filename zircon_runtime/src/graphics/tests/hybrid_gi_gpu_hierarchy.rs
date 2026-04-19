use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::math::{UVec2, Vec3};
use crate::scene::world::World;

use crate::{
    types::{
        EditorOrRuntimeFrame, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareUpdateRequest,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

#[test]
fn hybrid_gi_gpu_completion_readback_prefers_resident_ancestor_radiance_through_nonresident_hierarchy_gap(
) {
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
            generation: 35,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let warm_ancestor = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                probe_with_parent(250, 200, false, 96, Vec3::new(-0.1, 0.0, 0.0), 0.85),
                probe_with_parent(300, 250, false, 128, Vec3::ZERO, 0.85),
                probe(500, true, 96, Vec3::new(0.2, 0.0, 0.0), 0.85),
            ],
            trace_regions.clone(),
        ),
        prepare.clone(),
    );
    let cool_ancestor = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(-0.2, 0.0, 0.0), 0.85),
                probe(500, true, 96, Vec3::new(0.2, 0.0, 0.0), 0.85),
                probe_with_parent(550, 500, false, 96, Vec3::new(0.1, 0.0, 0.0), 0.85),
                probe_with_parent(300, 550, false, 128, Vec3::ZERO, 0.85),
            ],
            trace_regions,
        ),
        prepare,
    );
    let warm_ancestor_rgb = warm_ancestor
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm-ancestor pending probe irradiance");
    let cool_ancestor_rgb = cool_ancestor
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool-ancestor pending probe irradiance");

    assert!(
        warm_ancestor_rgb[0] > cool_ancestor_rgb[0] + 12,
        "expected a pending probe to inherit more red radiance when its nearest resident ancestor is the warm probe across a nonresident hierarchy gap; warm_ancestor={warm_ancestor_rgb:?}, cool_ancestor={cool_ancestor_rgb:?}"
    );
    assert!(
        cool_ancestor_rgb[2] > warm_ancestor_rgb[2] + 12,
        "expected a pending probe to inherit more blue radiance when its nearest resident ancestor is the cool probe across a nonresident hierarchy gap; warm_ancestor={warm_ancestor_rgb:?}, cool_ancestor={cool_ancestor_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_inherits_ancestor_trace_rt_lighting_through_nonresident_hierarchy_gap(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
            HybridGiPrepareProbe {
                probe_id: 500,
                slot: 1,
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 36,
        }],
        scheduled_trace_region_ids: vec![40, 50],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.8),
                probe(300, false, 128, Vec3::ZERO, 0.2),
                probe(500, true, 96, Vec3::new(2.5, 0.0, 0.0), 0.8),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.2, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-0.85, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            3,
            1,
            vec![
                probe(200, true, 96, Vec3::new(-0.85, 0.0, 0.0), 0.8),
                probe_with_parent(250, 200, false, 96, Vec3::new(-0.25, 0.0, 0.0), 0.85),
                probe_with_parent(300, 250, false, 128, Vec3::ZERO, 0.2),
                probe(500, true, 96, Vec3::new(2.5, 0.0, 0.0), 0.8),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.2, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-0.85, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare,
    );
    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe irradiance");

    assert!(
        hierarchical_rgb[0] > flat_rgb[0] + 8,
        "expected pending probe GPU completion to inherit warmer RT-lighting tint from an ancestor-covered trace region across a nonresident hierarchy gap while the flat hierarchy stays closer to the local neutral trace result; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_blends_farther_resident_ancestor_radiance_beyond_nearest_resident_parent(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [255, 64, 32],
            },
            HybridGiPrepareProbe {
                probe_id: 200,
                slot: 1,
                ray_budget: 96,
                irradiance_rgb: [40, 96, 255],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 500,
            ray_budget: 128,
            generation: 37,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            4,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-0.18, 0.0, 0.0), 1.0),
                probe(200, true, 96, Vec3::new(-0.06, 0.0, 0.0), 1.0),
                probe(500, false, 128, Vec3::ZERO, 1.0),
            ],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                1.0,
                0.95,
                [144, 144, 144],
            )],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            4,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-0.18, 0.0, 0.0), 1.0),
                probe_with_parent(200, 100, true, 96, Vec3::new(-0.06, 0.0, 0.0), 1.0),
                probe_with_parent(300, 200, false, 96, Vec3::new(-0.02, 0.0, 0.0), 1.0),
                probe_with_parent(500, 300, false, 128, Vec3::ZERO, 1.0),
            ],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                1.0,
                0.95,
                [144, 144, 144],
            )],
        ),
        prepare,
    );
    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 500)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 500)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe irradiance");
    let flat_chroma = i16::from(flat_rgb[0]) - i16::from(flat_rgb[2]);
    let hierarchical_chroma = i16::from(hierarchical_rgb[0]) - i16::from(hierarchical_rgb[2]);

    assert!(
        hierarchical_chroma > flat_chroma + 12,
        "expected pending probe radiance gather to pull additional warm energy from a farther resident ancestor instead of only favoring the nearest resident parent; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_inherits_farther_resident_ancestor_trace_rt_lighting_beyond_nearest_resident_parent(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
            HybridGiPrepareProbe {
                probe_id: 200,
                slot: 1,
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 500,
            ray_budget: 128,
            generation: 38,
        }],
        scheduled_trace_region_ids: vec![40, 50],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            4,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-1.2, 0.0, 0.0), 0.2),
                probe(200, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe(500, false, 128, Vec3::ZERO, 0.25),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.25, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.2, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            4,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-1.2, 0.0, 0.0), 0.2),
                probe_with_parent(200, 100, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe_with_parent(300, 200, false, 96, Vec3::new(-0.04, 0.0, 0.0), 0.8),
                probe_with_parent(500, 300, false, 128, Vec3::ZERO, 0.25),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.25, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.2, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare,
    );
    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 500)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 500)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe irradiance");

    assert!(
        hierarchical_rgb[0] > flat_rgb[0] + 8,
        "expected pending probe RT-lighting continuation to reach a farther resident ancestor when the nearest resident parent only sees the local neutral trace; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_inherits_third_resident_ancestor_trace_rt_lighting() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
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
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 700,
            ray_budget: 128,
            generation: 41,
        }],
        scheduled_trace_region_ids: vec![40, 50],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            5,
            2,
            vec![
                probe(100, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.18),
                probe(200, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe(300, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe(700, false, 128, Vec3::ZERO, 0.25),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.25, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.4, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            5,
            2,
            vec![
                probe(100, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.18),
                probe_with_parent(200, 100, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe_with_parent(300, 200, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe_with_parent(500, 300, false, 96, Vec3::new(-0.03, 0.0, 0.0), 0.8),
                probe_with_parent(700, 500, false, 128, Vec3::ZERO, 0.25),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.25, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.4, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare,
    );
    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 700)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 700)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe irradiance");

    assert!(
        hierarchical_rgb[0] > flat_rgb[0] + 8,
        "expected pending probe RT-lighting continuation to reach a third resident ancestor when the nearer resident ancestors only see the local neutral trace; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_gathers_third_resident_ancestor_radiance() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [255, 64, 32],
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
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 700,
            ray_budget: 128,
            generation: 42,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            5,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.18),
                probe(200, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe(300, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe(700, false, 128, Vec3::ZERO, 0.12),
            ],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                0.12,
                0.9,
                [144, 144, 144],
            )],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            5,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.18),
                probe_with_parent(200, 100, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe_with_parent(300, 200, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe_with_parent(500, 300, false, 96, Vec3::new(-0.03, 0.0, 0.0), 0.8),
                probe_with_parent(700, 500, false, 128, Vec3::ZERO, 0.12),
            ],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                0.12,
                0.9,
                [144, 144, 144],
            )],
        ),
        prepare,
    );
    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 700)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 700)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe irradiance");
    let flat_chroma = i16::from(flat_rgb[0]) - i16::from(flat_rgb[2]);
    let hierarchical_chroma = i16::from(hierarchical_rgb[0]) - i16::from(hierarchical_rgb[2]);

    assert!(
        hierarchical_chroma > flat_chroma + 10,
        "expected pending probe lineage gather to reach a third resident ancestor when the nearer resident ancestors stay neutral; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_inherits_fourth_resident_ancestor_trace_rt_lighting() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
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
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
            HybridGiPrepareProbe {
                probe_id: 400,
                slot: 3,
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 900,
            ray_budget: 128,
            generation: 43,
        }],
        scheduled_trace_region_ids: vec![40, 50],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            6,
            2,
            vec![
                probe(100, true, 96, Vec3::new(-1.8, 0.0, 0.0), 0.18),
                probe(200, true, 96, Vec3::new(-1.05, 0.0, 0.0), 0.18),
                probe(300, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe(400, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe(900, false, 128, Vec3::ZERO, 0.25),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.25, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.8, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            6,
            2,
            vec![
                probe(100, true, 96, Vec3::new(-1.8, 0.0, 0.0), 0.18),
                probe_with_parent(200, 100, true, 96, Vec3::new(-1.05, 0.0, 0.0), 0.18),
                probe_with_parent(300, 200, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe_with_parent(400, 300, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe_with_parent(700, 400, false, 96, Vec3::new(-0.03, 0.0, 0.0), 0.8),
                probe_with_parent(900, 700, false, 128, Vec3::ZERO, 0.25),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.25, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.8, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare,
    );
    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 900)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 900)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe irradiance");

    assert!(
        hierarchical_rgb[0] > flat_rgb[0] + 8,
        "expected pending probe RT-lighting continuation to reach a fourth resident ancestor when the nearer resident ancestors only see the local neutral trace; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_gathers_fourth_resident_ancestor_radiance() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![
            HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [255, 64, 32],
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
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
            HybridGiPrepareProbe {
                probe_id: 400,
                slot: 3,
                ray_budget: 96,
                irradiance_rgb: [160, 160, 160],
            },
        ],
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 900,
            ray_budget: 128,
            generation: 44,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            6,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-1.8, 0.0, 0.0), 0.18),
                probe(200, true, 96, Vec3::new(-1.05, 0.0, 0.0), 0.18),
                probe(300, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe(400, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe(900, false, 128, Vec3::ZERO, 0.12),
            ],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                0.12,
                0.9,
                [144, 144, 144],
            )],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            6,
            1,
            vec![
                probe(100, true, 96, Vec3::new(-1.8, 0.0, 0.0), 0.18),
                probe_with_parent(200, 100, true, 96, Vec3::new(-1.05, 0.0, 0.0), 0.18),
                probe_with_parent(300, 200, true, 96, Vec3::new(-0.55, 0.0, 0.0), 0.18),
                probe_with_parent(400, 300, true, 96, Vec3::new(-0.1, 0.0, 0.0), 0.18),
                probe_with_parent(700, 400, false, 96, Vec3::new(-0.03, 0.0, 0.0), 0.8),
                probe_with_parent(900, 700, false, 128, Vec3::ZERO, 0.12),
            ],
            vec![trace_region_with_rt_lighting(
                40,
                Vec3::ZERO,
                0.12,
                0.9,
                [144, 144, 144],
            )],
        ),
        prepare,
    );
    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 900)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 900)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe irradiance");
    let flat_chroma = i16::from(flat_rgb[0]) - i16::from(flat_rgb[2]);
    let hierarchical_chroma = i16::from(hierarchical_rgb[0]) - i16::from(hierarchical_rgb[2]);

    assert!(
        hierarchical_chroma > flat_chroma + 8,
        "expected pending probe lineage gather to reach a fourth resident ancestor when the nearer resident ancestors stay neutral; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_inherits_primary_resident_ancestor_radiance_without_spatial_overlap(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        2,
        1,
        vec![
            probe(200, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.2),
            probe_with_parent(250, 200, false, 96, Vec3::new(-0.45, 0.0, 0.0), 0.2),
            probe_with_parent(300, 250, false, 128, Vec3::ZERO, 0.2),
        ],
        vec![trace_region_with_rt_lighting(
            40,
            Vec3::ZERO,
            0.2,
            0.9,
            [144, 144, 144],
        )],
    );

    let warm_ancestor = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        extract.clone(),
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [255, 64, 32],
            }],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 128,
                generation: 39,
            }],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    let cool_ancestor = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        extract,
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 200,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [32, 96, 255],
            }],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 128,
                generation: 40,
            }],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let warm_ancestor_rgb = warm_ancestor
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm primary ancestor pending probe irradiance");
    let cool_ancestor_rgb = cool_ancestor
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool primary ancestor pending probe irradiance");

    assert!(
        warm_ancestor_rgb[0] > cool_ancestor_rgb[0] + 12,
        "expected hierarchy-only gather to inherit more red from the warm primary resident ancestor even when the ancestor is too far for direct spatial overlap; warm={warm_ancestor_rgb:?}, cool={cool_ancestor_rgb:?}"
    );
    assert!(
        cool_ancestor_rgb[2] > warm_ancestor_rgb[2] + 12,
        "expected hierarchy-only gather to inherit more blue from the cool primary resident ancestor even when the ancestor is too far for direct spatial overlap; warm={warm_ancestor_rgb:?}, cool={cool_ancestor_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_updates_resident_probe_from_hierarchy_only_resident_ancestor_radiance(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let hierarchical_extract = build_extract(
        viewport_size,
        2,
        1,
        vec![
            probe(200, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.2),
            probe_with_parent(250, 200, false, 96, Vec3::new(-0.45, 0.0, 0.0), 0.2),
            probe_with_parent(300, 250, true, 128, Vec3::ZERO, 0.2),
        ],
        vec![trace_region_with_rt_lighting(
            40,
            Vec3::ZERO,
            0.2,
            0.9,
            [144, 144, 144],
        )],
    );

    let warm_ancestor = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        hierarchical_extract.clone(),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 96,
                    irradiance_rgb: [255, 64, 32],
                },
                HybridGiPrepareProbe {
                    probe_id: 300,
                    slot: 1,
                    ray_budget: 128,
                    irradiance_rgb: [96, 96, 96],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    let cool_ancestor = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        hierarchical_extract,
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 96,
                    irradiance_rgb: [32, 96, 255],
                },
                HybridGiPrepareProbe {
                    probe_id: 300,
                    slot: 1,
                    ray_budget: 128,
                    irradiance_rgb: [96, 96, 96],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let warm_rgb = warm_ancestor
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm resident child probe irradiance");
    let cool_rgb = cool_ancestor
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool resident child probe irradiance");

    assert!(
        warm_rgb[0] > cool_rgb[0] + 12,
        "expected resident probe GPU completion to gather more red from a warm resident ancestor across a nonresident hierarchy gap instead of only using flat local gather; warm={warm_rgb:?}, cool={cool_rgb:?}"
    );
    assert!(
        cool_rgb[2] > warm_rgb[2] + 12,
        "expected resident probe GPU completion to gather more blue from a cool resident ancestor across a nonresident hierarchy gap instead of only using flat local gather; warm={warm_rgb:?}, cool={cool_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_updates_resident_probe_with_ancestor_rt_lighting_continuation()
{
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
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
        scheduled_trace_region_ids: vec![40, 50],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            2,
            2,
            vec![
                probe(200, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.2),
                probe(300, true, 128, Vec3::ZERO, 0.2),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.2, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.4, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            2,
            2,
            vec![
                probe(200, true, 96, Vec3::new(-1.4, 0.0, 0.0), 0.2),
                probe_with_parent(250, 200, false, 96, Vec3::new(-0.45, 0.0, 0.0), 0.2),
                probe_with_parent(300, 250, true, 128, Vec3::ZERO, 0.2),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.2, 0.9, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-1.4, 0.0, 0.0),
                    0.05,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare,
    );

    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("flat resident child probe irradiance");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical resident child probe irradiance");

    assert!(
        hierarchical_rgb[0] > flat_rgb[0] + 8,
        "expected resident probe GPU completion to inherit warmer ancestor RT-lighting tint across a nonresident hierarchy gap instead of staying on the flat local trace result; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

#[test]
fn hybrid_gi_gpu_completion_readback_inherits_nonresident_lineage_trace_lighting_without_resident_ancestors(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 46,
        }],
        scheduled_trace_region_ids: vec![40, 50],
        evictable_probe_ids: Vec::new(),
    };

    let flat = render_hybrid_gi_gpu_trace_lighting_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            1,
            2,
            vec![probe(300, false, 128, Vec3::ZERO, 0.18)],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.2, 0.95, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-0.9, 0.0, 0.0),
                    0.06,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare.clone(),
    );
    let hierarchical = render_hybrid_gi_gpu_trace_lighting_readback(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            1,
            2,
            vec![
                probe_with_parent(250, 100, false, 96, Vec3::new(-0.9, 0.0, 0.0), 0.2),
                probe_with_parent(300, 250, false, 128, Vec3::ZERO, 0.18),
            ],
            vec![
                trace_region_with_rt_lighting(40, Vec3::ZERO, 0.2, 0.95, [144, 144, 144]),
                trace_region_with_rt_lighting(
                    50,
                    Vec3::new(-0.9, 0.0, 0.0),
                    0.06,
                    0.95,
                    [255, 64, 32],
                ),
            ],
        ),
        prepare,
    );

    let flat_rgb = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending probe trace lighting");
    let hierarchical_rgb = hierarchical
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("hierarchical pending probe trace lighting");

    assert!(
        hierarchical_rgb[0] > flat_rgb[0] + 8,
        "expected pending probe GPU trace-lighting readback to inherit warm tint from a nonresident hierarchy chain even when no resident ancestor is available yet; flat={flat_rgb:?}, hierarchical={hierarchical_rgb:?}"
    );
}

fn render_hybrid_gi_gpu_readback(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
) -> Vec<(u32, [u8; 3])> {
    let (probe_irradiance_rgb, _) =
        render_hybrid_gi_gpu_trace_readback(renderer, viewport_size, extract, prepare);
    probe_irradiance_rgb
}

fn render_hybrid_gi_gpu_trace_lighting_readback(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
) -> Vec<(u32, [u8; 3])> {
    let (_, probe_trace_lighting_rgb) =
        render_hybrid_gi_gpu_trace_readback(renderer, viewport_size, extract, prepare);
    probe_trace_lighting_rgb
}

fn render_hybrid_gi_gpu_trace_readback(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
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
            &EditorOrRuntimeFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare)),
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

fn build_extract(
    viewport_size: UVec2,
    probe_budget: u32,
    tracing_budget: u32,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
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
