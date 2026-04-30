use std::collections::{BTreeMap, BTreeSet};

use super::super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::{
    hybrid_gi_hierarchy_rt_lighting, hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources,
};
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
    RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec3, Vec4};
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
    HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
    HybridGiResolveRuntime, HybridGiScenePrepareFrame, ViewportRenderFrame,
};

#[test]
fn exact_runtime_rt_lighting_keeps_blending_with_descendant_continuation() {
    let warm = hierarchy_rt_lighting_with_descendant(HybridGiResolveRuntime::pack_rgb_and_weight(
        [0.95, 0.3, 0.12],
        0.62,
    ));
    let cool = hierarchy_rt_lighting_with_descendant(HybridGiResolveRuntime::pack_rgb_and_weight(
        [0.12, 0.3, 0.95],
        0.62,
    ));

    assert!(
        warm[0] > cool[0] + 0.2,
        "expected exact runtime RT lighting to keep blending with descendant continuation instead of shadowing it; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected descendant continuation to affect RT blue when the child runtime turns cool; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn exact_runtime_rt_lighting_blends_current_surface_cache_truth_when_trace_schedule_is_empty() {
    let direct_runtime_rgb = [120, 120, 120];
    let warm = scene_prepare_rt_lighting_with_exact_runtime(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [224, 112, 64, 255],
                capture_sample_rgba: [240, 96, 48, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        direct_runtime_rgb,
    );
    let cool = scene_prepare_rt_lighting_with_exact_runtime(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [64, 112, 224, 255],
                capture_sample_rgba: [48, 96, 240, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        direct_runtime_rgb,
    );

    assert!(
        warm[0] > cool[0] + 0.12,
        "expected stale exact runtime RT lighting to keep blending with current warm surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.12,
        "expected stale exact runtime RT lighting to keep blending with current cool surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn exact_runtime_rt_lighting_skips_scene_prepare_reblend_when_runtime_source_is_already_scene_driven(
) {
    let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 11,
            owner_card_id: 11,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.6,
            atlas_sample_rgba: [64, 112, 224, 255],
            capture_sample_rgba: [48, 96, 240, 255],
        }],
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    };

    let scene_driven = scene_prepare_rt_lighting_with_exact_runtime_and_scene_driven_flag(
        scene_prepare.clone(),
        exact_runtime,
        true,
    );
    let reblended = scene_prepare_rt_lighting_with_exact_runtime_and_scene_driven_flag(
        scene_prepare,
        exact_runtime,
        false,
    );

    assert!(
        scene_driven[0] > reblended[0] + 0.08,
        "expected renderer-side hierarchy RT lighting to trust a runtime source that already includes current scene truth instead of blending the same surface-cache signal a second time; scene_driven={scene_driven:?}, reblended={reblended:?}"
    );
    assert!(
        reblended[2] > scene_driven[2] + 0.08,
        "expected unflagged runtime RT lighting to keep drifting toward the cool scene-prepare page while the scene-driven runtime source stays closer to its authored warm color; scene_driven={scene_driven:?}, reblended={reblended:?}"
    );
}

#[test]
fn scene_driven_exact_runtime_rt_lighting_ignores_descendant_lineage_tint() {
    let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
    let warm = scene_prepare_rt_lighting_with_scene_driven_exact_and_descendant(
        exact_runtime,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.32, 0.18], 0.66),
    );
    let cool = scene_prepare_rt_lighting_with_scene_driven_exact_and_descendant(
        exact_runtime,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.18, 0.32, 0.9], 0.66),
    );

    assert!(
        (warm[0] - cool[0]).abs() < 0.03,
        "expected scene-driven exact runtime RT lighting to stay anchored to current exact scene truth instead of drifting with descendant lineage tint; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        (warm[2] - cool[2]).abs() < 0.03,
        "expected scene-driven exact runtime RT lighting to keep blue output stable while descendant lineage tint changes; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        warm[0] > warm[2] + 0.35,
        "expected scene-driven exact runtime RT lighting to keep its authored warm bias after descendant lineage tint changes; warm={warm:?}"
    );
}

