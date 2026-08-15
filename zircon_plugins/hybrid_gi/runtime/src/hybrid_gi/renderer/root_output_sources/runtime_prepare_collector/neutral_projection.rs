use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::{
    RenderHybridGiPreparedFrame, RenderHybridGiPreparedRadianceCacheUpdate, RenderMeshBounds,
};
use zircon_runtime::core::math::Vec3;

use crate::hybrid_gi::types::{
    hybrid_gi_voxel_clipmap_aabb_cell_ranges, hybrid_gi_voxel_clipmap_cell_bit_index,
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
    HybridGiPrepareRadianceCacheConsume, HybridGiPrepareRadianceCacheUpdate,
    HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareUpdateRequest, HybridGiPrepareVoxelCell,
    HybridGiPrepareVoxelClipmap, HybridGiResolveProbeSceneData, HybridGiResolveRuntime,
    HybridGiResolveTraceRegionSceneData, HybridGiScenePrepareFrame,
};

fn radiance_cache_updates_from_neutral(
    updates: &[RenderHybridGiPreparedRadianceCacheUpdate],
) -> Vec<HybridGiPrepareRadianceCacheUpdate> {
    updates
        .iter()
        .map(|update| HybridGiPrepareRadianceCacheUpdate {
            slot: update.slot,
            generation: update.generation,
            radiance_rgb: update.radiance_rgb,
            confidence_q8: update.confidence_q8,
            reuse_committed_radiance: update.reuse_committed_radiance,
        })
        .collect()
}

pub(super) fn radiance_cache_updates_for_instance(
    frame: &RenderHybridGiPreparedFrame,
    uses_bootstrap_snapshot: bool,
) -> Vec<HybridGiPrepareRadianceCacheUpdate> {
    radiance_cache_updates_from_neutral(if uses_bootstrap_snapshot {
        &frame.radiance_cache_bootstrap_updates
    } else {
        &frame.radiance_cache_updates
    })
}

pub(super) fn radiance_cache_consumes_from_neutral(
    frame: &RenderHybridGiPreparedFrame,
) -> Vec<HybridGiPrepareRadianceCacheConsume> {
    frame
        .radiance_cache_consumes
        .iter()
        .map(|consume| HybridGiPrepareRadianceCacheConsume {
            probe_id: consume.probe_id,
            generation: consume.generation,
            slots: consume.slots,
            weights_q16: consume.weights_q16,
        })
        .collect()
}

pub(super) fn prepare_frame_from_neutral(
    frame: &RenderHybridGiPreparedFrame,
) -> HybridGiPrepareFrame {
    HybridGiPrepareFrame {
        resident_probes: frame
            .resident_probes
            .iter()
            .map(|probe| HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: probe.slot,
                stable_instance_key: probe.stable_instance_key,
                source_mask: probe.source_mask,
                dynamic_weight_q8: probe.dynamic_weight_q8,
                ray_budget: probe.ray_budget,
                irradiance_rgb: probe.irradiance_rgb,
            })
            .collect(),
        pending_updates: frame
            .pending_updates
            .iter()
            .map(|update| HybridGiPrepareUpdateRequest {
                probe_id: update.probe_id,
                ray_budget: update.ray_budget,
                generation: update.generation,
            })
            .collect(),
        scheduled_trace_region_ids: frame.scheduled_trace_region_ids.clone(),
        evictable_probe_ids: frame.evictable_probe_ids.clone(),
    }
}

pub(super) fn scene_prepare_from_neutral(
    frame: &RenderHybridGiPreparedFrame,
    scene_mesh_world_bounds: &[(u64, RenderMeshBounds)],
) -> Option<HybridGiScenePrepareFrame> {
    let scene = frame.scene_prepare.as_ref()?;
    let world_bounds_by_instance_key = scene_mesh_world_bounds
        .iter()
        .copied()
        .collect::<BTreeMap<_, _>>();
    let world_bounds_by_card_id = scene
        .card_owners
        .iter()
        .filter_map(|owner| {
            world_bounds_by_instance_key
                .get(&owner.stable_instance_key)
                .copied()
                .map(|bounds| (owner.card_id, bounds))
        })
        .collect::<BTreeMap<_, _>>();
    let voxel_clipmaps = scene
        .voxel_clipmaps
        .iter()
        .map(|clipmap| HybridGiPrepareVoxelClipmap {
            clipmap_id: clipmap.clipmap_id,
            center: Vec3::from_array(clipmap.center),
            half_extent: clipmap.half_extent,
        })
        .collect::<Vec<_>>();
    let voxel_cells = voxel_cells_from_prepared_bounds(
        &voxel_clipmaps,
        &world_bounds_by_card_id,
        &scene.voxel_cells,
    );
    Some(HybridGiScenePrepareFrame {
        card_capture_requests: scene
            .card_capture_requests
            .iter()
            .filter_map(|request| {
                let bounds = world_bounds_by_card_id.get(&request.card_id)?;
                Some(HybridGiPrepareCardCaptureRequest {
                    card_id: request.card_id,
                    page_id: request.page_id,
                    atlas_slot_id: request.atlas_slot_id,
                    capture_slot_id: request.capture_slot_id,
                    bounds_center: Vec3::from_array(bounds.center),
                    bounds_radius: bounds.radius,
                })
            })
            .collect(),
        surface_cache_page_contents: scene
            .surface_cache_page_contents
            .iter()
            .filter_map(|page| {
                let bounds = world_bounds_by_card_id.get(&page.owner_card_id)?;
                Some(HybridGiPrepareSurfaceCachePageContent {
                    page_id: page.page_id,
                    owner_card_id: page.owner_card_id,
                    atlas_slot_id: page.atlas_slot_id,
                    capture_slot_id: page.capture_slot_id,
                    bounds_center: Vec3::from_array(bounds.center),
                    bounds_radius: bounds.radius,
                    atlas_sample_rgba: page.atlas_sample_rgba,
                    capture_sample_rgba: page.capture_sample_rgba,
                })
            })
            .collect(),
        voxel_clipmaps,
        voxel_cells,
        card_owner_stable_instance_keys: scene
            .card_owners
            .iter()
            .map(|owner| (owner.card_id, owner.stable_instance_key))
            .collect(),
        ..HybridGiScenePrepareFrame::default()
    })
}

