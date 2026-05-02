use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderHybridGiProbe,
    RenderHybridGiTraceRegion, RenderMeshSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::graphics::{VisibilityHybridGiFeedback, VisibilityHybridGiUpdatePlan};

use crate::hybrid_gi::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
    HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareUpdateRequest, HybridGiPrepareVoxelCell,
    HybridGiPrepareVoxelClipmap, HybridGiProbeResidencyState, HybridGiProbeUpdateRequest,
    HybridGiRuntimeScenePrepareResources, HybridGiRuntimeState,
};

#[test]
fn hybrid_gi_runtime_state_registers_scene_cards_from_mesh_extract() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/b.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    assert_eq!(state.scene_card_ids(), vec![11, 22]);
    assert_eq!(state.scene_resident_page_ids(), vec![0, 1]);
    assert_eq!(state.scene_dirty_page_ids(), vec![0, 1]);
    assert_eq!(state.scene_invalidated_page_ids(), Vec::<u32>::new());
    assert_eq!(state.scene_feedback_card_ids(), Vec::<u32>::new());
    assert_eq!(state.scene_resident_clipmap_ids(), vec![0, 1]);
    assert_eq!(state.scene_dirty_clipmap_ids(), vec![0, 1]);
}

#[test]
fn hybrid_gi_runtime_state_builds_scene_clipmap_descriptors_from_mesh_bounds() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh_at(11, "res://materials/a.mat", Vec3::new(-4.0, 0.0, 0.0), 1.0),
            mesh_at(22, "res://materials/b.mat", Vec3::new(4.0, 0.0, 0.0), 1.0),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    assert_eq!(
        state.scene_clipmap_descriptors(),
        vec![(0, [0.0, 0.0, 0.0], 5.0), (1, [0.0, 0.0, 0.0], 10.0),]
    );
}

#[test]
fn hybrid_gi_runtime_state_invalidates_scene_clipmap_descriptors_when_scene_clears() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh_at(11, "res://materials/a.mat", Vec3::new(-4.0, 0.0, 0.0), 1.0),
            mesh_at(22, "res://materials/b.mat", Vec3::new(4.0, 0.0, 0.0), 1.0),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.register_scene_extract(Some(&extract), &[], &[], &[], &[]);

    assert_eq!(
        state.scene_clipmap_descriptors(),
        Vec::<(u32, [f32; 3], f32)>::new()
    );
    assert_eq!(state.scene_invalidated_clipmap_ids(), vec![0, 1]);
}

#[test]
fn hybrid_gi_runtime_state_snapshot_reports_scene_representation_counts() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/b.mat"),
            mesh(33, "res://materials/c.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    let snapshot = state.snapshot();
    assert_eq!(snapshot.cache_entry_count(), 0);
    assert_eq!(snapshot.resident_probe_count(), 0);
    assert_eq!(snapshot.pending_update_count(), 0);
    assert_eq!(snapshot.scheduled_trace_region_count(), 0);
    assert_eq!(snapshot.scene_card_count(), 3);
    assert_eq!(snapshot.surface_cache_resident_page_count(), 2);
    assert_eq!(snapshot.surface_cache_dirty_page_count(), 2);
    assert_eq!(snapshot.surface_cache_feedback_card_count(), 1);
    assert_eq!(snapshot.surface_cache_capture_slot_count(), 2);
    assert_eq!(snapshot.surface_cache_invalidated_page_count(), 0);
    assert_eq!(snapshot.voxel_resident_clipmap_count(), 2);
    assert_eq!(snapshot.voxel_dirty_clipmap_count(), 2);
    assert_eq!(snapshot.voxel_invalidated_clipmap_count(), 0);
}

#[test]
fn hybrid_gi_runtime_state_only_redirties_changed_cards_but_relights_all_resident_pages() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);
    let base_meshes = [
        mesh(11, "res://materials/a.mat"),
        mesh(22, "res://materials/b.mat"),
    ];

    state.register_scene_extract(
        Some(&extract),
        &base_meshes,
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/c.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    assert_eq!(state.scene_card_ids(), vec![11, 22]);
    assert_eq!(state.scene_resident_page_ids(), vec![0, 1]);
    assert_eq!(state.scene_dirty_page_ids(), vec![1]);
    assert_eq!(state.scene_invalidated_page_ids(), Vec::<u32>::new());
    assert_eq!(state.scene_dirty_clipmap_ids(), vec![0, 1]);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/c.mat"),
        ],
        &[directional_light(100, 2.0)],
        &[],
        &[],
    );

    assert_eq!(state.scene_resident_page_ids(), vec![0, 1]);
    assert_eq!(state.scene_dirty_page_ids(), vec![0, 1]);
    assert_eq!(state.scene_invalidated_page_ids(), Vec::<u32>::new());
    assert_eq!(state.scene_dirty_clipmap_ids(), vec![0, 1]);
}

#[test]
fn hybrid_gi_runtime_state_preserves_surface_cache_slots_across_dirtying_and_replacement() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/b.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    assert_eq!(state.scene_page_table_entries(), vec![(0, 0), (1, 1)]);
    assert_eq!(state.scene_capture_slot_entries(), vec![(0, 0), (1, 1)]);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/c.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    assert_eq!(state.scene_page_table_entries(), vec![(0, 0), (1, 1)]);
    assert_eq!(state.scene_capture_slot_entries(), vec![(1, 1)]);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(22, "res://materials/c.mat"),
            mesh(33, "res://materials/d.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    assert_eq!(state.scene_invalidated_page_ids(), vec![0]);
    assert_eq!(state.scene_page_table_entries(), vec![(1, 1), (0, 0)]);
    assert_eq!(state.scene_capture_slot_entries(), vec![(0, 0)]);
}

#[test]
fn hybrid_gi_runtime_state_persists_surface_cache_page_samples_across_clean_frame_and_invalidation()
{
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/b.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [11, 1, 0, 255]), (1, [22, 1, 0, 255])],
        vec![(0, [11, 2, 0, 255]), (1, [22, 2, 0, 255])],
    ));
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![
            (0, 11, 0, 0, [11, 1, 0, 255], [11, 2, 0, 255]),
            (1, 22, 1, 1, [22, 1, 0, 255], [22, 2, 0, 255]),
        ]
    );

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/b.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    assert_eq!(state.scene_dirty_page_ids(), Vec::<u32>::new());
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![
            (0, 11, 0, 0, [11, 1, 0, 255], [11, 2, 0, 255]),
            (1, 22, 1, 1, [22, 1, 0, 255], [22, 2, 0, 255]),
        ]
    );

    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        Vec::new(),
        Vec::new(),
    ));
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![
            (0, 11, 0, 0, [11, 1, 0, 255], [11, 2, 0, 255]),
            (1, 22, 1, 1, [22, 1, 0, 255], [22, 2, 0, 255]),
        ]
    );

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(22, "res://materials/b.mat"),
            mesh(33, "res://materials/c.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    assert_eq!(state.scene_invalidated_page_ids(), vec![0]);
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![(1, 22, 1, 1, [22, 1, 0, 255], [22, 2, 0, 255]),]
    );

    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [33, 1, 0, 255])],
        vec![(0, [33, 2, 0, 255])],
    ));
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![
            (1, 22, 1, 1, [22, 1, 0, 255], [22, 2, 0, 255]),
            (0, 33, 0, 0, [33, 1, 0, 255], [33, 2, 0, 255]),
        ]
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_atlas_only_surface_cache_page_samples_across_clean_frames() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(1, 1);

    state.register_scene_extract(
        Some(&extract),
        &[mesh(11, "res://materials/a.mat")],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [11, 1, 0, 255])],
        Vec::new(),
    ));
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![(0, 11, 0, 0, [11, 1, 0, 255], [0, 0, 0, 0])],
        "expected atlas-only scene-prepare truth to persist instead of being dropped just because the capture side is absent"
    );

    state.register_scene_extract(
        Some(&extract),
        &[mesh(11, "res://materials/a.mat")],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        Vec::new(),
        Vec::new(),
    ));
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![(0, 11, 0, 0, [11, 1, 0, 255], [0, 0, 0, 0])],
        "expected clean-frame runtime reuse to keep atlas-only persisted pages alive instead of requiring a capture-side sample to exist"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_capture_only_surface_cache_page_samples_across_clean_frames() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(1, 1);

    state.register_scene_extract(
        Some(&extract),
        &[mesh(11, "res://materials/a.mat")],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        Vec::new(),
        vec![(0, [11, 2, 0, 255])],
    ));
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![(0, 11, 0, 0, [0, 0, 0, 0], [11, 2, 0, 255])],
        "expected capture-only scene-prepare truth to persist instead of being dropped just because the atlas side is absent"
    );

    state.register_scene_extract(
        Some(&extract),
        &[mesh(11, "res://materials/a.mat")],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        Vec::new(),
        Vec::new(),
    ));
    assert_eq!(
        state.scene_surface_cache_page_contents(),
        vec![(0, 11, 0, 0, [0, 0, 0, 0], [11, 2, 0, 255])],
        "expected clean-frame runtime reuse to keep capture-only persisted pages alive instead of requiring an atlas-side sample to exist"
    );
}

#[test]
fn hybrid_gi_runtime_state_exposes_scene_card_capture_requests() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh_at(11, "res://materials/a.mat", Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, "res://materials/b.mat", Vec3::new(3.0, 0.0, 0.0), 1.0),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    assert_eq!(
        state.scene_card_capture_requests(),
        vec![
            (11, 0, 0, 0, [-1.0, 0.0, 0.0], 1.0),
            (22, 1, 1, 1, [3.0, 0.0, 0.0], 0.5),
        ]
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_scene_prepare_frame_from_scene_representation() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh_at(11, "res://materials/a.mat", Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, "res://materials/b.mat", Vec3::new(3.0, 0.0, 0.0), 1.0),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    let frame = state.build_scene_prepare_frame();
    assert_eq!(
        frame.card_capture_requests,
        vec![
            HybridGiPrepareCardCaptureRequest {
                card_id: 11,
                page_id: 0,
                atlas_slot_id: 0,
                capture_slot_id: 0,
                bounds_center: Vec3::new(-1.0, 0.0, 0.0),
                bounds_radius: 1.0,
            },
            HybridGiPrepareCardCaptureRequest {
                card_id: 22,
                page_id: 1,
                atlas_slot_id: 1,
                capture_slot_id: 1,
                bounds_center: Vec3::new(3.0, 0.0, 0.0),
                bounds_radius: 0.5,
            },
        ]
    );
    assert!(frame.surface_cache_page_contents.is_empty());
    assert_eq!(
        frame.voxel_clipmaps,
        vec![
            HybridGiPrepareVoxelClipmap {
                clipmap_id: 0,
                center: Vec3::new(0.75, 0.0, 0.0),
                half_extent: 3.0,
            },
            HybridGiPrepareVoxelClipmap {
                clipmap_id: 1,
                center: Vec3::new(0.75, 0.0, 0.0),
                half_extent: 6.0,
            },
        ]
    );
    assert_eq!(frame.voxel_cells.len(), 128);
}