#[test]
fn scene_driven_lineage_runtime_rt_lighting_ignores_scene_prepare_surface_cache_tint() {
    let inherited_warm = scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
        true,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
        [240, 96, 48, 255],
    );
    let inherited_cool = scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
        true,
        HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
        [48, 96, 240, 255],
    );
    let descendant_warm =
        scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
            false,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
            [240, 96, 48, 255],
        );
    let descendant_cool =
        scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
            false,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
            [48, 96, 240, 255],
        );

    assert!(
        (inherited_warm[0] - inherited_cool[0]).abs() < 0.03
            && (inherited_warm[2] - inherited_cool[2]).abs() < 0.03,
        "expected inherited scene-driven runtime RT lighting to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; inherited_warm={inherited_warm:?}, inherited_cool={inherited_cool:?}"
    );
    assert!(
        inherited_warm[0] > inherited_warm[2] + 0.35,
        "expected inherited scene-driven runtime RT lighting to keep its authored warm bias after scene_prepare page tint changes; inherited_warm={inherited_warm:?}"
    );
    assert!(
        (descendant_warm[0] - descendant_cool[0]).abs() < 0.03
            && (descendant_warm[2] - descendant_cool[2]).abs() < 0.03,
        "expected descendant scene-driven runtime RT lighting to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; descendant_warm={descendant_warm:?}, descendant_cool={descendant_cool:?}"
    );
    assert!(
        descendant_warm[0] > descendant_warm[2] + 0.35,
        "expected descendant scene-driven runtime RT lighting to keep its authored warm bias after scene_prepare page tint changes; descendant_warm={descendant_warm:?}"
    );
}

#[test]
fn scene_driven_inherited_legacy_probe_rt_lighting_uses_legacy_when_packed_hierarchy_rt_is_zero() {
    let inherited_rt_lighting = inherited_legacy_probe_rt_lighting_with_zero_packed_hierarchy_rt();

    assert!(
        inherited_rt_lighting[0] > 0.45,
        "expected inherited legacy RT lighting to remain visible when the packed hierarchy RT scene-truth source has zero support; inherited_rt_lighting={inherited_rt_lighting:?}"
    );
    assert!(
        inherited_rt_lighting[3] > 0.1,
        "expected inherited legacy RT lighting resolve weight to provide support when packed hierarchy RT is zero-weight; inherited_rt_lighting={inherited_rt_lighting:?}"
    );
}

#[test]
fn legacy_trace_region_inheritance_counts_duplicate_scheduled_live_payload_once() {
    let single = inherited_trace_region_rt_lighting_with_scheduled_region_ids(vec![40]);
    let duplicate = inherited_trace_region_rt_lighting_with_scheduled_region_ids(vec![40, 40]);

    assert!(
        (single[3] - duplicate[3]).abs() < f32::EPSILON,
        "expected duplicate scheduled ids for the same live RenderHybridGiTraceRegion payload not to inflate inherited RT lighting support; single={single:?}, duplicate={duplicate:?}"
    );
    assert!(
        single[3] > 0.0,
        "expected the live trace payload to remain visible after scheduled-id de-duplication; single={single:?}"
    );
}

#[test]
fn legacy_trace_region_inheritance_respects_live_payload_region_limit() {
    let rt_lighting = inherited_trace_region_rt_lighting_with_budget_excess_tail_payload();

    assert!(
        rt_lighting[3] <= f32::EPSILON,
        "expected legacy trace-region inheritance to respect the same live-payload GPU budget as trace-region encoding instead of reading a 17th scheduled payload; rt_lighting={rt_lighting:?}"
    );
}

#[test]
fn legacy_trace_region_inheritance_uses_first_payload_for_duplicate_region_ids() {
    let rt_lighting = inherited_trace_region_rt_lighting_with_duplicate_region_payloads();

    assert!(
        rt_lighting[0] > rt_lighting[2] + 0.2,
        "expected legacy RT-lighting inheritance to match runtime registration and use the first RenderHybridGiTraceRegion payload for a duplicate region id; rt_lighting={rt_lighting:?}"
    );
    assert!(
        rt_lighting[3] > 0.0,
        "expected the first duplicate trace-region payload to remain live; rt_lighting={rt_lighting:?}"
    );
}

