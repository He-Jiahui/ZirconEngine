use zircon_runtime::core::framework::render::{
    RenderHybridGiPreparedCardCaptureRequest, RenderHybridGiPreparedCardOwner,
    RenderHybridGiPreparedProbe, RenderHybridGiPreparedProbeRtLighting,
    RenderHybridGiPreparedProbeSceneData, RenderHybridGiPreparedRadianceCacheConsume,
    RenderHybridGiPreparedSceneFrame, RenderHybridGiPreparedSurfaceCachePageContent,
    RenderHybridGiPreparedUpdateRequest, RenderHybridGiPreparedVoxelCell,
    RenderHybridGiPreparedVoxelClipmap, RenderMeshBounds,
};

use super::*;

#[test]
fn neutral_prepared_frame_projects_to_gpu_prepare_inputs() {
    let frame = RenderHybridGiPreparedFrame {
        composite_policy: Default::default(),
        resolved_settings: None,
        radiance_cache_instance_id: 101,
        scene_prepare: Some(RenderHybridGiPreparedSceneFrame {
            card_capture_requests: vec![RenderHybridGiPreparedCardCaptureRequest {
                card_id: 3,
                page_id: 5,
                atlas_slot_id: 7,
                capture_slot_id: 11,
                bounds_center: [1.0, 2.0, 3.0],
                bounds_radius: 4.0,
            }],
            surface_cache_page_contents: vec![RenderHybridGiPreparedSurfaceCachePageContent {
                page_id: 5,
                owner_card_id: 3,
                atlas_slot_id: 7,
                capture_slot_id: 11,
                bounds_center: [1.0, 2.0, 3.0],
                bounds_radius: 4.0,
                atlas_sample_rgba: [1, 2, 3, 255],
                capture_sample_rgba: [4, 5, 6, 255],
            }],
            voxel_clipmaps: vec![RenderHybridGiPreparedVoxelClipmap {
                clipmap_id: 13,
                center: [0.0, 1.0, 2.0],
                half_extent: 16.0,
            }],
            voxel_cells: vec![RenderHybridGiPreparedVoxelCell {
                clipmap_id: 13,
                cell_index: 9,
                occupancy_count: 2,
                dominant_card_id: 3,
                radiance_present: true,
                radiance_rgb: [32, 64, 96],
            }],
            card_owners: vec![RenderHybridGiPreparedCardOwner {
                card_id: 3,
                stable_instance_key: 77,
            }],
        }),
        radiance_cache_bootstrap_updates: vec![RenderHybridGiPreparedRadianceCacheUpdate {
            slot: 4,
            generation: 13,
            radiance_rgb: [12, 13, 14],
            confidence_q8: 224,
            reuse_committed_radiance: false,
        }],
        radiance_cache_updates: vec![RenderHybridGiPreparedRadianceCacheUpdate {
            slot: 2,
            generation: 13,
            radiance_rgb: [9, 10, 11],
            confidence_q8: 192,
            reuse_committed_radiance: false,
        }],
        radiance_cache_consumes: vec![RenderHybridGiPreparedRadianceCacheConsume {
            probe_id: 7,
            generation: 13,
            slots: [2; 8],
            weights_q16: [u16::MAX, 0, 0, 0, 0, 0, 0, 0],
        }],
        resident_probes: vec![RenderHybridGiPreparedProbe {
            probe_id: 7,
            slot: 2,
            stable_instance_key: 77,
            source_mask: zircon_runtime::core::framework::render::HYBRID_GI_SOURCE_FULL_DYNAMIC,
            dynamic_weight_q8: u8::MAX,
            ray_budget: 32,
            irradiance_rgb: [3, 4, 5],
        }],
        pending_updates: vec![RenderHybridGiPreparedUpdateRequest {
            probe_id: 9,
            ray_budget: 64,
            generation: 11,
        }],
        scheduled_trace_region_ids: vec![44],
        evictable_probe_ids: vec![6],
        probe_scene_data: vec![RenderHybridGiPreparedProbeSceneData {
            probe_id: 7,
            position_x_q: 2000,
            position_y_q: 2010,
            position_z_q: 2020,
            radius_q: 96,
        }],
        probe_rt_lighting_rgb: vec![RenderHybridGiPreparedProbeRtLighting {
            probe_id: 7,
            rt_lighting_rgb: [64, 32, 16],
        }],
        trace_region_scene_data: Vec::new(),
    };

    let prepare = prepare_frame_from_neutral(&frame);
    let runtime = resolve_runtime_from_neutral(&frame);
    let world_bounds = RenderMeshBounds::from_min_max([2.0, 3.0, 4.0], [4.0, 5.0, 6.0]);
    let scene_prepare = scene_prepare_from_neutral(&frame, &[(77, world_bounds)]).unwrap();
    let radiance_cache_updates = radiance_cache_updates_from_neutral(&frame.radiance_cache_updates);
    let radiance_cache_bootstrap_updates =
        radiance_cache_updates_from_neutral(&frame.radiance_cache_bootstrap_updates);
    let radiance_cache_consumes = radiance_cache_consumes_from_neutral(&frame);

    assert_eq!(prepare.resident_probes[0].probe_id, 7);
    assert_eq!(prepare.pending_updates[0].generation, 11);
    assert_eq!(prepare.scheduled_trace_region_ids, vec![44]);
    assert_eq!(prepare.evictable_probe_ids, vec![6]);
    assert_eq!(runtime.probe_scene_data(7).unwrap().position_x_q(), 2000);
    assert_eq!(runtime.probe_rt_lighting_rgb(7), Some([64, 32, 16]));
    assert_eq!(radiance_cache_updates[0].slot, 2);
    assert_eq!(radiance_cache_updates[0].generation, 13);
    assert_eq!(radiance_cache_updates[0].radiance_rgb, [9, 10, 11]);
    assert_eq!(radiance_cache_bootstrap_updates[0].slot, 4);
    assert_eq!(
        radiance_cache_bootstrap_updates[0].radiance_rgb,
        [12, 13, 14]
    );
    assert_eq!(radiance_cache_updates_for_instance(&frame, true)[0].slot, 4);
    assert_eq!(
        radiance_cache_updates_for_instance(&frame, false)[0].slot,
        2
    );
    assert_eq!(radiance_cache_consumes[0].probe_id, 7);
    assert_eq!(radiance_cache_consumes[0].slots, [2; 8]);
    assert_eq!(scene_prepare.card_capture_requests[0].card_id, 3);
    assert_eq!(
        scene_prepare.card_capture_requests[0].bounds_center,
        Vec3::new(3.0, 4.0, 5.0)
    );
    assert_eq!(
        scene_prepare.card_capture_requests[0].bounds_radius,
        world_bounds.radius
    );
    assert_eq!(
        scene_prepare.surface_cache_page_contents[0].capture_sample_rgba,
        [4, 5, 6, 255]
    );
    assert_eq!(
        scene_prepare.surface_cache_page_contents[0].bounds_center,
        Vec3::new(3.0, 4.0, 5.0)
    );
    assert_eq!(scene_prepare.voxel_clipmaps[0].half_extent, 16.0);
    assert_eq!(scene_prepare.voxel_cells.len(), 1);
    assert_eq!(
        scene_prepare.voxel_cells[0].cell_index,
        hybrid_gi_voxel_clipmap_cell_bit_index(2, 2, 2) as u32
    );
    assert_eq!(scene_prepare.voxel_cells[0].dominant_card_id, 3);
    assert_eq!(scene_prepare.voxel_cells[0].occupancy_count, 1);
    assert_eq!(scene_prepare.card_owner_stable_instance_keys, vec![(3, 77)]);

    let missing_geometry = scene_prepare_from_neutral(&frame, &[]).unwrap();
    assert!(missing_geometry.card_capture_requests.is_empty());
    assert!(missing_geometry.surface_cache_page_contents.is_empty());
    assert_eq!(missing_geometry.voxel_clipmaps.len(), 1);
    assert!(missing_geometry.voxel_cells.is_empty());
}