fn voxel_cells_from_prepared_bounds(
    clipmaps: &[HybridGiPrepareVoxelClipmap],
    world_bounds_by_card_id: &BTreeMap<u32, RenderMeshBounds>,
    previous_cells: &[zircon_runtime::core::framework::render::RenderHybridGiPreparedVoxelCell],
) -> Vec<HybridGiPrepareVoxelCell> {
    // Geometry occupancy is rebuilt from prepared bounds; radiance can only survive an exact
    // clipmap-cell match and must never reintroduce transform-derived occupancy.
    let previous_radiance_by_cell = previous_cells
        .iter()
        .map(|cell| {
            (
                (cell.clipmap_id, cell.cell_index),
                (cell.radiance_present, cell.radiance_rgb),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut occupancy_by_cell = BTreeMap::<(u32, u32), (u32, u32)>::new();

    for (card_id, bounds) in world_bounds_by_card_id {
        for clipmap in clipmaps {
            let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
                hybrid_gi_voxel_clipmap_aabb_cell_ranges(
                    clipmap,
                    Vec3::from_array(bounds.min),
                    Vec3::from_array(bounds.max),
                )
            else {
                continue;
            };
            for z in z_start..=z_end {
                for y in y_start..=y_end {
                    for x in x_start..=x_end {
                        let cell_index = hybrid_gi_voxel_clipmap_cell_bit_index(x, y, z) as u32;
                        let entry = occupancy_by_cell
                            .entry((clipmap.clipmap_id, cell_index))
                            .or_insert((0, *card_id));
                        entry.0 = entry.0.saturating_add(1);
                        entry.1 = entry.1.min(*card_id);
                    }
                }
            }
        }
    }

    occupancy_by_cell
        .into_iter()
        .map(
            |((clipmap_id, cell_index), (occupancy_count, dominant_card_id))| {
                let (radiance_present, radiance_rgb) = previous_radiance_by_cell
                    .get(&(clipmap_id, cell_index))
                    .copied()
                    .unwrap_or((false, [0; 3]));
                HybridGiPrepareVoxelCell {
                    clipmap_id,
                    cell_index,
                    occupancy_count,
                    dominant_card_id,
                    radiance_present,
                    radiance_rgb,
                }
            },
        )
        .collect()
}

pub(super) fn resolve_runtime_from_neutral(
    frame: &RenderHybridGiPreparedFrame,
) -> HybridGiResolveRuntime {
    let probe_scene_data = frame
        .probe_scene_data
        .iter()
        .map(|probe| {
            (
                probe.probe_id,
                HybridGiResolveProbeSceneData::new(
                    probe.position_x_q,
                    probe.position_y_q,
                    probe.position_z_q,
                    probe.radius_q,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let trace_region_scene_data = frame
        .trace_region_scene_data
        .iter()
        .map(|region| {
            (
                region.region_id,
                HybridGiResolveTraceRegionSceneData::new(
                    region.center_x_q,
                    region.center_y_q,
                    region.center_z_q,
                    region.radius_q,
                    region.coverage_q,
                    region.rt_lighting_rgb,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let probe_rt_lighting_rgb = frame
        .probe_rt_lighting_rgb
        .iter()
        .map(|probe| (probe.probe_id, probe.rt_lighting_rgb))
        .collect::<BTreeMap<_, _>>();

    HybridGiResolveRuntime::new(
        probe_scene_data,
        trace_region_scene_data,
        BTreeMap::new(),
        probe_rt_lighting_rgb,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
}

#[cfg(test)]
mod tests {
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
        let radiance_cache_updates =
            radiance_cache_updates_from_neutral(&frame.radiance_cache_updates);
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
}