#[test]
fn legacy_trace_region_inheritance_uses_first_probe_payload_for_duplicate_probe_ids() {
    let rt_lighting = inherited_trace_region_rt_lighting_with_duplicate_child_probe_payloads();

    assert!(
        rt_lighting[3] <= f32::EPSILON,
        "expected legacy RT-lighting inheritance to keep the first RenderHybridGiProbe payload for a duplicate probe id instead of inheriting through a later stale parent link; rt_lighting={rt_lighting:?}"
    );
}

#[test]
fn flat_runtime_blocks_legacy_trace_region_rt_lighting_inheritance() {
    let rt_lighting = inherited_trace_region_rt_lighting_with_flat_runtime();

    assert!(
        rt_lighting[3] <= f32::EPSILON,
        "expected flat runtime topology to stop legacy RenderHybridGiProbe parent-chain trace-region RT inheritance; rt_lighting={rt_lighting:?}"
    );
}

#[test]
fn scene_prepare_voxel_cell_resource_samples_override_spatial_fallback_when_runtime_authority_is_absent(
) {
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
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
    };

    let warm = scene_prepare_rt_lighting_with_resources(
        scene_prepare.clone(),
        scene_prepare_resources_snapshot(
            vec![(7, [160, 120, 96, 255])],
            vec![
                (7, 20, [240, 96, 48, 255]),
                (7, 21, [240, 96, 48, 255]),
                (7, 24, [240, 96, 48, 255]),
                (7, 25, [240, 96, 48, 255]),
            ],
            Vec::new(),
        ),
    );
    let cool = scene_prepare_rt_lighting_with_resources(
        scene_prepare,
        scene_prepare_resources_snapshot(
            vec![(7, [96, 120, 160, 255])],
            vec![
                (7, 20, [48, 96, 240, 255]),
                (7, 21, [48, 96, 240, 255]),
                (7, 24, [48, 96, 240, 255]),
                (7, 25, [48, 96, 240, 255]),
            ],
            Vec::new(),
        ),
    );

    assert!(
        warm[0] > cool[0] + 0.2,
        "expected current-frame scene_prepare voxel-cell resource samples to override the spatial fallback when runtime radiance/owner authority is absent; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected current-frame scene_prepare voxel-cell resource samples to preserve blue authority when runtime radiance/owner authority is absent; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn scene_prepare_voxel_clipmap_resource_samples_override_spatial_fallback_when_runtime_cells_are_absent(
) {
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: Vec::new(),
        voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::new(0.0, 0.0, 0.2),
            half_extent: 4.0,
        }],
        voxel_cells: Vec::new(),
    };

    let warm = scene_prepare_rt_lighting_with_resources(
        scene_prepare.clone(),
        scene_prepare_resources_snapshot(vec![(7, [224, 96, 48, 255])], Vec::new(), Vec::new()),
    );
    let cool = scene_prepare_rt_lighting_with_resources(
        scene_prepare,
        scene_prepare_resources_snapshot(vec![(7, [48, 96, 224, 255])], Vec::new(), Vec::new()),
    );

    assert!(
        warm[0] > cool[0] + 0.2,
        "expected current-frame scene_prepare voxel-clipmap resource samples to override the coarse spatial fallback when runtime cells are absent; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected current-frame scene_prepare voxel-clipmap resource samples to preserve blue authority when runtime cells are absent; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn scene_prepare_present_black_voxel_cell_resource_samples_stay_authoritative_over_spatial_fallback(
) {
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
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
    };

    let present_black = scene_prepare_rt_lighting_with_resources(
        scene_prepare.clone(),
        scene_prepare_resources_snapshot(
            Vec::new(),
            Vec::new(),
            repeated_cell_samples([0, 0, 0, 255]),
        ),
    );
    let absent = scene_prepare_rt_lighting_with_resources(
        scene_prepare,
        scene_prepare_resources_snapshot(
            Vec::new(),
            Vec::new(),
            repeated_cell_samples([0, 0, 0, 0]),
        ),
    );

    assert!(
        present_black[3] > 0.18,
        "expected explicit-black cell resources to remain a valid GI fallback source; present_black={present_black:?}"
    );
    assert!(
        rgb_energy(present_black) < 0.05,
        "expected explicit-black cell resources to stay black instead of collapsing back to spatial fallback; present_black={present_black:?}"
    );
    assert!(
        rgb_energy(absent) > rgb_energy(present_black) + 0.2,
        "expected absent cell resources to fall back to the spatial heuristic while explicit-black resources remain authoritative; present_black={present_black:?}, absent={absent:?}"
    );
}