#[test]
fn hybrid_gi_runtime_state_builds_scene_prepare_frame_with_persisted_surface_cache_page_contents_on_clean_frame(
) {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/b.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [11, 1, 0, 255]), (1, [22, 1, 0, 255])],
        vec![(0, [11, 2, 0, 255]), (1, [22, 2, 0, 255])],
    ));
    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/a.mat"),
            mesh(22, "res://materials/b.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    let frame = state.build_scene_prepare_frame();
    assert!(frame.card_capture_requests.is_empty());
    assert_eq!(
        frame.surface_cache_page_contents,
        vec![
            HybridGiPrepareSurfaceCachePageContent {
                page_id: 0,
                owner_card_id: 11,
                atlas_slot_id: 0,
                capture_slot_id: 0,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                atlas_sample_rgba: [11, 1, 0, 255],
                capture_sample_rgba: [11, 2, 0, 255],
            },
            HybridGiPrepareSurfaceCachePageContent {
                page_id: 1,
                owner_card_id: 22,
                atlas_slot_id: 1,
                capture_slot_id: 1,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                atlas_sample_rgba: [22, 1, 0, 255],
                capture_sample_rgba: [22, 2, 0, 255],
            },
        ]
    );
}

#[test]
fn hybrid_gi_runtime_state_uses_persisted_surface_cache_page_sample_for_clean_frame_voxel_radiance()
{
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(1, 1);
    let persisted_capture_rgba = [5, 200, 13, 255];
    let persisted_capture_rgb = [
        persisted_capture_rgba[0],
        persisted_capture_rgba[1],
        persisted_capture_rgba[2],
    ];

    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-persisted-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    let baseline_frame = state.build_scene_prepare_frame();
    let baseline_cells = baseline_frame
        .voxel_cells
        .iter()
        .filter(|cell| cell.occupancy_count > 0)
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        baseline_cells
            .iter()
            .any(|cell| cell.radiance_rgb != persisted_capture_rgb),
        "expected the placeholder runtime voxel radiance to differ before persisted page samples are applied; baseline_cells={baseline_cells:?}"
    );

    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, persisted_capture_rgba)],
        vec![(0, persisted_capture_rgba)],
    ));
    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-persisted-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    let frame = state.build_scene_prepare_frame();
    assert!(
        frame.card_capture_requests.is_empty(),
        "expected the second unchanged scene registration to become a clean frame"
    );
    let occupied_cells = frame
        .voxel_cells
        .into_iter()
        .filter(|cell| cell.occupancy_count > 0)
        .collect::<Vec<_>>();
    assert!(
        occupied_cells.iter().all(|cell| {
            cell.dominant_card_id == 11
                && cell.radiance_present
                && cell.radiance_rgb == persisted_capture_rgb
        }),
        "expected persisted clean-frame surface-cache page samples to become the runtime voxel radiance authority instead of leaving the old tint/direct-light placeholder truth in place; occupied_cells={occupied_cells:?}"
    );
}

#[test]
fn hybrid_gi_runtime_state_uses_atlas_only_surface_cache_page_sample_for_clean_frame_voxel_radiance(
) {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(1, 1);
    let persisted_atlas_rgba = [17, 33, 201, 255];
    let persisted_atlas_rgb = [
        persisted_atlas_rgba[0],
        persisted_atlas_rgba[1],
        persisted_atlas_rgba[2],
    ];

    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-atlas-only-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, persisted_atlas_rgba)],
        Vec::new(),
    ));
    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-atlas-only-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    let frame = state.build_scene_prepare_frame();
    assert!(
        frame.card_capture_requests.is_empty(),
        "expected the second unchanged scene registration to become a clean frame"
    );
    let occupied_cells = frame
        .voxel_cells
        .into_iter()
        .filter(|cell| cell.occupancy_count > 0)
        .collect::<Vec<_>>();
    assert!(
        occupied_cells.iter().all(|cell| {
            cell.dominant_card_id == 11
                && cell.radiance_present
                && cell.radiance_rgb == persisted_atlas_rgb
        }),
        "expected atlas-only persisted clean-frame surface-cache page samples to rehydrate runtime voxel radiance when capture truth is absent; occupied_cells={occupied_cells:?}"
    );
}

#[test]
fn hybrid_gi_runtime_state_prefers_capture_surface_cache_page_sample_over_atlas_for_clean_frame_voxel_radiance(
) {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(1, 1);
    let persisted_atlas_rgba = [17, 33, 201, 255];
    let persisted_capture_rgba = [5, 200, 13, 255];
    let persisted_capture_rgb = [
        persisted_capture_rgba[0],
        persisted_capture_rgba[1],
        persisted_capture_rgba[2],
    ];

    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-capture-preferred-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, persisted_atlas_rgba)],
        vec![(0, persisted_capture_rgba)],
    ));
    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-capture-preferred-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    let frame = state.build_scene_prepare_frame();
    assert!(
        frame.card_capture_requests.is_empty(),
        "expected the second unchanged scene registration to become a clean frame"
    );
    let occupied_cells = frame
        .voxel_cells
        .into_iter()
        .filter(|cell| cell.occupancy_count > 0)
        .collect::<Vec<_>>();
    assert!(
        occupied_cells.iter().all(|cell| {
            cell.dominant_card_id == 11
                && cell.radiance_present
                && cell.radiance_rgb == persisted_capture_rgb
        }),
        "expected capture-side persisted page truth to stay authoritative over atlas-side truth when both are present; occupied_cells={occupied_cells:?}"
    );
}

#[test]
fn hybrid_gi_runtime_state_excludes_dirty_pages_from_persisted_voxel_radiance_reuse() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(1, 1);
    let persisted_capture_rgba = [5, 200, 13, 255];
    let persisted_capture_rgb = [
        persisted_capture_rgba[0],
        persisted_capture_rgba[1],
        persisted_capture_rgba[2],
    ];

    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-dirty-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, persisted_capture_rgba)],
        vec![(0, persisted_capture_rgba)],
    ));

    state.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/runtime-voxel-dirty-page.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(100, 2.0)],
        &[],
        &[],
    );

    let frame = state.build_scene_prepare_frame();
    assert_eq!(state.scene_dirty_page_ids(), vec![0]);
    assert!(
        !frame.card_capture_requests.is_empty(),
        "expected the light change to keep the owner page dirty so the current frame still schedules a recapture"
    );
    let occupied_cells = frame
        .voxel_cells
        .into_iter()
        .filter(|cell| cell.occupancy_count > 0)
        .collect::<Vec<_>>();
    assert!(
        occupied_cells
            .iter()
            .all(|cell| cell.radiance_rgb != persisted_capture_rgb),
        "expected dirty owner pages to stop reusing stale persisted capture samples so the recapture frame can carry fresh voxel authority instead of last frame's clean-page color; occupied_cells={occupied_cells:?}"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_scene_prepare_voxel_cells_from_scene_representation() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 1);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh_at(11, "res://materials/a.mat", Vec3::new(-4.0, 0.0, 0.0), 1.0),
            mesh_at(22, "res://materials/b.mat", Vec3::new(4.0, 0.0, 0.0), 1.0),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    let frame = state.build_scene_prepare_frame();
    assert_eq!(
        frame.voxel_clipmaps,
        vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 0,
            center: Vec3::ZERO,
            half_extent: 5.0,
        }]
    );
    assert_eq!(frame.voxel_cells.len(), 64);
    assert_eq!(
        frame
            .voxel_cells
            .into_iter()
            .filter(|cell| cell.occupancy_count > 0)
            .collect::<Vec<_>>(),
        vec![
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 20,
                occupancy_count: 1,
                dominant_card_id: 11,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 23,
                occupancy_count: 1,
                dominant_card_id: 22,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 24,
                occupancy_count: 1,
                dominant_card_id: 11,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 27,
                occupancy_count: 1,
                dominant_card_id: 22,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 36,
                occupancy_count: 1,
                dominant_card_id: 11,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 39,
                occupancy_count: 1,
                dominant_card_id: 22,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 40,
                occupancy_count: 1,
                dominant_card_id: 11,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 0,
                cell_index: 43,
                occupancy_count: 1,
                dominant_card_id: 22,
                radiance_present: true,
                radiance_rgb: [99, 98, 97],
            },
        ]
    );
}

#[test]
fn hybrid_gi_runtime_state_scene_prepare_frame_only_keeps_changed_capture_requests() {
    let mut state = HybridGiRuntimeState::default();
    let extract = hybrid_gi_settings(2, 2);

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh_at(11, "res://materials/a.mat", Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, "res://materials/b.mat", Vec3::new(3.0, 0.0, 0.0), 1.0),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.register_scene_extract(
        Some(&extract),
        &[
            mesh_at(11, "res://materials/a.mat", Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, "res://materials/b.mat", Vec3::new(4.0, 0.0, 0.0), 1.5),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );

    assert_eq!(
        state.build_scene_prepare_frame().card_capture_requests,
        vec![HybridGiPrepareCardCaptureRequest {
            card_id: 22,
            page_id: 1,
            atlas_slot_id: 1,
            capture_slot_id: 1,
            bounds_center: Vec3::new(4.0, 0.0, 0.0),
            bounds_radius: 0.75,
        }]
    );
}

#[test]
fn hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule() {
    let mut state = HybridGiRuntimeState::default();
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
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(500), Some(1));
    assert_eq!(state.probe_slot(300), None);
    assert_eq!(
        state.probe_residency(200),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.probe_residency(500),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
    assert_eq!(pending_update_records(&state), vec![(300, 128, 9)]);
    assert_eq!(state.scheduled_trace_regions(), vec![40]);
    assert_eq!(state.evictable_probes(), vec![500]);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.cache_entry_count(), 2);
    assert_eq!(snapshot.resident_probe_count(), 2);
    assert_eq!(snapshot.pending_update_count(), 1);
    assert_eq!(snapshot.scheduled_trace_region_count(), 1);
}

