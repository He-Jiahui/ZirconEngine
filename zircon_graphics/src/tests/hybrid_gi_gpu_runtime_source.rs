use std::sync::Arc;

use zircon_asset::pipeline::manager::ProjectAssetManager;
use zircon_framework::render::{
    RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    RenderSceneSnapshot, RenderWorldSnapshotHandle,
};
use zircon_math::{UVec2, Vec3};
use zircon_scene::world::World;

use crate::{
    runtime::HybridGiRuntimeState,
    types::{
        EditorOrRuntimeFrame, HybridGiPrepareFrame, HybridGiPrepareUpdateRequest,
        HybridGiResolveRuntime,
    },
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

#[test]
fn hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_hierarchy_rt_continuation_after_schedule_clears(
) {
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
            false,
            128,
            Vec3::new(0.0, 0.0, 0.0),
            0.85,
        )],
        vec![trace_region(40, Vec3::new(0.0, 0.0, 0.0), 0.8, 0.9)],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 14,
        }],
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
        .expect("warm pending trace-lighting probe");
    let cool_trace = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("cool pending trace-lighting probe");

    assert!(
        u16::from(warm_trace[0]) > u16::from(cool_trace[0]) + 20,
        "expected pending probe GPU prepare to keep consuming warm hierarchy RT-lighting continuation from runtime after the current trace schedule clears, instead of collapsing to the same flat source; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
    assert!(
        u16::from(cool_trace[2]) > u16::from(warm_trace[2]) + 20,
        "expected pending probe GPU prepare to keep consuming cool hierarchy RT-lighting continuation from runtime after the current trace schedule clears, instead of collapsing to the same flat source; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_direct_rt_history_when_hierarchy_weight_is_flat(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![probe(200, false, 128, Vec3::new(0.0, 0.0, 0.0), 0.85)],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 15,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(direct_runtime_trace_history_for_gpu_source([240, 96, 48], &extract)),
    );
    let cool = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare,
        Some(direct_runtime_trace_history_for_gpu_source([48, 96, 240], &extract)),
    );

    let warm_trace = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("warm pending direct-runtime trace-lighting probe");
    let cool_trace = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("cool pending direct-runtime trace-lighting probe");

    assert!(
        u16::from(warm_trace[0]) > u16::from(cool_trace[0]) + 20,
        "expected runtime-host direct RT history from build_resolve_runtime() to keep influencing GPU prepare even when the hierarchy resolve weight remains flat, instead of collapsing both pending-probe paths back to the same black/no-source trace-lighting result; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
    assert!(
        u16::from(cool_trace[2]) > u16::from(warm_trace[2]) + 20,
        "expected runtime-host direct RT history from build_resolve_runtime() to keep influencing GPU prepare even when the hierarchy resolve weight remains flat, instead of collapsing both pending-probe paths back to the same black/no-source trace-lighting result; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_irradiance_uses_runtime_hierarchy_source_when_scene_gather_is_missing(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![probe_with_parent(
            200,
            100,
            false,
            128,
            Vec3::new(0.0, 0.0, 0.0),
            0.85,
        )],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 19,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(HybridGiResolveRuntime {
            probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                200,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.34, 0.12], 0.62),
            )]),
            ..Default::default()
        }),
    );
    let cool = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(HybridGiResolveRuntime {
            probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                200,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.34, 0.92], 0.62),
            )]),
            ..Default::default()
        }),
    );

    let warm_irradiance = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("warm pending irradiance probe");
    let cool_irradiance = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("cool pending irradiance probe");

    assert!(
        u16::from(warm_irradiance[0]) > u16::from(cool_irradiance[0]) + 20,
        "expected pending probe GPU prepare to keep consuming warm runtime hierarchy irradiance when current-frame resident gather is unavailable, instead of collapsing both paths to the same black/no-gather source; warm_irradiance={warm_irradiance:?}, cool_irradiance={cool_irradiance:?}"
    );
    assert!(
        u16::from(cool_irradiance[2]) > u16::from(warm_irradiance[2]) + 20,
        "expected pending probe GPU prepare to keep consuming cool runtime hierarchy irradiance when current-frame resident gather is unavailable, instead of collapsing both paths to the same black/no-gather source; warm_irradiance={warm_irradiance:?}, cool_irradiance={cool_irradiance:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_trace_lighting_uses_requested_lineage_runtime_source_without_trace_schedule(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![
            probe(100, false, 64, Vec3::new(-0.4, 0.0, 0.0), 0.75),
            probe_with_parent(
                200,
                100,
                false,
                128,
                Vec3::new(0.0, 0.0, 0.0),
                0.85,
            ),
        ],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 24,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let requested_runtime = requested_lineage_runtime_for_gpu_source(true, &extract);
    let flat_runtime = requested_lineage_runtime_for_gpu_source(false, &extract);
    let requested = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(requested_runtime),
    );
    let flat = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(flat_runtime),
    );

    let requested_trace = requested
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("requested pending trace-lighting probe");
    let flat_trace = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending trace-lighting probe");

    assert!(
        u16::from(requested_trace[0]) > u16::from(flat_trace[0]) + 40,
        "expected requested screen-probe lineage support to keep runtime/GPU trace-lighting continuation alive even with no current trace schedule, instead of letting the pending probe collapse back to the same flat black source; flat_trace={flat_trace:?}, requested_trace={requested_trace:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_irradiance_uses_requested_lineage_runtime_source_without_trace_schedule(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![
            probe(100, false, 64, Vec3::new(-0.4, 0.0, 0.0), 0.75),
            probe_with_parent(
                200,
                100,
                false,
                128,
                Vec3::new(0.0, 0.0, 0.0),
                0.85,
            ),
        ],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 25,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let requested_runtime = requested_lineage_irradiance_runtime_for_gpu_source(true, &extract);
    let flat_runtime = requested_lineage_irradiance_runtime_for_gpu_source(false, &extract);
    let requested = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(requested_runtime),
    );
    let flat = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(flat_runtime),
    );

    let requested_irradiance = requested
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("requested pending irradiance probe");
    let flat_irradiance = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending irradiance probe");

    assert!(
        u16::from(requested_irradiance[0]) > u16::from(flat_irradiance[0]) + 30,
        "expected requested screen-probe lineage support to keep runtime/GPU irradiance continuation alive even with no current trace schedule, instead of letting the pending probe collapse back to the same flat black source; flat_irradiance={flat_irradiance:?}, requested_irradiance={requested_irradiance:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_irradiance_inherits_requested_nonresident_ancestor_runtime_source_without_trace_schedule(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![
            probe(100, false, 64, Vec3::new(-0.4, 0.0, 0.0), 0.75),
            probe_with_parent(
                200,
                100,
                false,
                128,
                Vec3::new(0.0, 0.0, 0.0),
                0.85,
            ),
        ],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 26,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let requested_runtime =
        requested_lineage_nonresident_ancestor_irradiance_runtime_for_gpu_source(true, &extract);
    let flat_runtime =
        requested_lineage_nonresident_ancestor_irradiance_runtime_for_gpu_source(false, &extract);
    let requested = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(requested_runtime),
    );
    let flat = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(flat_runtime),
    );

    let requested_irradiance = requested
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("requested pending irradiance probe");
    let flat_irradiance = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending irradiance probe");

    assert!(
        u16::from(requested_irradiance[0]) > u16::from(flat_irradiance[0]) + 24,
        "expected requested lineage support to let a pending child probe inherit runtime irradiance from a nonresident ancestor even with no current trace schedule, instead of collapsing both paths back to the same flat source; flat_irradiance={flat_irradiance:?}, requested_irradiance={requested_irradiance:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_trace_lighting_inherits_requested_nonresident_ancestor_runtime_source_without_trace_schedule(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![
            probe(100, false, 64, Vec3::new(-0.4, 0.0, 0.0), 0.75),
            probe_with_parent(
                200,
                100,
                false,
                128,
                Vec3::new(0.0, 0.0, 0.0),
                0.85,
            ),
        ],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 27,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let requested_runtime =
        requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source(true, &extract);
    let flat_runtime =
        requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source(false, &extract);
    let requested = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(requested_runtime),
    );
    let flat = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(flat_runtime),
    );

    let requested_trace = requested
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("requested pending trace-lighting probe");
    let flat_trace = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending trace-lighting probe");

    assert!(
        u16::from(requested_trace[0]) > u16::from(flat_trace[0]) + 24,
        "expected requested lineage support to let a pending child probe inherit runtime RT lighting from a nonresident ancestor even with no current trace schedule, instead of collapsing both paths back to the same flat source; flat_trace={flat_trace:?}, requested_trace={requested_trace:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_trace_lighting_keeps_recent_requested_lineage_runtime_source_after_request_clears(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![
            probe(100, false, 64, Vec3::new(-0.4, 0.0, 0.0), 0.75),
            probe_with_parent(
                200,
                100,
                false,
                128,
                Vec3::new(0.0, 0.0, 0.0),
                0.85,
            ),
        ],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 28,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let supported_runtime =
        recent_requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source(
            true, &extract,
        );
    let flat_runtime =
        recent_requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source(
            false, &extract,
        );
    let supported = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(supported_runtime),
    );
    let flat = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(flat_runtime),
    );

    let supported_trace = supported
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("supported pending trace-lighting probe");
    let flat_trace = flat
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("flat pending trace-lighting probe");

    assert!(
        u16::from(supported_trace[0]) > u16::from(flat_trace[0]) + 24,
        "expected one more frame of recent requested-lineage support to keep runtime/GPU trace-lighting continuation alive after the request clears, instead of collapsing pending probe source to the same flat path; flat_trace={flat_trace:?}, supported_trace={supported_trace:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_trace_lighting_blends_runtime_hierarchy_source_with_current_trace_schedule(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        1,
        vec![
            probe(100, false, 64, Vec3::new(-0.4, 0.0, 0.0), 0.75),
            probe_with_parent(
                200,
                100,
                false,
                128,
                Vec3::new(0.0, 0.0, 0.0),
                0.85,
            ),
        ],
        vec![trace_region_with_lighting(
            40,
            Vec3::new(0.0, 0.0, 0.0),
            0.9,
            0.95,
            [96, 96, 96],
        )],
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 128,
            generation: 30,
        }],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    };

    let warm_runtime =
        requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source_with_color(
            true,
            &extract,
            [240, 96, 48],
        );
    let cool_runtime =
        requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source_with_color(
            true,
            &extract,
            [48, 96, 240],
        );
    let warm = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(warm_runtime),
    );
    let cool = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(cool_runtime),
    );

    let warm_trace = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("warm pending trace-lighting probe");
    let cool_trace = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 200)
        .map(|(_, rgb)| *rgb)
        .expect("cool pending trace-lighting probe");

    assert!(
        u16::from(warm_trace[0]) > u16::from(cool_trace[0]) + 12,
        "expected current-frame trace scheduling to keep blending with runtime hierarchy RT continuation instead of letting scheduled trace lighting fully overwrite runtime lineage truth; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
    assert!(
        u16::from(cool_trace[2]) > u16::from(warm_trace[2]) + 12,
        "expected current-frame trace scheduling to keep blending with runtime hierarchy RT continuation instead of letting scheduled trace lighting fully overwrite runtime lineage truth; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_trace_lighting_gathers_runtime_grandparent_hierarchy_source_when_exact_probe_entry_is_missing(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![
            probe(100, false, 64, Vec3::new(-0.5, 0.0, 0.0), 0.7),
            probe_with_parent(
                200,
                100,
                false,
                96,
                Vec3::new(-0.1, 0.0, 0.0),
                0.8,
            ),
            probe_with_parent(
                300,
                200,
                false,
                128,
                Vec3::new(0.2, 0.0, 0.0),
                0.85,
            ),
        ],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 31,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(HybridGiResolveRuntime {
            probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
                100,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.94, 0.26, 0.12], 0.7),
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
            probe_hierarchy_rt_lighting_rgb_and_weight: std::collections::BTreeMap::from([(
                100,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.26, 0.94], 0.7),
            )]),
            ..Default::default()
        }),
    );

    let warm_trace = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm pending grandchild trace-lighting probe");
    let cool_trace = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool pending grandchild trace-lighting probe");

    assert!(
        u16::from(warm_trace[0]) > u16::from(cool_trace[0]) + 20,
        "expected GPU prepare to gather runtime hierarchy RT-lighting through the current scene-driven parent chain even when the pending probe itself has no exact runtime entry; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
    assert!(
        u16::from(cool_trace[2]) > u16::from(warm_trace[2]) + 20,
        "expected GPU prepare to keep the deeper screen-probe hierarchy RT continuation alive from a runtime grandparent source instead of collapsing both paths to the same fallback; warm_trace={warm_trace:?}, cool_trace={cool_trace:?}"
    );
}