#[test]
fn scene_prepare_present_black_voxel_clipmap_resource_samples_stay_authoritative_over_spatial_fallback(
) {
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: Vec::new(),
        voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::new(0.0, 0.0, 0.2),
            half_extent: 4.0,
        }],
        voxel_cells: Vec::new(),
    };

    let present_black = scene_prepare_rt_lighting_with_resources(
        scene_prepare.clone(),
        scene_prepare_resources_snapshot(vec![(7, [0, 0, 0, 255])], Vec::new(), Vec::new()),
    );
    let absent = scene_prepare_rt_lighting_with_resources(
        scene_prepare,
        scene_prepare_resources_snapshot(vec![(7, [0, 0, 0, 0])], Vec::new(), Vec::new()),
    );

    assert!(
        present_black[3] > 0.18,
        "expected explicit-black clipmap resources to remain a valid GI fallback source; present_black={present_black:?}"
    );
    assert!(
        rgb_energy(present_black) < 0.05,
        "expected explicit-black clipmap resources to stay black instead of collapsing back to spatial fallback; present_black={present_black:?}"
    );
    assert!(
        rgb_energy(absent) > rgb_energy(present_black) + 0.2,
        "expected absent clipmap resources to fall back to the spatial heuristic while explicit-black clipmap resources remain authoritative; present_black={present_black:?}, absent={absent:?}"
    );
}

#[test]
fn runtime_explicit_black_voxel_radiance_stays_authoritative_over_owner_card_and_spatial_fallback()
{
    let card_capture_request = HybridGiPrepareCardCaptureRequest {
        card_id: 11,
        page_id: 22,
        atlas_slot_id: 3,
        capture_slot_id: 4,
        bounds_center: Vec3::new(20.0, 20.0, 20.0),
        bounds_radius: 0.25,
    };
    let scene_prepare = HybridGiScenePrepareFrame {
        card_capture_requests: vec![card_capture_request],
        surface_cache_page_contents: Vec::new(),
        voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 4.0,
        }],
        voxel_cells: runtime_owner_voxel_cells(true),
    };

    let present_black = scene_prepare_rt_lighting_with_resources(
        scene_prepare.clone(),
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );
    let absent = scene_prepare_rt_lighting_with_resources(
        HybridGiScenePrepareFrame {
            voxel_cells: runtime_owner_voxel_cells(false),
            ..scene_prepare
        },
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );

    assert!(
        present_black[3] > 0.18,
        "expected explicit-black runtime voxel radiance to remain a valid GI fallback source; present_black={present_black:?}"
    );
    assert!(
        rgb_energy(present_black) < 0.05,
        "expected explicit-black runtime voxel radiance to stay black instead of collapsing to owner-card or spatial fallback; present_black={present_black:?}"
    );
    assert!(
        rgb_energy(absent) > rgb_energy(present_black) + 0.2,
        "expected absent runtime voxel radiance to fall back to owner-card or spatial fallback while explicit-black runtime radiance remains authoritative; present_black={present_black:?}, absent={absent:?}"
    );
}