#[test]
fn hybrid_gi_runtime_state_ignores_disabled_extract_payloads() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: false,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 2,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![probe(200, true, 64), probe(300, false, 128)],
        trace_regions: vec![trace_region(40)],
    };

    state.register_scene_extract(
        Some(&extract),
        &[mesh(11, "res://materials/disabled.mat")],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.ingest_plan(
        17,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let snapshot = state.snapshot();
    assert_eq!(snapshot.resident_probe_count(), 0);
    assert_eq!(snapshot.pending_update_count(), 0);
    assert_eq!(snapshot.scheduled_trace_region_count(), 0);
    assert_eq!(snapshot.scene_card_count(), 0);
    assert_eq!(snapshot.surface_cache_resident_page_count(), 0);
    assert_eq!(snapshot.voxel_resident_clipmap_count(), 0);
}

#[test]
fn hybrid_gi_runtime_state_ignores_legacy_payloads_when_scene_representation_is_budgeted() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 3,
        card_budget: 2,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 2,
        probes: vec![
            probe(200, true, 64),
            probe_with_parent(300, false, 128, 200),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_scene_extract(
        Some(&extract),
        &[
            mesh(11, "res://materials/scene-a.mat"),
            mesh(22, "res://materials/scene-b.mat"),
        ],
        &[directional_light(100, 1.0)],
        &[],
        &[],
    );
    state.ingest_plan(
        19,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40, 50],
            evictable_probe_ids: vec![200],
        },
    );

    assert_eq!(state.scene_card_ids(), vec![11, 22]);
    assert_eq!(
        state.probe_slot(200),
        None,
        "scene-representation budgets should keep old resident RenderHybridGiProbe payloads from allocating runtime cache slots"
    );
    assert_eq!(
        state.probe_residency(300),
        None,
        "scene-representation budgets should keep old requested RenderHybridGiProbe payloads out of runtime update queues"
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new()
    );
    assert_eq!(state.scheduled_trace_regions(), Vec::<u32>::new());
    let resolve_runtime = state.build_resolve_runtime();
    assert!(
        !resolve_runtime.has_probe_scene_data_entries(),
        "scene-representation budgets should keep old RenderHybridGiProbe geometry out of resolve runtime scene data"
    );
    assert!(
        !resolve_runtime.has_trace_region_scene_data_entries(),
        "scene-representation budgets should keep old RenderHybridGiTraceRegion geometry out of resolve runtime scene data"
    );

    let snapshot = state.snapshot();
    assert_eq!(snapshot.resident_probe_count(), 0);
    assert_eq!(snapshot.pending_update_count(), 0);
    assert_eq!(snapshot.scheduled_trace_region_count(), 0);
    assert_eq!(snapshot.scene_card_count(), 2);
    assert_eq!(snapshot.surface_cache_resident_page_count(), 2);
    assert_eq!(snapshot.voxel_resident_clipmap_count(), 1);
}

#[test]
fn hybrid_gi_runtime_state_ignores_plan_probe_ids_without_live_payloads() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: vec![probe(200, true, 64), probe(300, false, 128)],
        trace_regions: Vec::new(),
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        11,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 9_999],
            requested_probe_ids: vec![300, 8_888],
            dirty_requested_probe_ids: vec![300, 8_888],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: vec![200, 9_999],
        },
    );

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(
        state.probe_slot(9_999),
        None,
        "expected stale plan resident ids without a live RenderHybridGiProbe payload not to allocate runtime cache slots"
    );
    assert_eq!(
        state.probe_residency(8_888),
        None,
        "expected stale dirty/requested plan ids without a live RenderHybridGiProbe payload not to enter the runtime update queue"
    );
    assert_eq!(pending_update_records(&state), vec![(300, 128, 11)]);
    assert_eq!(state.evictable_probes(), vec![200]);
}

#[test]
fn hybrid_gi_runtime_state_filters_scheduled_trace_region_ids_without_live_payloads() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 2,
        probes: vec![probe(200, true, 64)],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        12,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![9_999, 40, 40, 8_888, 50],
            evictable_probe_ids: Vec::new(),
        },
    );

    assert_eq!(
        state.scheduled_trace_regions(),
        vec![40, 50],
        "expected runtime trace scheduling to keep only deduplicated ids backed by current RenderHybridGiTraceRegion payloads"
    );
}

#[test]
fn hybrid_gi_runtime_state_ignores_gpu_completion_probe_payloads_without_live_probe_payloads() {
    let mut state = HybridGiRuntimeState::default();
    let initial_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 0,
        probes: vec![probe(200, true, 64)],
        trace_regions: Vec::new(),
    };
    let later_extract = RenderHybridGiExtract {
        probes: vec![probe(999, false, 128)],
        ..initial_extract.clone()
    };

    state.register_extract(Some(&initial_extract));
    state.complete_gpu_updates([], [], &[(999, [8, 16, 32])], &[(999, [240, 96, 48])], &[]);
    state.register_extract(Some(&later_extract));
    state.ingest_plan(
        13,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![999],
            dirty_requested_probe_ids: vec![999],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let runtime = state.build_resolve_runtime();

    assert_eq!(
        runtime.hierarchy_rt_lighting(999),
        None,
        "expected stale GPU completion payloads for ids without a live RenderHybridGiProbe payload not to be reused when the same id appears in a later extract"
    );
}

#[test]
fn hybrid_gi_runtime_state_ignores_parent_probe_ids_without_live_payloads() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 0,
        probes: vec![probe_with_parent(200, false, 128, 9_999)],
        trace_regions: Vec::new(),
    };

    state.register_extract(Some(&extract));
    state.complete_gpu_updates([], [], &[], &[(200, [240, 96, 48])], &[]);
    state.ingest_plan(
        14,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let runtime = state.build_resolve_runtime();

    assert_eq!(
        runtime.parent_probe_id(200),
        None,
        "expected runtime parent topology to drop parent ids that have no live RenderHybridGiProbe payload"
    );
    assert!(
        runtime
            .hierarchy_rt_lighting(200)
            .map(|source| source[0] > 0.8 && source[1] > 0.25 && source[2] > 0.1)
            .unwrap_or(false),
        "expected a dangling legacy parent id not to block standalone direct RT fallback for the live child probe"
    );
}

#[test]
fn hybrid_gi_runtime_state_ignores_gpu_cache_entries_without_live_probe_payloads() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 0,
        probes: vec![probe(200, false, 64)],
        trace_regions: Vec::new(),
    };

    state.register_extract(Some(&extract));
    state.apply_gpu_cache_entries(&[(9_999, 3)]);

    assert_eq!(
        state.probe_slot(9_999),
        None,
        "expected stale GPU cache entries without a live RenderHybridGiProbe payload not to allocate runtime cache slots"
    );
    assert_eq!(
        state.snapshot().cache_entry_count(),
        0,
        "expected stale GPU cache entries to be discarded before residency accounting"
    );
}

#[test]
fn hybrid_gi_runtime_state_breaks_legacy_probe_parent_cycles_on_registration() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: vec![
            probe_with_parent(100, false, 64, 200),
            probe_with_parent(200, false, 96, 100),
        ],
        trace_regions: Vec::new(),
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        15,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![100, 200],
            dirty_requested_probe_ids: vec![100, 200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame();
    let runtime = state.build_resolve_runtime();

    assert_eq!(
        runtime.parent_probe_count(),
        1,
        "expected runtime registration to break cyclic legacy RenderHybridGiProbe parent topology instead of exporting both cycle edges"
    );
    assert!(
        !prepare.pending_updates.is_empty(),
        "expected cyclic legacy parent topology not to make every pending probe wait on another pending ancestor"
    );
}

#[test]
fn hybrid_gi_runtime_state_rebuilds_parent_topology_from_current_extract_before_cycle_check() {
    let mut state = HybridGiRuntimeState::default();
    let initial_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: vec![probe_with_parent(100, true, 64, 200), probe(200, true, 96)],
        trace_regions: Vec::new(),
    };
    let updated_extract = RenderHybridGiExtract {
        probes: vec![probe_with_parent(200, true, 96, 100), probe(100, true, 64)],
        ..initial_extract.clone()
    };

    state.register_extract(Some(&initial_extract));
    state.register_extract(Some(&updated_extract));

    let runtime = state.build_resolve_runtime();

    assert_eq!(
        runtime.parent_probe_id(200),
        Some(100),
        "expected current legacy RenderHybridGiProbe payloads to rebuild runtime parent topology before cycle pruning, instead of letting a stale previous-frame parent edge suppress the new valid parent"
    );
    assert_eq!(
        runtime.parent_probe_id(100),
        None,
        "expected parent topology registration to remove stale previous-frame edges for probes whose current payload no longer declares a parent"
    );
}

#[test]
fn hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots() {
    let mut state = HybridGiRuntimeState::default();
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
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );
    state.ingest_plan(
        10,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![50],
            evictable_probe_ids: vec![500],
        },
    );

    assert_eq!(state.pending_updates().len(), 1);
    state.apply_evictions([500]);
    state.fulfill_updates([300]);

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new()
    );
    assert_eq!(state.scheduled_trace_regions(), vec![50]);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.cache_entry_count(), 2);
    assert_eq!(snapshot.resident_probe_count(), 2);
    assert_eq!(snapshot.pending_update_count(), 0);
    assert_eq!(snapshot.scheduled_trace_region_count(), 1);
}

#[test]
fn hybrid_gi_runtime_state_builds_prepare_frame_without_host_bootstrap_irradiance() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 2,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 64,
                    irradiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareProbe {
                    probe_id: 500,
                    slot: 1,
                    ray_budget: 32,
                    irradiance_rgb: [0, 0, 0],
                },
            ],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 128,
                generation: 9,
            }],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_consumes_feedback_and_promotes_requested_probes() {
    let mut state = HybridGiRuntimeState::default();
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
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    state.consume_feedback(&VisibilityHybridGiFeedback {
        active_probe_ids: vec![300, 200],
        requested_probe_ids: vec![300],
        scheduled_trace_region_ids: vec![50],
        evictable_probe_ids: vec![500],
    });

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new()
    );
    assert_eq!(state.scheduled_trace_regions(), vec![50]);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.cache_entry_count(), 2);
    assert_eq!(snapshot.resident_probe_count(), 2);
    assert_eq!(snapshot.pending_update_count(), 0);
    assert_eq!(snapshot.scheduled_trace_region_count(), 1);
}

