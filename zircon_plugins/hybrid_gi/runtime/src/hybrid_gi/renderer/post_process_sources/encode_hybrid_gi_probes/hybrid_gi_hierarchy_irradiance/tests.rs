use std::collections::{BTreeMap, BTreeSet};

use super::hybrid_gi_hierarchy_irradiance;
use crate::hybrid_gi::types::{
    HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiPrepareSurfaceCachePageContent,
    HybridGiResolveRuntime, HybridGiScenePrepareFrame,
};
use zircon_runtime::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
    RenderHybridGiProbe, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{UVec2, Vec4};

use super::HybridGiProbeEncodeFrame;

#[test]
fn exact_runtime_irradiance_keeps_blending_with_descendant_continuation() {
    let warm = hierarchy_irradiance_with_descendant(HybridGiResolveRuntime::pack_rgb_and_weight(
        [0.95, 0.28, 0.12],
        0.68,
    ));
    let cool = hierarchy_irradiance_with_descendant(HybridGiResolveRuntime::pack_rgb_and_weight(
        [0.12, 0.28, 0.95],
        0.68,
    ));

    assert!(
        warm[0] > cool[0] + 0.2,
        "expected exact runtime irradiance to keep blending with descendant continuation instead of returning the parent entry unchanged; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected descendant continuation to affect the blue channel when the child runtime turns cool; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn exact_runtime_irradiance_blends_current_surface_cache_truth_when_trace_schedule_is_empty() {
    let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.47, 0.47, 0.47], 0.62);
    let warm = hierarchy_irradiance_with_exact_runtime(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: zircon_runtime::core::math::Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [224, 112, 64, 255],
                capture_sample_rgba: [240, 96, 48, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        exact_runtime,
    );
    let cool = hierarchy_irradiance_with_exact_runtime(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: zircon_runtime::core::math::Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [64, 112, 224, 255],
                capture_sample_rgba: [48, 96, 240, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        exact_runtime,
    );

    assert!(
        warm[0] > cool[0] + 0.12,
        "expected stale exact runtime irradiance to keep blending with current warm surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.12,
        "expected stale exact runtime irradiance to keep blending with current cool surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn exact_runtime_irradiance_skips_scene_prepare_reblend_when_runtime_source_is_already_scene_driven(
) {
    let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: zircon_runtime::core::math::Vec3::ZERO,
            bounds_radius: 0.6,
            atlas_sample_rgba: [64, 112, 224, 255],
            capture_sample_rgba: [48, 96, 240, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };

    let scene_driven = hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
        scene_prepare.clone(),
        exact_runtime,
        true,
    );
    let reblended = hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
        scene_prepare,
        exact_runtime,
        false,
    );

    assert!(
        scene_driven[0] > reblended[0] + 0.08,
        "expected renderer-side hierarchy irradiance to trust a runtime source that already includes current scene truth instead of blending the same surface-cache signal a second time; scene_driven={scene_driven:?}, reblended={reblended:?}"
    );
    assert!(
        reblended[2] > scene_driven[2] + 0.08,
        "expected unflagged runtime irradiance to keep drifting toward the cool scene-prepare page while the scene-driven runtime source stays closer to its authored warm color; scene_driven={scene_driven:?}, reblended={reblended:?}"
    );
}

#[test]
fn scene_driven_exact_runtime_irradiance_ignores_descendant_lineage_tint() {
    let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
    let warm = hierarchy_irradiance_with_scene_driven_exact_and_descendant(
        exact_runtime,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.32, 0.18], 0.66),
    );
    let cool = hierarchy_irradiance_with_scene_driven_exact_and_descendant(
        exact_runtime,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.18, 0.32, 0.9], 0.66),
    );

    assert!(
        (warm[0] - cool[0]).abs() < 0.03,
        "expected scene-driven exact runtime irradiance to stay anchored to current exact scene truth instead of drifting with descendant lineage tint; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        (warm[2] - cool[2]).abs() < 0.03,
        "expected scene-driven exact runtime irradiance to keep blue output stable while descendant lineage tint changes; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        warm[0] > warm[2] + 0.35,
        "expected scene-driven exact runtime irradiance to keep its authored warm bias after descendant lineage tint changes; warm={warm:?}"
    );
}