#[test]
fn hybrid_gi_pending_probe_gpu_irradiance_gathers_runtime_grandparent_hierarchy_source_when_exact_probe_entry_is_missing(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = SceneRenderer::new(asset_manager).unwrap();
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        1,
        0,
        vec![
            probe(100, false, 64, Vec3::new(-0.5, 0.0, 0.0), 0.7),
            probe_with_parent(
                200,
                100,
                false,
                96,
                Vec3::new(-0.1, 0.0, 0.0),
                0.8,
            ),
            probe_with_parent(
                300,
                200,
                false,
                128,
                Vec3::new(0.2, 0.0, 0.0),
                0.85,
            ),
        ],
        Vec::new(),
    );
    let prepare = HybridGiPrepareFrame {
        resident_probes: Vec::new(),
        pending_updates: vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 32,
        }],
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract.clone(),
        prepare.clone(),
        Some(HybridGiResolveRuntime {
            probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                100,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.92, 0.34, 0.14], 0.68),
            )]),
            ..Default::default()
        }),
    );
    let cool = render_hybrid_gi_gpu_irradiance_readback_with_runtime(
        &mut renderer,
        viewport_size,
        extract,
        prepare,
        Some(HybridGiResolveRuntime {
            probe_hierarchy_irradiance_rgb_and_weight: std::collections::BTreeMap::from([(
                100,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.14, 0.34, 0.92], 0.68),
            )]),
            ..Default::default()
        }),
    );

    let warm_irradiance = warm
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("warm pending grandchild irradiance probe");
    let cool_irradiance = cool
        .iter()
        .find(|(probe_id, _)| *probe_id == 300)
        .map(|(_, rgb)| *rgb)
        .expect("cool pending grandchild irradiance probe");

    assert!(
        u16::from(warm_irradiance[0]) > u16::from(cool_irradiance[0]) + 20,
        "expected GPU prepare to gather runtime hierarchy irradiance through the current scene-driven parent chain even when the pending probe itself has no exact runtime entry; warm_irradiance={warm_irradiance:?}, cool_irradiance={cool_irradiance:?}"
    );
    assert!(
        u16::from(cool_irradiance[2]) > u16::from(warm_irradiance[2]) + 20,
        "expected GPU prepare to keep deeper screen-probe hierarchy irradiance continuation alive from a runtime grandparent source instead of collapsing both paths to the same fallback; warm_irradiance={warm_irradiance:?}, cool_irradiance={cool_irradiance:?}"
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
    trace_region_with_lighting(
        region_id,
        bounds_center,
        bounds_radius,
        screen_coverage,
        [0, 0, 0],
    )
}

