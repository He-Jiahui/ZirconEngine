use crate::core::framework::render::{
    RenderHybridGiDebugView, RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiQuality,
    RenderHybridGiTraceRegion, RenderMeshSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::runtime::{HybridGiInputSet, HybridGiSceneRepresentation};

#[test]
fn hybrid_gi_input_contract_stays_complete_for_deferred_and_forward_plus() {
    let deferred = HybridGiInputSet::deferred();
    let forward_plus = HybridGiInputSet::forward_plus();

    assert!(deferred.is_complete());
    assert!(forward_plus.is_complete());
    assert_eq!(deferred.required_input_count(), 7);
    assert_eq!(forward_plus.required_input_count(), 7);
}

#[test]
fn hybrid_gi_scene_representation_separates_public_settings_from_internal_fixture_bridge() {
    let representation = HybridGiSceneRepresentation::from_extract(&RenderHybridGiExtract {
        enabled: true,
        quality: RenderHybridGiQuality::High,
        trace_budget: 24,
        card_budget: 48,
        voxel_budget: 12,
        debug_view: RenderHybridGiDebugView::SurfaceCache,
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![RenderHybridGiProbe {
            entity: 7,
            probe_id: 10,
            position: Vec3::new(1.0, 2.0, 3.0),
            radius: 1.5,
            parent_probe_id: None,
            resident: true,
            ray_budget: 96,
        }],
        trace_regions: vec![RenderHybridGiTraceRegion {
            entity: 8,
            region_id: 11,
            bounds_center: Vec3::new(2.0, 1.0, -1.0),
            bounds_radius: 2.5,
            screen_coverage: 0.75,
            rt_lighting_rgb: [32, 64, 96],
        }],
    });

    assert_eq!(representation.settings.trace_budget, 24);
    assert_eq!(representation.settings.card_budget, 48);
    assert_eq!(representation.settings.voxel_budget, 12);
    assert_eq!(representation.fixture_probe_count(), 1);
    assert_eq!(representation.fixture_trace_region_count(), 1);
    assert!(representation.inputs.is_complete());
    assert_eq!(representation.surface_cache.resident_page_count(), 0);
    assert_eq!(representation.voxel_scene.resident_clipmap_count(), 0);
}

#[test]
fn hybrid_gi_scene_representation_counts_duplicate_legacy_fixture_payloads_once() {
    let representation = HybridGiSceneRepresentation::from_extract(&RenderHybridGiExtract {
        enabled: true,
        quality: RenderHybridGiQuality::High,
        trace_budget: 24,
        card_budget: 48,
        voxel_budget: 12,
        debug_view: RenderHybridGiDebugView::SurfaceCache,
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            RenderHybridGiProbe {
                probe_id: 10,
                ..Default::default()
            },
            RenderHybridGiProbe {
                probe_id: 10,
                position: Vec3::new(9.0, 0.0, 0.0),
                ..Default::default()
            },
        ],
        trace_regions: vec![
            RenderHybridGiTraceRegion {
                region_id: 11,
                ..Default::default()
            },
            RenderHybridGiTraceRegion {
                region_id: 11,
                bounds_center: Vec3::new(9.0, 0.0, 0.0),
                ..Default::default()
            },
        ],
    });

    assert_eq!(
        representation.fixture_probe_count(),
        1,
        "expected legacy fixture probe stats to count unique ids after the old authored probe path is demoted"
    );
    assert_eq!(
        representation.fixture_trace_region_count(),
        1,
        "expected legacy fixture trace-region stats to count unique ids after the old authored trace path is demoted"
    );
}

#[test]
fn hybrid_gi_scene_representation_tracks_surface_cache_feedback_from_card_budget() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 3));

    representation.synchronize_cards([11, 22, 33]);

    assert_eq!(representation.card_ids(), vec![11, 22, 33]);
    assert_eq!(representation.surface_cache.resident_page_ids(), vec![0, 1]);
    assert_eq!(representation.surface_cache.dirty_page_ids(), vec![0, 1]);
    assert_eq!(representation.surface_cache.feedback_card_ids(), vec![33]);
    assert_eq!(
        representation.surface_cache.invalidated_page_ids(),
        Vec::<u32>::new()
    );
    assert_eq!(
        representation.voxel_scene.resident_clipmap_ids(),
        vec![0, 1, 2]
    );
    assert_eq!(
        representation.voxel_scene.dirty_clipmap_ids(),
        vec![0, 1, 2]
    );
}

#[test]
fn hybrid_gi_scene_representation_reuses_evicted_pages_after_card_invalidation() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_cards([11, 22, 33]);
    representation.synchronize_cards([22, 33]);

    assert_eq!(representation.card_ids(), vec![22, 33]);
    assert_eq!(representation.surface_cache.resident_page_ids(), vec![1, 0]);
    assert_eq!(representation.surface_cache.dirty_page_ids(), vec![0]);
    assert_eq!(
        representation.surface_cache.feedback_card_ids(),
        Vec::<u32>::new()
    );
    assert_eq!(representation.surface_cache.invalidated_page_ids(), vec![0]);
    assert_eq!(
        representation.voxel_scene.resident_clipmap_ids(),
        vec![0, 1]
    );
    assert_eq!(representation.voxel_scene.dirty_clipmap_ids(), vec![0, 1]);
}

#[test]
fn hybrid_gi_scene_representation_builds_surface_cache_page_table_and_capture_slots() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_cards([11, 22, 33]);

    assert_eq!(
        representation.surface_cache.page_table_entries(),
        vec![(0, 0), (1, 1)]
    );
    assert_eq!(
        representation.surface_cache.capture_slot_entries(),
        vec![(0, 0), (1, 1)]
    );
}