#[test]
fn hybrid_gi_runtime_state_leaves_updates_pending_without_evictable_budget() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![probe(200, true, 64), probe(300, false, 128)],
        trace_regions: vec![trace_region(40), trace_region(60)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    state.consume_feedback(&VisibilityHybridGiFeedback {
        active_probe_ids: vec![300, 200],
        requested_probe_ids: vec![300],
        scheduled_trace_region_ids: vec![60],
        evictable_probe_ids: Vec::new(),
    });

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
    assert_eq!(pending_update_records(&state), vec![(300, 128, 9)]);
    assert_eq!(state.scheduled_trace_regions(), vec![60]);
}

#[test]
fn hybrid_gi_runtime_state_does_not_inflate_probe_budget_from_duplicate_resident_probe_payloads() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 64),
            probe(200, true, 64),
            probe(300, false, 128),
        ],
        trace_regions: vec![trace_region(40), trace_region(60)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    state.consume_feedback(&VisibilityHybridGiFeedback {
        active_probe_ids: vec![300, 200],
        requested_probe_ids: vec![300],
        scheduled_trace_region_ids: vec![60],
        evictable_probe_ids: Vec::new(),
    });

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(
        state.probe_slot(300),
        None,
        "expected duplicate legacy resident RenderHybridGiProbe payloads not to inflate runtime probe budget and promote a pending probe without an evictable slot"
    );
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
}

#[test]
fn hybrid_gi_runtime_state_ignores_later_duplicate_resident_payload_for_probe_budget() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 0,
        tracing_budget: 0,
        probes: vec![
            probe(200, false, 64),
            probe(200, true, 64),
            probe(300, false, 128),
        ],
        trace_regions: Vec::new(),
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        10,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    state.complete_gpu_updates([300], [], &[], &[], &[]);

    assert_eq!(
        state.probe_slot(300),
        None,
        "expected a later duplicate resident RenderHybridGiProbe payload not to inflate runtime probe budget after a first non-resident payload wins"
    );
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
}

#[test]
fn hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule() {
    let mut state = HybridGiRuntimeState::default();
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
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    state.complete_gpu_updates(
        [300],
        [50],
        &[
            (200, [121, 133, 145]),
            (300, [210, 164, 118]),
            (500, [89, 101, 113]),
        ],
        &[
            (200, [24, 32, 40]),
            (300, [176, 88, 48]),
            (500, [16, 20, 24]),
        ],
        &[500],
    );

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new()
    );
    assert_eq!(state.scheduled_trace_regions(), vec![50]);
    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 64,
                    irradiance_rgb: [121, 133, 145],
                },
                HybridGiPrepareProbe {
                    probe_id: 300,
                    slot: 1,
                    ray_budget: 128,
                    irradiance_rgb: [210, 164, 118],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![50],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_applies_gpu_cache_snapshot_as_residency_truth() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
            probe(600, false, 48),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300, 600],
            dirty_requested_probe_ids: vec![300, 600],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    state.apply_gpu_cache_entries(&[(200, 0), (300, 1)]);

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(state.probe_slot(600), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.probe_residency(600),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
    assert_eq!(pending_update_records(&state), vec![(600, 48, 9)]);
    assert!(state.evictable_probes().is_empty());

    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 64,
                    irradiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareProbe {
                    probe_id: 300,
                    slot: 1,
                    ray_budget: 128,
                    irradiance_rgb: [0, 0, 0],
                },
            ],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 600,
                ray_budget: 48,
                generation: 9,
            }],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_ignores_duplicate_gpu_cache_entries_after_first_unique_probe() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 64),
            probe(200, false, 128),
            probe(300, false, 96),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        14,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 500],
            requested_probe_ids: vec![200, 300],
            dirty_requested_probe_ids: vec![200, 300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    state.apply_gpu_cache_entries(&[(100, 0), (200, 2), (200, 1), (300, 1)]);

    assert_eq!(state.probe_slot(100), Some(0));
    assert_eq!(
        state.probe_slot(200),
        Some(2),
        "expected Hybrid GI runtime cache truth to keep the first unique cache entry for probe 200 instead of letting a later duplicate cache entry migrate the already-confirmed probe into a new slot"
    );
    assert_eq!(
        state.probe_slot(300),
        Some(1),
        "expected later unique cache entries to keep their authoritative slot after duplicate entries for an earlier probe id are ignored"
    );
    assert_eq!(
        state.probe_slot(500),
        None,
        "expected the later unique cache entry to recycle the stale resident probe instead of being blocked by a duplicate cache entry for an already-confirmed probe"
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new(),
        "expected duplicate GPU cache entries to stop leaving later unique pending probes stranded in the runtime update queue"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_processing_later_unique_feedback_probe_completions_after_leading_duplicate_requests(
) {
    let mut state = HybridGiRuntimeState::default();
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
            probe(100, true, 64),
            probe(200, false, 128),
            probe(300, false, 96),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        15,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200, 300],
            dirty_requested_probe_ids: vec![200, 300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![100],
        },
    );

    state.consume_feedback(&VisibilityHybridGiFeedback {
        active_probe_ids: vec![200, 300],
        requested_probe_ids: vec![200, 200, 300],
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: vec![100],
    });

    assert_eq!(
        state.probe_slot(100),
        None,
        "expected Hybrid GI feedback completion to spend the one eviction on the later unique probe instead of wasting it on a duplicate request id"
    );
    assert_eq!(state.probe_slot(200), Some(1));
    assert_eq!(
        state.probe_slot(300),
        Some(0),
        "expected feedback-driven probe completion to keep processing later unique requested probes after leading duplicate ids instead of truncating at probe_budget before deduplication"
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new(),
        "expected duplicate feedback request ids to stop leaving later unique pending probes stranded in the runtime queue"
    );
}

#[test]
fn hybrid_gi_runtime_state_drops_stale_scene_probes_and_pending_updates_when_extract_shrinks() {
    let mut state = HybridGiRuntimeState::default();
    let initial_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 96),
            probe(200, false, 64),
            probe(300, true, 48),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&initial_extract));
    state.ingest_plan(
        12,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 300],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![300],
        },
    );
    state.complete_gpu_updates(
        [200],
        [40],
        &[
            (100, [192, 144, 96]),
            (200, [96, 144, 192]),
            (300, [48, 64, 80]),
        ],
        &[
            (100, [208, 96, 48]),
            (200, [128, 144, 160]),
            (300, [24, 32, 40]),
        ],
        &[300],
    );

    let shrunk_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![probe(100, true, 96)],
        trace_regions: vec![trace_region(50)],
    };

    state.register_extract(Some(&shrunk_extract));

    assert_eq!(state.probe_slot(100), Some(0));
    assert_eq!(state.probe_slot(200), None);
    assert_eq!(state.probe_slot(300), None);
    assert_eq!(
        state.probe_residency(200),
        None,
        "expected runtime host state to purge removed scene probes instead of keeping stale pending/resident entries alive"
    );
    assert_eq!(
        state.probe_residency(300),
        None,
        "expected runtime host state to evict removed resident probes when the extract no longer contains their lineage"
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new(),
        "expected removed scene probes to drop out of the pending update queue"
    );
    assert_eq!(state.scheduled_trace_regions(), Vec::<u32>::new());
    assert_eq!(state.evictable_probes(), Vec::<u32>::new());
    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [192, 144, 96],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_withholds_descendant_probe_updates_while_ancestor_update_remains_pending(
) {
    let mut state = HybridGiRuntimeState::default();
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
            probe(100, true, 96),
            probe_with_parent(200, false, 64, 100),
            probe_with_parent(300, false, 48, 200),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        21,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200, 300],
            dirty_requested_probe_ids: vec![200, 300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let blocked_prepare = state.build_prepare_frame();
    assert_eq!(
        blocked_prepare.pending_updates,
        vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 64,
            generation: 21,
        }],
        "expected runtime prepare to withhold descendant probe updates while the missing ancestor probe update is still pending so hierarchy-aware GPU completion does not bypass the collapsed lineage"
    );

    state.complete_gpu_updates(
        [200],
        [40],
        &[(100, [48, 48, 48]), (200, [96, 128, 160])],
        &[(100, [160, 96, 48]), (200, [128, 144, 160])],
        &[],
    );

    let unblocked_prepare = state.build_prepare_frame();
    assert_eq!(
        unblocked_prepare.pending_updates,
        vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 48,
            generation: 21,
        }],
        "expected descendant probe updates to re-enter the prepare queue once their pending ancestor probe becomes resident"
    );
}