#[test]
fn scene_driven_lineage_runtime_irradiance_ignores_scene_prepare_surface_cache_tint() {
    let inherited_warm = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
        true,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
        [240, 96, 48, 255],
    );
    let inherited_cool = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
        true,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
        [48, 96, 240, 255],
    );
    let descendant_warm = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
        false,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
        [240, 96, 48, 255],
    );
    let descendant_cool = hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
        false,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
        [48, 96, 240, 255],
    );

    assert!(
        (inherited_warm[0] - inherited_cool[0]).abs() < 0.03
            && (inherited_warm[2] - inherited_cool[2]).abs() < 0.03,
        "expected inherited scene-driven runtime irradiance to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; inherited_warm={inherited_warm:?}, inherited_cool={inherited_cool:?}"
    );
    assert!(
        inherited_warm[0] > inherited_warm[2] + 0.35,
        "expected inherited scene-driven runtime irradiance to keep its authored warm bias after scene_prepare page tint changes; inherited_warm={inherited_warm:?}"
    );
    assert!(
        (descendant_warm[0] - descendant_cool[0]).abs() < 0.03
            && (descendant_warm[2] - descendant_cool[2]).abs() < 0.03,
        "expected descendant scene-driven runtime irradiance to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; descendant_warm={descendant_warm:?}, descendant_cool={descendant_cool:?}"
    );
    assert!(
        descendant_warm[0] > descendant_warm[2] + 0.35,
        "expected descendant scene-driven runtime irradiance to keep its authored warm bias after scene_prepare page tint changes; descendant_warm={descendant_warm:?}"
    );
}

#[test]
fn scene_prepare_atlas_only_surface_cache_page_samples_provide_irradiance_fallback_without_runtime_irradiance(
) {
    let warm =
        hierarchy_irradiance_with_scene_prepare_page_content([240, 96, 48, 255], [0, 0, 0, 0]);
    let cool =
        hierarchy_irradiance_with_scene_prepare_page_content([48, 96, 240, 255], [0, 0, 0, 0]);

    assert!(
        warm[3] > 0.1,
        "expected atlas-only persisted page truth to provide nonzero irradiance fallback when runtime irradiance is absent; warm={warm:?}"
    );
    assert!(
        warm[0] > cool[0] + 0.2,
        "expected atlas-only persisted page truth to warm the fallback irradiance instead of leaving it black or flat; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected atlas-only persisted page truth to preserve blue authority in irradiance fallback when runtime irradiance is absent; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn scene_prepare_capture_surface_cache_page_samples_stay_preferred_for_irradiance_fallback_without_runtime_irradiance(
) {
    let warm = hierarchy_irradiance_with_scene_prepare_page_content(
        [48, 96, 240, 255],
        [240, 96, 48, 255],
    );
    let cool = hierarchy_irradiance_with_scene_prepare_page_content(
        [240, 96, 48, 255],
        [48, 96, 240, 255],
    );

    assert!(
        warm[3] > 0.1,
        "expected capture-side persisted page truth to provide nonzero irradiance fallback when runtime irradiance is absent; warm={warm:?}"
    );
    assert!(
        warm[0] > cool[0] + 0.2,
        "expected capture-side persisted page truth to stay preferred over atlas-side truth in irradiance fallback; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected capture-side persisted page truth to preserve blue authority over atlas-side truth in irradiance fallback; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn legacy_irradiance_uses_first_probe_payload_for_duplicate_probe_ids() {
    let irradiance = hierarchy_irradiance_with_duplicate_middle_probe_payloads();

    assert!(
        irradiance[3] <= f32::EPSILON,
        "expected legacy hierarchy irradiance to keep the first RenderHybridGiProbe payload for a duplicate probe id instead of inheriting through a later stale grandparent link; irradiance={irradiance:?}"
    );
}

#[test]
fn flat_runtime_blocks_legacy_ancestor_prepare_irradiance_inheritance() {
    let irradiance = hierarchy_irradiance_with_flat_runtime_and_legacy_ancestor_prepare();

    assert!(
        irradiance[3] <= f32::EPSILON,
        "expected flat runtime topology to stop legacy RenderHybridGiProbe parent-chain irradiance inheritance from farther resident ancestors; irradiance={irradiance:?}"
    );
}

#[test]
fn scene_representation_budget_blocks_legacy_ancestor_prepare_irradiance_inheritance() {
    let irradiance =
        hierarchy_irradiance_with_budgeted_scene_representation_and_legacy_ancestor_prepare();

    assert!(
        irradiance[3] <= f32::EPSILON,
        "expected budgeted scene-representation extracts to stop legacy RenderHybridGiProbe parent-chain irradiance inheritance from farther resident ancestors; irradiance={irradiance:?}"
    );
}

fn hierarchy_irradiance_with_descendant(descendant_runtime: [u8; 4]) -> [f32; 4] {
    let parent_probe = RenderHybridGiProbe {
        probe_id: 100,
        resident: true,
        ray_budget: 128,
        ..Default::default()
    };
    let child_probe = RenderHybridGiProbe {
        probe_id: 200,
        parent_probe_id: Some(parent_probe.probe_id),
        ray_budget: 88,
        ..Default::default()
    };

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 2,
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        probes: vec![parent_probe, child_probe],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_parent_probes(BTreeMap::from([(
                    child_probe.probe_id,
                    parent_probe.probe_id,
                )]))
                .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([
                    (
                        parent_probe.probe_id,
                        HybridGiResolveRuntime::pack_rgb_and_weight([0.5, 0.5, 0.5], 0.12),
                    ),
                    (child_probe.probe_id, descendant_runtime),
                ]))
                .build(),
        ));

    hybrid_gi_hierarchy_irradiance(&frame, &parent_probe)
}