#[test]
fn scene_prepare_card_capture_request_falls_back_to_atlas_resource_sample_when_capture_resource_sample_is_absent(
) {
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
        voxel_cells: runtime_owner_voxel_cells(false),
    };

    let warm = scene_prepare_rt_lighting_with_resources(
        scene_prepare.clone(),
        scene_prepare_resources_snapshot_with_surface_cache_samples(
            vec![(3, [224, 112, 64, 255])],
            vec![(4, [0, 0, 0, 0])],
        ),
    );
    let cool = scene_prepare_rt_lighting_with_resources(
        scene_prepare,
        scene_prepare_resources_snapshot_with_surface_cache_samples(
            vec![(3, [64, 112, 224, 255])],
            vec![(4, [0, 0, 0, 0])],
        ),
    );

    assert!(
        warm[0] > cool[0] + 0.2,
        "expected absent capture-side resource samples to fall back to atlas-side truth instead of flattening request-owned voxel fallback to black; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected atlas-side resource samples to stay color-authoritative when capture-side resource samples are absent; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn scene_prepare_card_capture_request_ignores_absent_resource_samples_and_keeps_synthesized_fallback(
) {
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
        voxel_cells: runtime_owner_voxel_cells(false),
    };

    let baseline = scene_prepare_rt_lighting_with_resources(
        scene_prepare.clone(),
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );
    let absent = scene_prepare_rt_lighting_with_resources(
        scene_prepare,
        scene_prepare_resources_snapshot_with_surface_cache_samples(
            vec![(3, [0, 0, 0, 0])],
            vec![(4, [0, 0, 0, 0])],
        ),
    );

    assert_eq!(
        absent,
        baseline,
        "expected zero-alpha request resource samples to be treated as absent and fall through to the synthesized request fallback instead of becoming a false black authority; baseline={baseline:?}, absent={absent:?}"
    );
}

#[test]
fn scene_prepare_persisted_surface_cache_page_samples_override_spatial_fallback_when_owner_request_is_absent(
) {
    let warm = scene_prepare_rt_lighting_with_resources(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.25,
                atlas_sample_rgba: [224, 112, 64, 255],
                capture_sample_rgba: [240, 96, 48, 255],
            }],
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: runtime_owner_voxel_cells(false),
        },
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );
    let cool = scene_prepare_rt_lighting_with_resources(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.25,
                atlas_sample_rgba: [64, 112, 224, 255],
                capture_sample_rgba: [48, 96, 240, 255],
            }],
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: runtime_owner_voxel_cells(false),
        },
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );

    assert!(
        warm[0] > cool[0] + 0.2,
        "expected clean-frame persisted surface-cache samples to drive owner-card voxel fallback when no current card-capture request exists; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected clean-frame persisted surface-cache samples to preserve blue authority on owner-card voxel fallback when no current card-capture request exists; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn scene_prepare_persisted_surface_cache_page_samples_provide_spatial_fallback_without_runtime_voxel_support(
) {
    let warm = scene_prepare_rt_lighting_with_resources(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [224, 112, 64, 255],
                capture_sample_rgba: [240, 96, 48, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );
    let cool = scene_prepare_rt_lighting_with_resources(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [64, 112, 224, 255],
                capture_sample_rgba: [48, 96, 240, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );

    assert!(
        warm[3] > 0.18,
        "expected persisted surface-cache pages to remain a valid GI fallback source even when runtime voxel support is absent; warm={warm:?}"
    );
    assert!(
        warm[0] > cool[0] + 0.2,
        "expected nearby persisted surface-cache page samples to provide warm authority without runtime voxel support; warm={warm:?}, cool={cool:?}"
    );
    assert!(
        cool[2] > warm[2] + 0.2,
        "expected nearby persisted surface-cache page samples to preserve blue authority without runtime voxel support; warm={warm:?}, cool={cool:?}"
    );
}

#[test]
fn scene_prepare_absent_surface_cache_page_samples_do_not_create_false_black_fallback_without_runtime_voxel_support(
) {
    let absent = scene_prepare_rt_lighting_with_resources(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [0, 0, 0, 0],
                capture_sample_rgba: [0, 0, 0, 0],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );
    let explicit_black = scene_prepare_rt_lighting_with_resources(
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [0, 0, 0, 255],
                capture_sample_rgba: [0, 0, 0, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
        scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
    );

    assert!(
        absent[3] <= f32::EPSILON,
        "expected truly absent persisted surface-cache samples to produce no fallback support instead of a false black GI source; absent={absent:?}"
    );
    assert!(
        explicit_black[3] > 0.18,
        "expected explicit-black persisted surface-cache samples to remain a valid GI fallback source; explicit_black={explicit_black:?}"
    );
    assert!(
        rgb_energy(explicit_black) < 0.05,
        "expected explicit-black persisted surface-cache samples to stay black instead of reintroducing a color heuristic; explicit_black={explicit_black:?}"
    );
}

fn hierarchy_rt_lighting_with_descendant(descendant_runtime: [u8; 4]) -> [f32; 4] {
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

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_parent_probes(BTreeMap::from([(
                    child_probe.probe_id,
                    parent_probe.probe_id,
                )]))
                .with_probe_hierarchy_rt_lighting_rgb_and_weight(BTreeMap::from([
                    (
                        parent_probe.probe_id,
                        HybridGiResolveRuntime::pack_rgb_and_weight([0.5, 0.5, 0.5], 0.12),
                    ),
                    (child_probe.probe_id, descendant_runtime),
                ]))
                .build(),
        ));

    hybrid_gi_hierarchy_rt_lighting(&frame, &parent_probe)
}

fn scene_prepare_rt_lighting_with_resources(
    scene_prepare: HybridGiScenePrepareFrame,
    scene_prepare_resources: HybridGiScenePrepareResourcesSnapshot,
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
        card_budget: 0,
        voxel_budget: 1,
        probes: vec![probe],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: 0,
                ray_budget: probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_scene_prepare(Some(scene_prepare));

    hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources(
        &frame,
        &probe,
        Some(&scene_prepare_resources),
    )
}

fn scene_prepare_rt_lighting_with_exact_runtime(
    scene_prepare: HybridGiScenePrepareFrame,
    direct_runtime_rgb: [u8; 3],
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
        voxel_budget: 1,
        probes: vec![probe],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: 0,
                ray_budget: probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_scene_prepare(Some(scene_prepare))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_rt_lighting_rgb(BTreeMap::from([(probe.probe_id, direct_runtime_rgb)]))
                .build(),
        ));

    hybrid_gi_hierarchy_rt_lighting(&frame, &probe)
}