#[test]
fn hybrid_gi_runtime_state_prioritizes_pending_ancestor_probes_that_reconnect_hot_descendants() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 96),
            probe_with_parent(200, false, 72, 100),
            probe_with_parent(400, true, 56, 200),
            probe(800, false, 48),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        31,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 400],
            requested_probe_ids: vec![800, 200],
            dirty_requested_probe_ids: vec![800, 200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame();
    assert_eq!(
        prepare.pending_updates,
        vec![
            HybridGiPrepareUpdateRequest {
                probe_id: 200,
                ray_budget: 72,
                generation: 31,
            },
            HybridGiPrepareUpdateRequest {
                probe_id: 800,
                ray_budget: 48,
                generation: 31,
            },
        ],
        "expected runtime prepare to prioritize the missing ancestor probe that reconnects already-resident descendant history before unrelated pending probe updates so the hierarchy-aware radiance-cache path converges instead of thrashing hot descendants"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_resolve_runtime_from_gpu_trace_lighting_history() {
    let mut state = HybridGiRuntimeState::default();
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
            probe(200, true, 96),
            probe(300, false, 64),
            probe(500, true, 48),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        13,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );
    state.complete_gpu_updates(
        [300],
        [40],
        &[
            (200, [112, 128, 144]),
            (300, [160, 144, 128]),
            (500, [72, 88, 104]),
        ],
        &[
            (200, [208, 96, 48]),
            (300, [176, 104, 64]),
            (500, [32, 48, 80]),
        ],
        &[500],
    );

    let resolve_runtime = state.build_resolve_runtime();
    assert_eq!(
        resolve_runtime.probe_rt_lighting_rgb(200),
        Some([208, 96, 48])
    );
    assert_eq!(
        resolve_runtime.probe_rt_lighting_rgb(300),
        Some([176, 104, 64])
    );
    assert_eq!(
        resolve_runtime.probe_rt_lighting_rgb(500),
        None,
        "expected runtime-host resolve inputs to retain GPU-produced per-probe trace-lighting truth for resident probes so post-process resolve can consume GPU source instead of recomputing all RT tint encode-side"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_hierarchy_resolve_runtime_from_resident_lineage_history() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 4,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 96),
            probe_with_parent(200, true, 80, 100),
            probe_with_parent(300, true, 64, 200),
            probe_with_parent(400, true, 48, 300),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.complete_gpu_updates(
        [100, 200, 300, 400],
        [40],
        &[
            (100, [220, 180, 120]),
            (200, [180, 144, 112]),
            (300, [144, 112, 88]),
            (400, [96, 96, 96]),
        ],
        &[
            (100, [240, 80, 32]),
            (200, [208, 112, 48]),
            (300, [176, 96, 64]),
            (400, [96, 96, 96]),
        ],
        &[],
    );

    let runtime = state.build_resolve_runtime();
    assert!(
        runtime
            .hierarchy_resolve_weight(400)
            .is_some_and(|weight| weight > 1.4),
        "expected runtime-host resolve inputs to carry hierarchy-aware resolve weight for deeper resident probe lineages instead of leaving that weighting exclusively to encode-time hierarchy scans"
    );
    assert!(
        runtime
            .hierarchy_irradiance(400)
            .is_some_and(|encoded| encoded[3] > 0.1 && encoded[0] > encoded[2]),
        "expected runtime-host resolve inputs to carry farther-ancestor irradiance continuation for deeper resident probe lineages instead of recomputing it only from current-frame prepare ancestry"
    );
    assert!(
        runtime
            .hierarchy_rt_lighting(400)
            .is_some_and(|encoded| encoded[3] > 0.1 && encoded[0] > encoded[2]),
        "expected runtime-host resolve inputs to carry ancestor-derived RT-lighting continuation for deeper resident probe lineages instead of leaving that continuation exclusively to encode-time hierarchy scans"
    );
}

#[test]
fn hybrid_gi_runtime_state_prioritizes_pending_probe_with_stronger_lineage_trace_support() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 3,
        tracing_budget: 2,
        probes: vec![
            probe_at(100, true, 96, Vec3::new(-0.9, 0.0, 0.0)),
            probe_with_parent_at(200, false, 72, 100, Vec3::new(0.0, 0.0, 0.0)),
            probe_at(300, false, 80, Vec3::new(0.55, 0.0, 0.0)),
        ],
        trace_regions: vec![
            trace_region_at(40, Vec3::ZERO),
            trace_region_at(50, Vec3::new(-0.9, 0.0, 0.0)),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        56,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![300, 200],
            dirty_requested_probe_ids: vec![300, 200],
            scheduled_trace_region_ids: vec![40, 50],
            evictable_probe_ids: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame();
    assert_eq!(
        prepare.pending_updates,
        vec![
            HybridGiPrepareUpdateRequest {
                probe_id: 200,
                ray_budget: 72,
                generation: 56,
            },
            HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 80,
                generation: 56,
            },
        ],
        "expected runtime prepare to prioritize the pending probe whose nonresident lineage stays aligned with the scheduled trace hierarchy instead of only sorting by flat descendant counts or shallow depth"
    );
}