fn trace_region_with_lighting(
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

fn render_hybrid_gi_gpu_trace_lighting_readback_with_runtime(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    runtime: Option<HybridGiResolveRuntime>,
) -> Vec<(u32, [u8; 3])> {
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
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(runtime),
            &compiled,
            None,
        )
        .unwrap();

    renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback")
        .probe_trace_lighting_rgb
}

fn render_hybrid_gi_gpu_irradiance_readback_with_runtime(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    runtime: Option<HybridGiResolveRuntime>,
) -> Vec<(u32, [u8; 3])> {
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
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_resolve_runtime(runtime),
            &compiled,
            None,
        )
        .unwrap();

    renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback")
        .probe_irradiance_rgb
}

fn requested_lineage_runtime_for_gpu_source(
    requested: bool,
    extract: &RenderFrameExtract,
) -> HybridGiResolveRuntime {
    let hybrid_gi = extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .expect("hybrid gi extract");
    let mut runtime = HybridGiRuntimeState::default();
    runtime.register_extract(Some(hybrid_gi));
    runtime.complete_gpu_updates([], [], &[], &[(200, [240, 96, 48])], &[]);
    runtime.ingest_plan(
        24,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: requested.then_some(vec![200]).unwrap_or_default(),
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.build_resolve_runtime()
}

fn direct_runtime_trace_history_for_gpu_source(
    rt_lighting_rgb: [u8; 3],
    extract: &RenderFrameExtract,
) -> HybridGiResolveRuntime {
    let hybrid_gi = extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .expect("hybrid gi extract");
    let mut runtime = HybridGiRuntimeState::default();
    runtime.register_extract(Some(hybrid_gi));
    runtime.complete_gpu_updates([], [], &[], &[(200, rt_lighting_rgb)], &[]);
    runtime.ingest_plan(
        15,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.build_resolve_runtime()
}

fn requested_lineage_irradiance_runtime_for_gpu_source(
    requested: bool,
    extract: &RenderFrameExtract,
) -> HybridGiResolveRuntime {
    let hybrid_gi = extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .expect("hybrid gi extract");
    let mut runtime = HybridGiRuntimeState::default();
    runtime.register_extract(Some(hybrid_gi));
    runtime.complete_gpu_updates([200], [], &[(200, [240, 96, 48])], &[], &[]);
    runtime.ingest_plan(
        25,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: requested.then_some(vec![200]).unwrap_or_default(),
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.build_resolve_runtime()
}

fn requested_lineage_nonresident_ancestor_irradiance_runtime_for_gpu_source(
    requested: bool,
    extract: &RenderFrameExtract,
) -> HybridGiResolveRuntime {
    let hybrid_gi = extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .expect("hybrid gi extract");
    let mut runtime = HybridGiRuntimeState::default();
    runtime.register_extract(Some(hybrid_gi));
    runtime.complete_gpu_updates([100], [], &[(100, [240, 96, 48])], &[], &[]);
    runtime.ingest_plan(
        26,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: requested.then_some(vec![200]).unwrap_or_default(),
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.build_resolve_runtime()
}

fn requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source(
    requested: bool,
    extract: &RenderFrameExtract,
) -> HybridGiResolveRuntime {
    requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source_with_color(
        requested,
        extract,
        [240, 96, 48],
    )
}

fn requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source_with_color(
    requested: bool,
    extract: &RenderFrameExtract,
    ancestor_trace_lighting_rgb: [u8; 3],
) -> HybridGiResolveRuntime {
    let hybrid_gi = extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .expect("hybrid gi extract");
    let mut runtime = HybridGiRuntimeState::default();
    runtime.register_extract(Some(hybrid_gi));
    runtime.complete_gpu_updates([], [], &[], &[(100, ancestor_trace_lighting_rgb)], &[]);
    runtime.ingest_plan(
        27,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: requested.then_some(vec![200]).unwrap_or_default(),
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.build_resolve_runtime()
}

fn recent_requested_lineage_nonresident_ancestor_trace_lighting_runtime_for_gpu_source(
    supported: bool,
    extract: &RenderFrameExtract,
) -> HybridGiResolveRuntime {
    let hybrid_gi = extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .expect("hybrid gi extract");
    let mut runtime = HybridGiRuntimeState::default();
    runtime.register_extract(Some(hybrid_gi));
    runtime.complete_gpu_updates([], [], &[], &[(100, [240, 96, 48])], &[]);
    runtime.ingest_plan(
        28,
        &crate::VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: supported.then_some(vec![200]).unwrap_or_default(),
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    runtime.ingest_plan(
        29,
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