fn scene_prepare_rt_lighting_with_exact_runtime_and_scene_driven_flag(
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
        voxel_budget: 1,
        probes: vec![probe],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: 0,
                ray_budget: probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_scene_prepare(Some(scene_prepare))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_hierarchy_rt_lighting_rgb_and_weight(BTreeMap::from([(
                    probe.probe_id,
                    exact_runtime,
                )]))
                .with_probe_scene_driven_hierarchy_rt_lighting_ids(
                    scene_driven
                        .then(|| BTreeSet::from([probe.probe_id]))
                        .unwrap_or_default(),
                )
                .build(),
        ));

    hybrid_gi_hierarchy_rt_lighting(&frame, &probe)
}

fn scene_prepare_rt_lighting_with_scene_driven_exact_and_descendant(
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
        voxel_budget: 1,
        probes: vec![probe, descendant_probe],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: 0,
                ray_budget: probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
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
                .with_probe_hierarchy_rt_lighting_rgb_and_weight(BTreeMap::from([
                    (probe.probe_id, exact_runtime),
                    (descendant_probe.probe_id, descendant_runtime),
                ]))
                .with_probe_scene_driven_hierarchy_rt_lighting_ids(BTreeSet::from([
                    probe.probe_id,
                    descendant_probe.probe_id,
                ]))
                .build(),
        ));

    hybrid_gi_hierarchy_rt_lighting(&frame, &probe)
}

fn scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
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
        voxel_budget: 1,
        probes: vec![parent_probe, child_probe],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: encoded_probe.probe_id,
                slot: 0,
                ray_budget: encoded_probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
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
                bounds_center: Vec3::ZERO,
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
                .with_probe_hierarchy_rt_lighting_rgb_and_weight(BTreeMap::from([(
                    runtime_probe_id,
                    lineage_runtime,
                )]))
                .with_probe_scene_driven_hierarchy_rt_lighting_ids(BTreeSet::from([
                    runtime_probe_id,
                ]))
                .build(),
        ));

    hybrid_gi_hierarchy_rt_lighting(&frame, &encoded_probe)
}

