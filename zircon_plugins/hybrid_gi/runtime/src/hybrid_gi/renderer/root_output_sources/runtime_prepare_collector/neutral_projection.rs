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
    // 卡页仅使用本帧已投影的权威世界边界；缺少实例边界时过滤其捕获与页内容。
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
#[path = "tests/neutral_projection.rs"]
mod tests;