fn hierarchy_irradiance_with_scene_prepare_page_content(
    atlas_sample_rgba: [u8; 4],
    capture_sample_rgba: [u8; 4],
) -> [f32; 4] {
    let probe = RenderHybridGiProbe {
        probe_id: 200,
        resident: true,
        ray_budget: 128,
        radius: 1.8,
        ..Default::default()
    };
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 1,
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        probes: vec![probe],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: 0,
                ray_budget: probe.ray_budget,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: zircon_runtime::core::math::Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba,
                capture_sample_rgba,
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        }));

    hybrid_gi_hierarchy_irradiance(&frame, &probe)
}

fn hierarchy_irradiance_with_duplicate_middle_probe_payloads() -> [f32; 4] {
    let grandparent_probe = RenderHybridGiProbe {
        probe_id: 100,
        resident: true,
        ray_budget: 128,
        ..Default::default()
    };
    let middle_probe = RenderHybridGiProbe {
        probe_id: 200,
        parent_probe_id: None,
        resident: true,
        ray_budget: 96,
        ..Default::default()
    };
    let stale_middle_probe = RenderHybridGiProbe {
        parent_probe_id: Some(grandparent_probe.probe_id),
        ..middle_probe
    };
    let child_probe = RenderHybridGiProbe {
        probe_id: 300,
        parent_probe_id: Some(middle_probe.probe_id),
        resident: true,
        ray_budget: 88,
        ..Default::default()
    };

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 3,
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        probes: vec![
            grandparent_probe,
            middle_probe,
            stale_middle_probe,
            child_probe,
        ],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: middle_probe.probe_id,
                    slot: 0,
                    ray_budget: middle_probe.ray_budget,
                    irradiance_rgb: [112, 112, 112],
                },
                HybridGiPrepareProbe {
                    probe_id: grandparent_probe.probe_id,
                    slot: 1,
                    ray_budget: grandparent_probe.ray_budget,
                    irradiance_rgb: [240, 96, 48],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }));

    hybrid_gi_hierarchy_irradiance(&frame, &child_probe)
}

fn hierarchy_irradiance_with_budgeted_scene_representation_and_legacy_ancestor_prepare() -> [f32; 4]
{
    let grandparent_probe = RenderHybridGiProbe {
        probe_id: 100,
        resident: true,
        ray_budget: 128,
        ..Default::default()
    };
    let parent_probe = RenderHybridGiProbe {
        probe_id: 200,
        parent_probe_id: Some(grandparent_probe.probe_id),
        resident: true,
        ray_budget: 96,
        ..Default::default()
    };
    let child_probe = RenderHybridGiProbe {
        probe_id: 300,
        parent_probe_id: Some(parent_probe.probe_id),
        resident: true,
        ray_budget: 88,
        ..Default::default()
    };

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 3,
        trace_budget: 1,
        card_budget: 1,
        voxel_budget: 1,
        probes: vec![grandparent_probe, parent_probe, child_probe],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: parent_probe.probe_id,
                    slot: 0,
                    ray_budget: parent_probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareProbe {
                    probe_id: grandparent_probe.probe_id,
                    slot: 1,
                    ray_budget: grandparent_probe.ray_budget,
                    irradiance_rgb: [240, 96, 48],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }));

    hybrid_gi_hierarchy_irradiance(&frame, &child_probe)
}