fn inherited_legacy_probe_rt_lighting_with_zero_packed_hierarchy_rt() -> [f32; 4] {
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

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: child_probe.probe_id,
                slot: 0,
                ray_budget: child_probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_resolve_runtime(Some(
            HybridGiResolveRuntime::fixture()
                .with_probe_parent_probes(BTreeMap::from([(
                    child_probe.probe_id,
                    parent_probe.probe_id,
                )]))
                .with_probe_rt_lighting_rgb(BTreeMap::from([(
                    parent_probe.probe_id,
                    [240, 96, 48],
                )]))
                .with_probe_hierarchy_resolve_weight_q8(BTreeMap::from([(
                    parent_probe.probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                )]))
                .with_probe_hierarchy_rt_lighting_rgb_and_weight(BTreeMap::from([(
                    parent_probe.probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.05, 0.05, 0.05], 0.0),
                )]))
                .with_probe_scene_driven_hierarchy_rt_lighting_ids(BTreeSet::from([
                    parent_probe.probe_id
                ]))
                .build(),
        ));

    hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
}

fn inherited_trace_region_rt_lighting_with_scheduled_region_ids(
    scheduled_trace_region_ids: Vec<u32>,
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
        trace_regions: vec![RenderHybridGiTraceRegion {
            entity: 40,
            region_id: 40,
            bounds_center: Vec3::ZERO,
            bounds_radius: 2.0,
            screen_coverage: 1.0,
            rt_lighting_rgb: [240, 96, 48],
        }],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: parent_probe.probe_id,
                slot: 0,
                ray_budget: parent_probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids,
            evictable_probe_ids: Vec::new(),
        }));

    hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
}

fn inherited_trace_region_rt_lighting_with_budget_excess_tail_payload() -> [f32; 4] {
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
    let live_tail_region_id = 40;
    let budget_filler_region_ids = (0..MAX_HYBRID_GI_TRACE_REGIONS)
        .map(|index| 10_000 + index as u32)
        .collect::<Vec<_>>();
    let mut scheduled_trace_region_ids = budget_filler_region_ids.clone();
    scheduled_trace_region_ids.push(live_tail_region_id);
    let mut trace_regions = budget_filler_region_ids
        .iter()
        .copied()
        .map(|region_id| RenderHybridGiTraceRegion {
            entity: u64::from(region_id),
            region_id,
            bounds_center: Vec3::new(10_000.0, 0.0, 0.0),
            bounds_radius: 0.5,
            screen_coverage: 1.0,
            rt_lighting_rgb: [240, 96, 48],
        })
        .collect::<Vec<_>>();
    trace_regions.push(RenderHybridGiTraceRegion {
        entity: u64::from(live_tail_region_id),
        region_id: live_tail_region_id,
        bounds_center: Vec3::ZERO,
        bounds_radius: 2.0,
        screen_coverage: 1.0,
        rt_lighting_rgb: [240, 96, 48],
    });

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
        trace_regions,
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: parent_probe.probe_id,
                slot: 0,
                ray_budget: parent_probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids,
            evictable_probe_ids: Vec::new(),
        }));

    hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
}

fn inherited_trace_region_rt_lighting_with_duplicate_region_payloads() -> [f32; 4] {
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
    let region_id = 40;
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
        trace_regions: vec![
            RenderHybridGiTraceRegion {
                entity: 40,
                region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 1.0,
                rt_lighting_rgb: [240, 96, 48],
            },
            RenderHybridGiTraceRegion {
                entity: 41,
                region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 1.0,
                rt_lighting_rgb: [48, 96, 240],
            },
        ],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: parent_probe.probe_id,
                slot: 0,
                ray_budget: parent_probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![region_id],
            evictable_probe_ids: Vec::new(),
        }));

    hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
}