#[test]
fn hybrid_gi_runtime_state_strengthens_resolve_weight_when_trace_schedule_supports_lineage() {
    let hierarchical_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe_at(100, true, 96, Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, true, 96, 100, Vec3::ZERO),
        ],
        trace_regions: vec![trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0))],
    };
    let flat_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        trace_regions: vec![trace_region_at(40, Vec3::new(1.6, 0.0, 0.0))],
        ..hierarchical_extract.clone()
    };

    let mut hierarchical = HybridGiRuntimeState::default();
    hierarchical.register_extract(Some(&hierarchical_extract));
    hierarchical.ingest_plan(
        57,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&flat_extract));
    flat.ingest_plan(
        57,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let hierarchical_weight = hierarchical
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("hierarchical resolve weight");
    let flat_weight = flat
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("flat resolve weight");

    assert!(
        hierarchical_weight > flat_weight + 0.05,
        "expected runtime-host resolve weighting to strengthen when the current scheduled trace work still supports the probe lineage instead of leaving that scene-driven weighting entirely outside runtime resolve inputs; flat_weight={flat_weight:.3}, hierarchical_weight={hierarchical_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_strengthens_parent_resolve_weight_when_descendant_trace_schedule_supports_merge_back(
) {
    let supported_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            RenderHybridGiProbe {
                radius: 0.08,
                ..probe_at(100, true, 96, Vec3::ZERO)
            },
            RenderHybridGiProbe {
                radius: 0.08,
                ..probe_with_parent_at(200, true, 96, 100, Vec3::new(-0.8, 0.0, 0.0))
            },
        ],
        trace_regions: vec![RenderHybridGiTraceRegion {
            bounds_radius: 0.05,
            ..trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0))
        }],
    };
    let flat_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        trace_regions: vec![trace_region_at(40, Vec3::new(2.4, 0.0, 0.0))],
        ..supported_extract.clone()
    };

    let mut supported = HybridGiRuntimeState::default();
    supported.register_extract(Some(&supported_extract));
    supported.ingest_plan(
        83,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&flat_extract));
    flat.ingest_plan(
        83,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let supported_weight = supported
        .build_resolve_runtime()
        .hierarchy_resolve_weight(100)
        .expect("supported parent resolve weight");
    let flat_weight = flat
        .build_resolve_runtime()
        .hierarchy_resolve_weight(100)
        .expect("flat parent resolve weight");

    assert!(
        supported_weight > flat_weight + 0.04,
        "expected merge-back parent resolve weighting to strengthen when a scheduled trace region still supports a resident child probe, instead of only counting scene-driven support that lands directly on the parent probe; flat_weight={flat_weight:.3}, supported_weight={supported_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_parent_descendant_rt_continuation_after_child_trace_schedule_clears(
) {
    let supported_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            RenderHybridGiProbe {
                radius: 0.08,
                ..probe_at(100, true, 96, Vec3::ZERO)
            },
            RenderHybridGiProbe {
                radius: 0.08,
                ..probe_with_parent_at(200, true, 96, 100, Vec3::new(-0.8, 0.0, 0.0))
            },
        ],
        trace_regions: vec![RenderHybridGiTraceRegion {
            bounds_radius: 0.05,
            ..trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0))
        }],
    };
    let flat_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        trace_regions: vec![trace_region_at(40, Vec3::new(2.4, 0.0, 0.0))],
        ..supported_extract.clone()
    };

    let mut supported_warm = HybridGiRuntimeState::default();
    supported_warm.register_extract(Some(&supported_extract));
    supported_warm.ingest_plan(
        84,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    supported_warm.complete_gpu_updates([], [40], &[], &[(200, [240, 96, 48])], &[]);
    supported_warm.ingest_plan(
        85,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut supported_cool = HybridGiRuntimeState::default();
    supported_cool.register_extract(Some(&supported_extract));
    supported_cool.ingest_plan(
        84,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    supported_cool.complete_gpu_updates([], [40], &[], &[(200, [48, 96, 240])], &[]);
    supported_cool.ingest_plan(
        85,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&flat_extract));
    flat.ingest_plan(
        84,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    flat.complete_gpu_updates([], [40], &[], &[(200, [240, 96, 48])], &[]);
    flat.ingest_plan(
        85,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let supported_warm_runtime = supported_warm.build_resolve_runtime();
    let supported_cool_runtime = supported_cool.build_resolve_runtime();
    let flat_runtime = flat.build_resolve_runtime();
    let supported_warm_parent_rt = supported_warm_runtime
        .hierarchy_rt_lighting(100)
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let supported_cool_parent_rt = supported_cool_runtime
        .hierarchy_rt_lighting(100)
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let flat_parent_rt = flat_runtime
        .hierarchy_rt_lighting(100)
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);

    assert!(
        supported_warm_parent_rt[3] > flat_parent_rt[3] + 0.015,
        "expected merge-back parent runtime resolve inputs to keep descendant RT-lighting continuation for one more frame after the child trace schedule clears, instead of dropping the parent back to the same flat no-support path; flat_parent_rt={flat_parent_rt:?}, supported_warm_parent_rt={supported_warm_parent_rt:?}"
    );
    assert!(
        supported_warm_parent_rt[0] > supported_cool_parent_rt[0] + 0.2,
        "expected merge-back parent runtime resolve inputs to retain warm descendant RT-lighting color after the child trace schedule clears, instead of flattening warm/cool child history into the same parent continuation; supported_warm_parent_rt={supported_warm_parent_rt:?}, supported_cool_parent_rt={supported_cool_parent_rt:?}"
    );
    assert!(
        supported_cool_parent_rt[2] > supported_warm_parent_rt[2] + 0.2,
        "expected merge-back parent runtime resolve inputs to retain cool descendant RT-lighting color after the child trace schedule clears, instead of flattening warm/cool child history into the same parent continuation; supported_warm_parent_rt={supported_warm_parent_rt:?}, supported_cool_parent_rt={supported_cool_parent_rt:?}"
    );
}

#[test]
fn hybrid_gi_runtime_state_deduplicates_scheduled_trace_region_ids_before_lineage_support_scoring()
{
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
            probe_at(100, true, 96, Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, false, 80, 100, Vec3::ZERO),
        ],
        trace_regions: vec![trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0))],
    };

    let mut unique = HybridGiRuntimeState::default();
    unique.register_extract(Some(&extract));
    unique.ingest_plan(
        71,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut duplicated = HybridGiRuntimeState::default();
    duplicated.register_extract(Some(&extract));
    duplicated.ingest_plan(
        71,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40, 40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let unique_weight = unique
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("unique trace schedule weight");
    let duplicated_weight = duplicated
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("duplicated trace schedule weight");

    assert!(
        (duplicated_weight - unique_weight).abs() <= 0.001,
        "expected duplicate scheduled trace-region ids to be ignored before scene-driven lineage support scoring instead of artificially inflating runtime resolve weight; unique_weight={unique_weight:.3}, duplicated_weight={duplicated_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_first_duplicate_trace_region_payload_for_lineage_support() {
    let far_region = trace_region_at(40, Vec3::new(2.4, 0.0, 0.0));
    let near_duplicate_region = trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0));
    let base_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe_at(100, true, 96, Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, false, 80, 100, Vec3::ZERO),
        ],
        trace_regions: vec![far_region.clone()],
    };
    let duplicate_extract = RenderHybridGiExtract {
        trace_regions: vec![far_region, near_duplicate_region],
        ..base_extract.clone()
    };

    let mut first_payload = HybridGiRuntimeState::default();
    first_payload.register_extract(Some(&base_extract));
    first_payload.ingest_plan(
        71,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut duplicate_payload = HybridGiRuntimeState::default();
    duplicate_payload.register_extract(Some(&duplicate_extract));
    duplicate_payload.ingest_plan(
        71,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let first_payload_weight = first_payload
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("first trace payload weight");
    let duplicate_payload_weight = duplicate_payload
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("duplicate trace payload weight");

    assert!(
        (duplicate_payload_weight - first_payload_weight).abs() <= 0.001,
        "expected duplicate legacy RenderHybridGiTraceRegion payloads with the same id to keep the first live payload for runtime lineage support, matching renderer scheduled trace lookup instead of letting a later duplicate override it; first_payload_weight={first_payload_weight:.3}, duplicate_payload_weight={duplicate_payload_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_first_duplicate_probe_payload_for_parent_topology() {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 3,
        tracing_budget: 0,
        probes: vec![
            probe_at(100, true, 96, Vec3::new(-1.0, 0.0, 0.0)),
            probe_at(300, true, 96, Vec3::new(1.0, 0.0, 0.0)),
            probe_with_parent_at(200, true, 80, 100, Vec3::ZERO),
            probe_with_parent_at(200, true, 80, 300, Vec3::ZERO),
        ],
        trace_regions: Vec::new(),
    };
    let mut state = HybridGiRuntimeState::default();

    state.register_extract(Some(&extract));
    let runtime = state.build_resolve_runtime();

    assert_eq!(
        runtime.parent_probe_id(200),
        Some(100),
        "expected duplicate legacy RenderHybridGiProbe payloads with the same id to keep the first parent topology, matching renderer source-probe lookup instead of letting a later duplicate override it"
    );
}

#[test]
fn hybrid_gi_runtime_state_limits_lineage_trace_support_to_live_payload_region_budget() {
    const MAX_TEST_TRACE_REGIONS: usize = 16;

    let filler_region_ids = (0..MAX_TEST_TRACE_REGIONS)
        .map(|index| 10_000 + index as u32)
        .collect::<Vec<_>>();
    let tail_region_id = 40;
    let mut scheduled_trace_region_ids = filler_region_ids.clone();
    scheduled_trace_region_ids.push(tail_region_id);
    let mut trace_regions = filler_region_ids
        .iter()
        .copied()
        .map(|region_id| trace_region_at(region_id, Vec3::new(10.0, 0.0, 0.0)))
        .collect::<Vec<_>>();
    trace_regions.push(trace_region_at(tail_region_id, Vec3::new(-0.8, 0.0, 0.0)));

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
            probe_at(100, true, 96, Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, false, 80, 100, Vec3::ZERO),
        ],
        trace_regions,
    };

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&extract));
    flat.ingest_plan(
        71,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: filler_region_ids,
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut tail_supported = HybridGiRuntimeState::default();
    tail_supported.register_extract(Some(&extract));
    tail_supported.ingest_plan(
        71,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids,
            evictable_probe_ids: Vec::new(),
        },
    );

    let flat_weight = flat
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("flat trace schedule weight");
    let tail_supported_weight = tail_supported
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("tail trace schedule weight");

    assert!(
        (tail_supported_weight - flat_weight).abs() <= 0.001,
        "expected runtime lineage trace support to ignore live trace payloads beyond the same region budget used by GPU trace encoding, instead of letting a 17th legacy scheduled payload strengthen runtime resolve; flat_weight={flat_weight:.3}, tail_supported_weight={tail_supported_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_for_pending_probe_order_after_schedule_clears(
) {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe_at(100, true, 96, Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, false, 72, 100, Vec3::ZERO),
            probe_at(300, false, 80, Vec3::new(0.85, 0.0, 0.0)),
        ],
        trace_regions: vec![trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0))],
    };
    let mut state = HybridGiRuntimeState::default();
    state.register_extract(Some(&extract));
    state.ingest_plan(
        58,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200, 300],
            dirty_requested_probe_ids: vec![200, 300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    state.ingest_plan(
        59,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200, 300],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame();
    assert_eq!(
        prepare.pending_updates,
        vec![
            HybridGiPrepareUpdateRequest {
                probe_id: 200,
                ray_budget: 72,
                generation: 58,
            },
            HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 80,
                generation: 58,
            },
        ],
        "expected runtime prepare to keep prioritizing the hierarchy-supported child probe for one more frame after the trace schedule clears, instead of immediately falling back to flat root-first ordering and losing scene-driven probe-request continuation"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_in_resolve_runtime_after_schedule_clears(
) {
    let supported_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe_at(100, true, 96, Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, true, 96, 100, Vec3::ZERO),
        ],
        trace_regions: vec![trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0))],
    };
    let flat_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        trace_regions: vec![trace_region_at(40, Vec3::new(2.4, 0.0, 0.0))],
        ..supported_extract.clone()
    };

    let mut supported = HybridGiRuntimeState::default();
    supported.register_extract(Some(&supported_extract));
    supported.ingest_plan(
        60,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    supported.complete_gpu_updates(
        [],
        [40],
        &[(100, [144, 120, 96]), (200, [120, 120, 120])],
        &[(100, [240, 96, 48]), (200, [176, 112, 72])],
        &[],
    );
    supported.ingest_plan(
        61,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&flat_extract));
    flat.ingest_plan(
        60,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    flat.complete_gpu_updates(
        [],
        [40],
        &[(100, [144, 120, 96]), (200, [120, 120, 120])],
        &[(100, [240, 96, 48]), (200, [176, 112, 72])],
        &[],
    );
    flat.ingest_plan(
        61,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let supported_runtime = supported.build_resolve_runtime();
    let flat_runtime = flat.build_resolve_runtime();
    let supported_weight = supported_runtime
        .hierarchy_resolve_weight(200)
        .expect("supported hierarchy resolve weight");
    let flat_weight = flat_runtime
        .hierarchy_resolve_weight(200)
        .expect("flat hierarchy resolve weight");
    let supported_rt_weight = supported_runtime
        .hierarchy_rt_lighting(200)
        .expect("supported hierarchy rt lighting")[3];
    let flat_rt_weight = flat_runtime
        .hierarchy_rt_lighting(200)
        .expect("flat hierarchy rt lighting")[3];

    assert!(
        supported_weight > flat_weight + 0.04,
        "expected runtime-host resolve weighting to retain recent scene-driven lineage support for one more frame after the trace schedule clears, instead of collapsing immediately to the same flat hierarchy weight; flat_weight={flat_weight:.3}, supported_weight={supported_weight:.3}"
    );
    assert!(
        supported_rt_weight > flat_rt_weight + 0.02,
        "expected hierarchy RT-lighting continuation to preserve stronger scene-driven support after the trace schedule clears, instead of immediately matching the flat no-support runtime path; flat_rt_weight={flat_rt_weight:.3}, supported_rt_weight={supported_rt_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_pending_probe_hierarchy_rt_continuation_after_schedule_clears() {
    let supported_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe_at(100, true, 96, Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, false, 88, 100, Vec3::ZERO),
        ],
        trace_regions: vec![trace_region_at(40, Vec3::new(-0.8, 0.0, 0.0))],
    };
    let flat_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        trace_regions: vec![trace_region_at(40, Vec3::new(2.4, 0.0, 0.0))],
        ..supported_extract.clone()
    };

    let mut supported = HybridGiRuntimeState::default();
    supported.register_extract(Some(&supported_extract));
    supported.ingest_plan(
        62,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    supported.complete_gpu_updates(
        [],
        [40],
        &[(100, [144, 120, 96])],
        &[(100, [240, 96, 48])],
        &[],
    );
    supported.ingest_plan(
        63,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&flat_extract));
    flat.ingest_plan(
        62,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );
    flat.complete_gpu_updates(
        [],
        [40],
        &[(100, [144, 120, 96])],
        &[(100, [240, 96, 48])],
        &[],
    );
    flat.ingest_plan(
        63,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let supported_runtime = supported.build_resolve_runtime();
    let flat_runtime = flat.build_resolve_runtime();
    let supported_rt_weight = supported_runtime
        .hierarchy_rt_lighting(200)
        .expect("supported pending hierarchy rt lighting")[3];
    let flat_rt_weight = flat_runtime
        .hierarchy_rt_lighting(200)
        .expect("flat pending hierarchy rt lighting")[3];
    let supported_resolve_weight = supported_runtime
        .hierarchy_resolve_weight(200)
        .expect("supported pending hierarchy resolve weight");
    let flat_resolve_weight = flat_runtime
        .hierarchy_resolve_weight(200)
        .expect("flat pending hierarchy resolve weight");

    assert!(
        supported_rt_weight > flat_rt_weight + 0.002,
        "expected runtime-host resolve inputs to keep hierarchy RT-lighting continuation for pending probes after the trace schedule clears instead of dropping pending probes from the runtime source map; flat_rt_weight={flat_rt_weight:.3}, supported_rt_weight={supported_rt_weight:.3}"
    );
    assert!(
        supported_resolve_weight > flat_resolve_weight + 0.03,
        "expected runtime-host resolve inputs to keep hierarchy resolve weighting for pending probes after the trace schedule clears instead of emitting only resident entries; flat_resolve_weight={flat_resolve_weight:.3}, supported_resolve_weight={supported_resolve_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_uses_requested_lineage_support_in_runtime_resolve_without_trace_schedule(
) {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: vec![probe(100, true, 64), probe_with_parent(200, false, 96, 100)],
        trace_regions: Vec::new(),
    };

    let mut requested = HybridGiRuntimeState::default();
    requested.register_extract(Some(&extract));
    requested.complete_gpu_updates([], [], &[], &[(100, [220, 80, 32])], &[]);
    requested.ingest_plan(
        64,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&extract));
    flat.complete_gpu_updates([], [], &[], &[(100, [220, 80, 32])], &[]);
    flat.ingest_plan(
        64,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let requested_runtime = requested.build_resolve_runtime();
    let flat_runtime = flat.build_resolve_runtime();
    let requested_resolve_weight = requested_runtime
        .hierarchy_resolve_weight(200)
        .expect("requested hierarchy resolve weight");
    let flat_resolve_weight = flat_runtime
        .hierarchy_resolve_weight(200)
        .expect("flat hierarchy resolve weight");
    let requested_rt_weight = requested_runtime
        .hierarchy_rt_lighting(200)
        .expect("requested hierarchy rt lighting")[3];
    let flat_rt_weight = flat_runtime
        .hierarchy_rt_lighting(200)
        .expect("flat hierarchy rt lighting")[3];

    assert!(
        requested_resolve_weight > flat_resolve_weight + 0.08,
        "expected runtime-host resolve inputs to strengthen a still-requested screen-probe lineage even when no current trace schedule exists, instead of collapsing requested hierarchy continuation down to the same flat pending-probe weight; flat_resolve_weight={flat_resolve_weight:.3}, requested_resolve_weight={requested_resolve_weight:.3}"
    );
    assert!(
        requested_rt_weight > flat_rt_weight + 0.015,
        "expected requested screen-probe lineage support to keep RT-lighting continuation alive in runtime resolve inputs without a current trace schedule, instead of leaving the requested pending probe on the same zero-support path as an unrelated flat probe; flat_rt_weight={flat_rt_weight:.3}, requested_rt_weight={requested_rt_weight:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_scene_surface_cache_irradiance_continuation_without_trace_schedule(
) {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };

    let mut warm = HybridGiRuntimeState::default();
    warm.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/surface-cache-warm.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(300, 1.0)],
        &[],
        &[],
    );
    seed_runtime_probe_lineage_for_scene_truth(&mut warm);
    warm.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    ));

    let mut cool = HybridGiRuntimeState::default();
    cool.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/surface-cache-cool.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(300, 1.0)],
        &[],
        &[],
    );
    seed_runtime_probe_lineage_for_scene_truth(&mut cool);
    cool.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [64, 112, 224, 255])],
        vec![(0, [48, 96, 240, 255])],
    ));

    let warm_runtime = warm.build_resolve_runtime();
    let cool_runtime = cool.build_resolve_runtime();
    let warm_irradiance = warm_runtime
        .hierarchy_irradiance(200)
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let cool_irradiance = cool_runtime
        .hierarchy_irradiance(200)
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);

    assert!(
        warm_irradiance[3] > 0.1,
        "expected runtime-host resolve inputs to synthesize nonzero hierarchy irradiance from current surface-cache truth when probe-only continuation is absent and the trace schedule is empty; warm_irradiance={warm_irradiance:?}"
    );
    assert!(
        warm_irradiance[0] > cool_irradiance[0] + 0.2,
        "expected warm surface-cache truth to warm the runtime-host hierarchy irradiance continuation when no trace schedule exists, instead of collapsing warm/cool scene pages to the same empty runtime path; warm_irradiance={warm_irradiance:?}, cool_irradiance={cool_irradiance:?}"
    );
    assert!(
        cool_irradiance[2] > warm_irradiance[2] + 0.2,
        "expected cool surface-cache truth to cool the runtime-host hierarchy irradiance continuation when no trace schedule exists, instead of collapsing warm/cool scene pages to the same empty runtime path; warm_irradiance={warm_irradiance:?}, cool_irradiance={cool_irradiance:?}"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_scene_voxel_rt_lighting_continuation_without_trace_schedule() {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };

    let mut warm = HybridGiRuntimeState::default();
    warm.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/surface-cache-warm.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(300, 1.0)],
        &[],
        &[],
    );
    seed_runtime_probe_lineage_for_scene_truth(&mut warm);
    warm.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    ));

    let mut cool = HybridGiRuntimeState::default();
    cool.register_scene_extract(
        Some(&extract),
        &[mesh_at(
            11,
            "res://materials/surface-cache-cool.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(300, 1.0)],
        &[],
        &[],
    );
    seed_runtime_probe_lineage_for_scene_truth(&mut cool);
    cool.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [64, 112, 224, 255])],
        vec![(0, [48, 96, 240, 255])],
    ));

    let warm_runtime = warm.build_resolve_runtime();
    let cool_runtime = cool.build_resolve_runtime();
    let warm_rt_lighting = warm_runtime
        .hierarchy_rt_lighting(200)
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let cool_rt_lighting = cool_runtime
        .hierarchy_rt_lighting(200)
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);

    assert!(
        warm_runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected runtime-host exact RT-lighting continuation to record when it already includes scene-driven voxel or surface-cache truth so renderer-side resolve can avoid reblending the same current-frame scene signal"
    );
    assert!(
        cool_runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected runtime-host exact RT-lighting continuation to mark scene-driven truth for both warm and cool variants instead of leaving metadata empty on the same empty-trace path"
    );
    assert!(
        warm_rt_lighting[3] > 0.1,
        "expected runtime-host resolve inputs to synthesize nonzero hierarchy RT lighting from current scene-owned voxel or surface-cache truth when probe-only continuation is absent and the trace schedule is empty; warm_rt_lighting={warm_rt_lighting:?}"
    );
    assert!(
        warm_rt_lighting[0] > cool_rt_lighting[0] + 0.2,
        "expected warm scene-owned truth to warm the runtime-host hierarchy RT-lighting continuation when no trace schedule exists, instead of collapsing warm/cool scene inputs to the same empty runtime path; warm_rt_lighting={warm_rt_lighting:?}, cool_rt_lighting={cool_rt_lighting:?}"
    );
    assert!(
        cool_rt_lighting[2] > warm_rt_lighting[2] + 0.2,
        "expected cool scene-owned truth to cool the runtime-host hierarchy RT-lighting continuation when no trace schedule exists, instead of collapsing warm/cool scene inputs to the same empty runtime path; warm_rt_lighting={warm_rt_lighting:?}, cool_rt_lighting={cool_rt_lighting:?}"
    );
}