fn hierarchy_irradiance_with_flat_runtime_and_legacy_ancestor_prepare() -> [f32; 4] {
    let grandparent_probe = RenderHybridGiProbe {
        probe_id: 100,
        resident: true,
        ray_budget: 128,
        ..Default::default()
    };
    let parent_probe = RenderHybridGiProbe {
        probe_id: 200,
        parent_probe_id: Some(grandparent_probe.probe_id),
        resident: true,
        ray_budget: 96,
        ..Default::default()
    };
    let child_probe = RenderHybridGiProbe {
        probe_id: 300,
        parent_probe_id: Some(parent_probe.probe_id),
        resident: true,
        ray_budget: 88,
        ..Default::default()
    };

    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 3,
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        probes: vec![grandparent_probe, parent_probe, child_probe],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: parent_probe.probe_id,
                    slot: 0,
                    ray_budget: parent_probe.ray_budget,
                    irradiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareProbe {
                    probe_id: grandparent_probe.probe_id,
                    slot: 1,
                    ray_budget: grandparent_probe.ray_budget,
                    irradiance_rgb: [240, 96, 48],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()));

    hybrid_gi_hierarchy_irradiance(&frame, &child_probe)
}

fn hierarchy_irradiance_with_exact_runtime(
    scene_prepare: HybridGiScenePrepareFrame,
    exact_runtime: [u8; 4],
) -> [f32; 4] {
    hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
        scene_prepare,
        exact_runtime,
        false,
    )
}

fn hierarchy_irradiance_with_exact_runtime_and_scene_driven_flag(
    scene_prepare: HybridGiScenePrepareFrame,
    exact_runtime: [u8; 4],
    scene_driven: bool,
) -> [f32; 4] {
    let probe = RenderHybridGiProbe {
        probe_id: 200,
        resident: true,
        ray_budget: 128,
        radius: 1.8,
        ..Default::default()
    };
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 1,
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        probes: vec![probe],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: 0,
                ray_budget: probe.ray_budget,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_scene_prepare(Some(scene_prepare))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                    probe.probe_id,
                    exact_runtime,
                )]))
                .with_probe_scene_driven_hierarchy_irradiance_ids(
                    scene_driven
                        .then(|| BTreeSet::from([probe.probe_id]))
                        .unwrap_or_default(),
                )
                .build(),
        ));

    hybrid_gi_hierarchy_irradiance(&frame, &probe)
}

fn hierarchy_irradiance_with_scene_driven_exact_and_descendant(
    exact_runtime: [u8; 4],
    descendant_runtime: [u8; 4],
) -> [f32; 4] {
    let probe = RenderHybridGiProbe {
        probe_id: 200,
        resident: true,
        ray_budget: 128,
        radius: 1.8,
        ..Default::default()
    };
    let descendant_probe = RenderHybridGiProbe {
        probe_id: 260,
        parent_probe_id: Some(probe.probe_id),
        resident: false,
        ray_budget: 88,
        radius: 1.2,
        ..Default::default()
    };
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 2,
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        probes: vec![probe, descendant_probe],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: 0,
                ray_budget: probe.ray_budget,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        }))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([
                    (probe.probe_id, exact_runtime),
                    (descendant_probe.probe_id, descendant_runtime),
                ]))
                .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([
                    probe.probe_id,
                    descendant_probe.probe_id,
                ]))
                .build(),
        ));

    hybrid_gi_hierarchy_irradiance(&frame, &probe)
}

fn hierarchy_irradiance_with_scene_driven_lineage_and_scene_prepare_page(
    scene_truth_on_ancestor: bool,
    lineage_runtime: [u8; 4],
    page_capture_sample_rgba: [u8; 4],
) -> [f32; 4] {
    let parent_probe = RenderHybridGiProbe {
        probe_id: 100,
        resident: true,
        ray_budget: 128,
        radius: 1.8,
        ..Default::default()
    };
    let child_probe = RenderHybridGiProbe {
        probe_id: 200,
        parent_probe_id: Some(parent_probe.probe_id),
        resident: true,
        ray_budget: 96,
        radius: 1.2,
        ..Default::default()
    };
    let encoded_probe = if scene_truth_on_ancestor {
        child_probe
    } else {
        parent_probe
    };
    let runtime_probe_id = if scene_truth_on_ancestor {
        parent_probe.probe_id
    } else {
        child_probe.probe_id
    };
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    };
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        probe_budget: 2,
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        probes: vec![parent_probe, child_probe],
        ..Default::default()
    });

    let frame = HybridGiProbeEncodeFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: encoded_probe.probe_id,
                slot: 0,
                ray_budget: encoded_probe.ray_budget,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: zircon_runtime::core::math::Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: page_capture_sample_rgba,
                capture_sample_rgba: page_capture_sample_rgba,
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        }))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_parent_probes(BTreeMap::from([(
                    child_probe.probe_id,
                    parent_probe.probe_id,
                )]))
                .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                    runtime_probe_id,
                    lineage_runtime,
                )]))
                .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([
                    runtime_probe_id,
                ]))
                .build(),
        ));

    hybrid_gi_hierarchy_irradiance(&frame, &encoded_probe)
}