fn inherited_trace_region_rt_lighting_with_duplicate_child_probe_payloads() -> [f32; 4] {
    let parent_probe = RenderHybridGiProbe {
        probe_id: 100,
        resident: true,
        ray_budget: 128,
        radius: 1.8,
        ..Default::default()
    };
    let child_probe = RenderHybridGiProbe {
        probe_id: 200,
        parent_probe_id: None,
        resident: true,
        ray_budget: 96,
        radius: 1.2,
        ..Default::default()
    };
    let stale_child_probe = RenderHybridGiProbe {
        parent_probe_id: Some(parent_probe.probe_id),
        ..child_probe
    };
    let region_id = 40;
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
        probes: vec![parent_probe, child_probe, stale_child_probe],
        trace_regions: vec![RenderHybridGiTraceRegion {
            entity: 40,
            region_id,
            bounds_center: Vec3::ZERO,
            bounds_radius: 2.0,
            screen_coverage: 1.0,
            rt_lighting_rgb: [240, 96, 48],
        }],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: parent_probe.probe_id,
                slot: 0,
                ray_budget: parent_probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![region_id],
            evictable_probe_ids: Vec::new(),
        }));

    hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
}

fn inherited_trace_region_rt_lighting_with_flat_runtime() -> [f32; 4] {
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
    let region_id = 40;
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
        trace_regions: vec![RenderHybridGiTraceRegion {
            entity: 40,
            region_id,
            bounds_center: Vec3::ZERO,
            bounds_radius: 2.0,
            screen_coverage: 1.0,
            rt_lighting_rgb: [240, 96, 48],
        }],
        ..Default::default()
    });

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
        .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: parent_probe.probe_id,
                slot: 0,
                ray_budget: parent_probe.ray_budget,
                irradiance_rgb: [112, 112, 112],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![region_id],
            evictable_probe_ids: Vec::new(),
        }))
        .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime::default()));

    hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
}

fn scene_prepare_resources_snapshot(
    voxel_clipmap_rgba_samples: Vec<(u32, [u8; 4])>,
    voxel_clipmap_cell_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    voxel_clipmap_cell_dominant_rgba_samples: Vec<(u32, u32, [u8; 4])>,
) -> HybridGiScenePrepareResourcesSnapshot {
    let mut snapshot = HybridGiScenePrepareResourcesSnapshot::new(
        0,
        vec![7],
        Vec::new(),
        Vec::new(),
        0,
        0,
        (0, 0),
        (0, 0),
        0,
    );
    snapshot.store_voxel_resource_samples(
        voxel_clipmap_rgba_samples,
        Vec::new(),
        voxel_clipmap_cell_rgba_samples,
        Vec::new(),
        Vec::new(),
        voxel_clipmap_cell_dominant_rgba_samples,
    );
    snapshot
}

fn scene_prepare_resources_snapshot_with_surface_cache_samples(
    atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
) -> HybridGiScenePrepareResourcesSnapshot {
    let occupied_atlas_slots = atlas_slot_rgba_samples
        .iter()
        .map(|(slot_id, _)| *slot_id)
        .collect();
    let occupied_capture_slots = capture_slot_rgba_samples
        .iter()
        .map(|(slot_id, _)| *slot_id)
        .collect();
    let mut snapshot = HybridGiScenePrepareResourcesSnapshot::new(
        0,
        vec![7],
        occupied_atlas_slots,
        occupied_capture_slots,
        0,
        0,
        (0, 0),
        (0, 0),
        0,
    );
    snapshot.store_texture_slot_rgba_samples(atlas_slot_rgba_samples, capture_slot_rgba_samples);
    snapshot
}

fn repeated_cell_samples(rgba: [u8; 4]) -> Vec<(u32, u32, [u8; 4])> {
    [20_u32, 21, 24, 25]
        .into_iter()
        .map(|cell_index| (7, cell_index, rgba))
        .collect()
}

fn runtime_owner_voxel_cells(radiance_present: bool) -> Vec<HybridGiPrepareVoxelCell> {
    [20_u32, 21, 24, 25]
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

fn rgb_energy(sample: [f32; 4]) -> f32 {
    sample[0] + sample[1] + sample[2]
}