#[test]
fn hybrid_gi_runtime_state_reports_higher_scene_truth_quality_for_voxel_rt_than_surface_cache_only_rt(
) {
    let voxel_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };
    let surface_only_extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };

    let mut voxel_backed = HybridGiRuntimeState::default();
    voxel_backed.register_scene_extract(
        Some(&voxel_extract),
        &[mesh_at(
            11,
            "res://materials/runtime-scene-truth-quality-voxel.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(300, 1.0)],
        &[],
        &[],
    );
    seed_runtime_probe_lineage_for_scene_truth(&mut voxel_backed);
    let voxel_runtime = voxel_backed.build_resolve_runtime();
    let voxel_quality = voxel_runtime.hierarchy_rt_lighting_scene_truth_quality(200);

    let mut surface_only = HybridGiRuntimeState::default();
    surface_only.register_scene_extract(
        Some(&surface_only_extract),
        &[mesh_at(
            11,
            "res://materials/runtime-scene-truth-quality-surface-cache.mat",
            Vec3::ZERO,
            2.0,
        )],
        &[directional_light(300, 1.0)],
        &[],
        &[],
    );
    seed_runtime_probe_lineage_for_scene_truth(&mut surface_only);
    surface_only.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    ));
    let surface_only_runtime = surface_only.build_resolve_runtime();
    let surface_only_quality = surface_only_runtime.hierarchy_rt_lighting_scene_truth_quality(200);

    assert!(
        voxel_runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected voxel-backed exact RT-lighting continuation to mark scene-driven truth so runtime quality metadata can distinguish it from pure continuation"
    );
    assert!(
        surface_only_runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected surface-cache-only exact RT-lighting continuation to keep scene-driven truth metadata instead of collapsing the no-trace fallback into plain continuation"
    );
    assert!(
        voxel_quality > surface_only_quality + 0.1,
        "expected voxel-backed exact scene truth to carry higher quality than surface-cache-only fallback in runtime metadata; voxel_quality={voxel_quality:.3}, surface_only_quality={surface_only_quality:.3}"
    );
    assert!(
        surface_only_quality > 0.7,
        "expected surface-cache-only fallback quality to stay nonzero and reflect real capture/atlas authority instead of dropping to continuation-level zero; surface_only_quality={surface_only_quality:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_reports_clean_surface_cache_scene_truth_freshness_above_dirty_surface_cache_truth(
) {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };
    let scene_meshes = [mesh_at(
        11,
        "res://materials/runtime-scene-truth-freshness-surface-cache.mat",
        Vec3::ZERO,
        2.0,
    )];
    let scene_lights = [directional_light(300, 1.0)];
    let scene_prepare_resources = scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    );

    let mut dirty = HybridGiRuntimeState::default();
    dirty.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    dirty.apply_scene_prepare_resources_for_test(&scene_prepare_resources);
    seed_runtime_probe_lineage_for_scene_truth(&mut dirty);
    let dirty_runtime = dirty.build_resolve_runtime();
    let dirty_freshness = dirty_runtime.hierarchy_irradiance_scene_truth_freshness(200);

    let mut clean = HybridGiRuntimeState::default();
    clean.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    clean.apply_scene_prepare_resources_for_test(&scene_prepare_resources);
    clean.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    seed_runtime_probe_lineage_for_scene_truth(&mut clean);
    let clean_runtime = clean.build_resolve_runtime();
    let clean_freshness = clean_runtime.hierarchy_irradiance_scene_truth_freshness(200);

    assert_eq!(
        dirty.scene_dirty_page_ids(),
        vec![0],
        "expected the first surface-cache-backed scene registration to keep the authored page dirty until a stable follow-up registration clears it"
    );
    assert_eq!(
        clean.scene_dirty_page_ids(),
        Vec::<u32>::new(),
        "expected an unchanged follow-up scene registration to clear dirty surface-cache pages while preserving persisted page contents"
    );
    assert!(
        dirty_runtime.hierarchy_irradiance_includes_scene_truth(200),
        "expected dirty surface-cache-backed irradiance fallback to remain tagged as scene-driven truth so freshness metadata can down-weight reuse instead of collapsing to plain continuation"
    );
    assert!(
        clean_runtime.hierarchy_irradiance_includes_scene_truth(200),
        "expected clean surface-cache-backed irradiance fallback to keep scene-driven truth metadata instead of dropping freshness tracking on stable frames"
    );
    assert!(
        clean_freshness > dirty_freshness + 0.3,
        "expected stable surface-cache scene truth to report materially higher freshness than dirty page-backed truth in runtime metadata; clean_freshness={clean_freshness:.3}, dirty_freshness={dirty_freshness:.3}"
    );
    assert!(
        clean_freshness > 0.95,
        "expected clean surface-cache scene truth to approach full freshness once dirty pages clear; clean_freshness={clean_freshness:.3}"
    );
    assert!(
        dirty_freshness < 0.6,
        "expected dirty surface-cache pages to materially discount runtime freshness instead of reusing history as if the cached truth were stable; dirty_freshness={dirty_freshness:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_surface_cache_scene_truth_with_stale_scheduled_trace_region() {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };
    let scene_meshes = [mesh_at(
        11,
        "res://materials/runtime-scene-truth-stale-trace-region.mat",
        Vec3::ZERO,
        2.0,
    )];
    let scene_lights = [directional_light(300, 1.0)];
    let mut state = HybridGiRuntimeState::default();

    state.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    state.apply_scene_prepare_resources_for_test(&scene_prepare_resources_snapshot(
        vec![(0, [224, 112, 64, 255])],
        vec![(0, [240, 96, 48, 255])],
    ));
    seed_runtime_probe_lineage_for_scene_truth(&mut state);
    state.ingest_plan(
        1,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let runtime = state.build_resolve_runtime();

    assert_eq!(state.scheduled_trace_regions(), Vec::<u32>::new());
    assert!(
        runtime.hierarchy_irradiance_includes_scene_truth(200),
        "expected scheduled trace-region ids without current region scene data to be filtered before runtime surface-cache scene truth is resolved"
    );
}

#[test]
fn hybrid_gi_runtime_state_reports_clean_voxel_scene_truth_freshness_above_dirty_voxel_truth() {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };
    let scene_meshes = [mesh_at(
        11,
        "res://materials/runtime-scene-truth-freshness-voxel.mat",
        Vec3::ZERO,
        2.0,
    )];
    let scene_lights = [directional_light(300, 1.0)];

    let mut dirty = HybridGiRuntimeState::default();
    dirty.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    seed_runtime_probe_lineage_for_scene_truth(&mut dirty);
    let dirty_runtime = dirty.build_resolve_runtime();
    let dirty_freshness = dirty_runtime.hierarchy_rt_lighting_scene_truth_freshness(200);

    let mut clean = HybridGiRuntimeState::default();
    clean.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    clean.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    seed_runtime_probe_lineage_for_scene_truth(&mut clean);
    let clean_runtime = clean.build_resolve_runtime();
    let clean_freshness = clean_runtime.hierarchy_rt_lighting_scene_truth_freshness(200);

    assert_eq!(
        dirty.scene_dirty_clipmap_ids(),
        vec![0],
        "expected the first voxel clipmap build to stay dirty until an unchanged follow-up registration confirms stable scene residency"
    );
    assert_eq!(
        clean.scene_dirty_clipmap_ids(),
        Vec::<u32>::new(),
        "expected stable voxel clipmaps to clear the dirty set on an unchanged follow-up scene registration"
    );
    assert!(
        dirty_runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected dirty voxel-backed exact RT-lighting continuation to remain tagged as scene-driven truth so runtime freshness can down-weight temporal reuse"
    );
    assert!(
        clean_runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected clean voxel-backed exact RT-lighting continuation to keep scene-driven metadata instead of bypassing freshness tracking on stable frames"
    );
    assert!(
        clean_freshness > dirty_freshness + 0.2,
        "expected stable voxel scene truth to report materially higher freshness than dirty clipmap-backed truth in runtime metadata; clean_freshness={clean_freshness:.3}, dirty_freshness={dirty_freshness:.3}"
    );
    assert!(
        clean_freshness > 0.95,
        "expected clean voxel scene truth to approach full freshness on stable frames; clean_freshness={clean_freshness:.3}"
    );
    assert!(
        dirty_freshness < 0.8,
        "expected dirty voxel clipmaps to reduce runtime freshness instead of looking fully stable to temporal reuse; dirty_freshness={dirty_freshness:.3}"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_voxel_scene_truth_with_stale_scheduled_trace_region() {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };
    let scene_meshes = [mesh_at(
        11,
        "res://materials/runtime-voxel-scene-truth-stale-trace-region.mat",
        Vec3::ZERO,
        2.0,
    )];
    let scene_lights = [directional_light(300, 1.0)];
    let mut state = HybridGiRuntimeState::default();

    state.register_scene_extract(Some(&extract), &scene_meshes, &scene_lights, &[], &[]);
    seed_runtime_probe_lineage_for_scene_truth(&mut state);
    state.ingest_plan(
        1,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let runtime = state.build_resolve_runtime();

    assert_eq!(state.scheduled_trace_regions(), Vec::<u32>::new());
    assert!(
        runtime.hierarchy_rt_lighting_includes_scene_truth(200),
        "expected scheduled trace-region ids without current region scene data to be filtered before runtime voxel scene truth is resolved"
    );
}

#[test]
fn hybrid_gi_runtime_state_keeps_recent_requested_lineage_support_after_request_clears() {
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 2,
        tracing_budget: 0,
        probes: vec![probe(100, true, 64), probe_with_parent(200, false, 96, 100)],
        trace_regions: Vec::new(),
    };

    let mut supported = HybridGiRuntimeState::default();
    supported.register_extract(Some(&extract));
    supported.complete_gpu_updates([], [], &[], &[(100, [220, 80, 32])], &[]);
    supported.ingest_plan(
        65,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    supported.ingest_plan(
        66,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&extract));
    flat.complete_gpu_updates([], [], &[], &[(100, [220, 80, 32])], &[]);
    flat.ingest_plan(
        65,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );
    flat.ingest_plan(
        66,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: Vec::new(),
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        },
    );

    let supported_runtime = supported.build_resolve_runtime();
    let flat_runtime = flat.build_resolve_runtime();
    let supported_resolve_weight = supported_runtime
        .hierarchy_resolve_weight(200)
        .expect("supported hierarchy resolve weight");
    let flat_resolve_weight = flat_runtime
        .hierarchy_resolve_weight(200)
        .expect("flat hierarchy resolve weight");
    let supported_rt_weight = supported_runtime
        .hierarchy_rt_lighting(200)
        .expect("supported hierarchy rt lighting")[3];
    let flat_rt_weight = flat_runtime
        .hierarchy_rt_lighting(200)
        .expect("flat hierarchy rt lighting")[3];

    assert!(
        supported_resolve_weight > flat_resolve_weight + 0.04,
        "expected runtime-host resolve inputs to preserve one more frame of requested-lineage hierarchy support after the current request clears, instead of immediately collapsing pending probe weighting to the same flat no-request path; flat_resolve_weight={flat_resolve_weight:.3}, supported_resolve_weight={supported_resolve_weight:.3}"
    );
    assert!(
        supported_rt_weight > flat_rt_weight + 0.015,
        "expected requested-lineage history to keep hierarchy RT-lighting continuation alive for one more frame after the request clears, instead of immediately erasing the runtime/GPU source distinction; flat_rt_weight={flat_rt_weight:.3}, supported_rt_weight={supported_rt_weight:.3}"
    );
}