#[test]
fn hybrid_gi_scene_representation_builds_card_capture_requests_from_dirty_pages() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, Vec3::new(3.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(
        representation.card_capture_requests(),
        vec![
            (11, 0, 0, 0, [-1.0, 0.0, 0.0], 1.0),
            (22, 1, 1, 1, [3.0, 0.0, 0.0], 0.5),
        ]
    );
}

#[test]
fn hybrid_gi_scene_representation_allocates_page_ids_separately_from_owner_card_ids() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, Vec3::new(3.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(representation.surface_cache.resident_page_ids(), vec![0, 1]);
    assert_eq!(
        representation.surface_cache.page_table_entries(),
        vec![(0, 0), (1, 1)]
    );
    assert_eq!(
        representation.card_capture_requests(),
        vec![
            (11, 0, 0, 0, [-1.0, 0.0, 0.0], 1.0),
            (22, 1, 1, 1, [3.0, 0.0, 0.0], 0.5),
        ]
    );
}

#[test]
fn hybrid_gi_scene_representation_reuses_surface_cache_slots_after_invalidation() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_cards([11, 22]);
    representation.synchronize_cards([22, 33]);

    assert_eq!(
        representation.surface_cache.page_table_entries(),
        vec![(1, 1), (0, 0)]
    );
    assert_eq!(
        representation.surface_cache.capture_slot_entries(),
        vec![(0, 0)]
    );
}

#[test]
fn hybrid_gi_scene_representation_reuses_recycled_page_id_for_new_owner_after_invalidation() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, Vec3::new(3.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );
    representation.synchronize_scene(
        &[
            mesh_at(22, Vec3::new(3.0, 0.0, 0.0), 1.0),
            mesh_at(33, Vec3::new(6.0, 0.0, 0.0), 1.5),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(representation.surface_cache.resident_page_ids(), vec![1, 0]);
    assert_eq!(representation.surface_cache.invalidated_page_ids(), vec![0]);
    assert_eq!(
        representation.surface_cache.page_table_entries(),
        vec![(1, 1), (0, 0)]
    );
    assert_eq!(
        representation.card_capture_requests(),
        vec![(33, 0, 0, 0, [6.0, 0.0, 0.0], 0.75)]
    );
}

#[test]
fn hybrid_gi_scene_representation_keeps_only_changed_pages_in_card_capture_requests() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, Vec3::new(3.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );
    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(-1.0, 0.0, 0.0), 2.0),
            mesh_at(22, Vec3::new(4.0, 0.0, 0.0), 1.5),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(
        representation.card_capture_requests(),
        vec![(22, 1, 1, 1, [4.0, 0.0, 0.0], 0.75)]
    );
}

#[test]
fn hybrid_gi_scene_representation_preserves_capture_slot_for_resident_page_redirty() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(2, 2));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(0.0, 0.0, 0.0), 1.0),
            mesh_at(22, Vec3::new(2.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );
    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(0.0, 0.0, 0.0), 1.0),
            mesh_at(22, Vec3::new(2.0, 0.0, 0.0), 1.5),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(
        representation.surface_cache.page_table_entries(),
        vec![(0, 0), (1, 1)]
    );
    assert_eq!(
        representation.surface_cache.capture_slot_entries(),
        vec![(1, 1)]
    );
}

#[test]
fn hybrid_gi_scene_representation_builds_voxel_clipmap_descriptors_from_scene_bounds() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(4, 2));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(-4.0, 0.0, 0.0), 1.0),
            mesh_at(22, Vec3::new(4.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(
        representation.voxel_scene.clipmap_descriptors(),
        vec![(0, [0.0, 0.0, 0.0], 5.0), (1, [0.0, 0.0, 0.0], 10.0),]
    );
}

#[test]
fn hybrid_gi_scene_representation_updates_and_invalidates_voxel_clipmap_descriptors() {
    let mut representation = HybridGiSceneRepresentation::from_extract(&extract_with_budgets(4, 2));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(-4.0, 0.0, 0.0), 1.0),
            mesh_at(22, Vec3::new(4.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );
    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::new(7.0, 0.0, 0.0), 1.0),
            mesh_at(22, Vec3::new(9.0, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(
        representation.voxel_scene.clipmap_descriptors(),
        vec![(0, [8.0, 0.0, 0.0], 2.0), (1, [8.0, 0.0, 0.0], 4.0),]
    );
    assert_eq!(representation.voxel_scene.dirty_clipmap_ids(), vec![0, 1]);

    representation.synchronize_scene(&[], &[], &[], &[]);

    assert_eq!(
        representation.voxel_scene.clipmap_descriptors(),
        Vec::<(u32, [f32; 3], f32)>::new()
    );
    assert_eq!(
        representation.voxel_scene.invalidated_clipmap_ids(),
        vec![0, 1]
    );
}

fn extract_with_budgets(card_budget: u32, voxel_budget: u32) -> RenderHybridGiExtract {
    RenderHybridGiExtract {
        enabled: true,
        quality: RenderHybridGiQuality::High,
        trace_budget: 24,
        card_budget,
        voxel_budget,
        debug_view: RenderHybridGiDebugView::SurfaceCache,
        probe_budget: 0,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    }
}

fn mesh_at(node_id: u64, translation: Vec3, uniform_scale: f32) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        transform: Transform::from_translation(translation).with_scale(Vec3::splat(uniform_scale)),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
            "builtin://hybrid-gi/test-mesh/{node_id}/model"
        ))),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(&format!(
            "builtin://hybrid-gi/test-mesh/{node_id}/material"
        ))),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }
}