fn probe(probe_id: u32, resident: bool, ray_budget: u32) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position: Vec3::ZERO,
        radius: 0.5,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn probe_at(probe_id: u32, resident: bool, ray_budget: u32, position: Vec3) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        position,
        ..probe(probe_id, resident, ray_budget)
    }
}

fn probe_with_parent(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    parent_probe_id: u32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        parent_probe_id: Some(parent_probe_id),
        ..probe(probe_id, resident, ray_budget)
    }
}

fn probe_with_parent_at(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    parent_probe_id: u32,
    position: Vec3,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        position,
        ..probe_with_parent(probe_id, resident, ray_budget, parent_probe_id)
    }
}

fn seed_runtime_probe_lineage_for_scene_truth(state: &mut HybridGiRuntimeState) {
    state.seed_runtime_probe_scene_data_for_test([
        (100, Vec3::ZERO, 0.5, None, 96),
        (200, Vec3::ZERO, 1.8, Some(100), 88),
    ]);
}

fn trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity: 1,
        region_id,
        bounds_center: Vec3::ZERO,
        bounds_radius: 0.5,
        screen_coverage: 1.0,
        rt_lighting_rgb: [0, 0, 0],
    }
}

fn trace_region_at(region_id: u32, bounds_center: Vec3) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        bounds_center,
        ..trace_region(region_id)
    }
}

fn hybrid_gi_settings(card_budget: u32, voxel_budget: u32) -> RenderHybridGiExtract {
    RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget,
        voxel_budget,
        debug_view: Default::default(),
        probe_budget: 0,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    }
}

fn mesh(entity: u64, material: &str) -> RenderMeshSnapshot {
    mesh_at(entity, material, Vec3::ZERO, 1.0)
}

fn mesh_at(
    entity: u64,
    material: &str,
    translation: Vec3,
    uniform_scale: f32,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: entity,
        transform: Transform::from_translation(translation).with_scale(Vec3::splat(uniform_scale)),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/card.obj",
        )),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(material)),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }
}

fn directional_light(node_id: u64, intensity: f32) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id,
        direction: Vec3::new(-0.4, -1.0, -0.2),
        color: Vec3::new(1.0, 0.95, 0.9),
        intensity,
    }
}

fn pending_update_records(state: &HybridGiRuntimeState) -> Vec<(u32, u32, u64)> {
    state
        .pending_updates()
        .iter()
        .map(|update| (update.probe_id(), update.ray_budget(), update.generation()))
        .collect()
}

fn scene_prepare_resources_snapshot(
    atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
) -> HybridGiRuntimeScenePrepareResources {
    HybridGiRuntimeScenePrepareResources::new(atlas_slot_rgba_samples, capture_slot_rgba_samples)
}
