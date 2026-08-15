use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::core::framework::render::{
    RenderHybridGiGlobalSdfStats, RenderHybridGiPreparedCardCaptureRequest,
    RenderHybridGiPreparedCardOwner, RenderHybridGiPreparedFrame, RenderHybridGiPreparedProbe,
    RenderHybridGiPreparedProbeRtLighting, RenderHybridGiPreparedProbeSceneData,
    RenderHybridGiPreparedRadianceCacheConsume, RenderHybridGiPreparedRadianceCacheUpdate,
    RenderHybridGiPreparedSceneFrame, RenderHybridGiPreparedSurfaceCachePageContent,
    RenderHybridGiPreparedTraceRegionSceneData, RenderHybridGiPreparedUpdateRequest,
    RenderHybridGiPreparedVoxelCell, RenderHybridGiPreparedVoxelClipmap,
    RenderHybridGiResolvedSettings, RenderHybridGiScenePrepareReadbackOutputs,
    RenderHybridGiVoxelCellDominantNodeRecord, RenderHybridGiVoxelCellRecord,
    RenderHybridGiVoxelCellSampleRecord, RenderPluginRendererOutputs,
    RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT,
};
use zircon_runtime::graphics::{
    HybridGiGpuCompletion as RuntimeHybridGiGpuCompletion, HybridGiRuntimeFeedback,
    HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput, HybridGiRuntimeProvider,
    HybridGiRuntimeState as RuntimeStateContract, HybridGiRuntimeStats, HybridGiRuntimeUpdate,
};

use crate::hybrid_gi::{
    HybridGiPrepareFrame, HybridGiPrepareVoxelCell, HybridGiResolveRuntime,
    HybridGiRuntimeScenePrepareResources, HybridGiRuntimeState, HybridGiScenePrepareFrame,
};

const LOW_DETAIL_VOXEL_FALLBACK_CELL_INDEX: u32 = 0;
const FIRST_RADIANCE_CACHE_INSTANCE_ID: u64 = 1;
static NEXT_RADIANCE_CACHE_INSTANCE_ID: AtomicU64 =
    AtomicU64::new(FIRST_RADIANCE_CACHE_INSTANCE_ID);

#[derive(Clone, Debug, Default)]
pub struct PluginHybridGiRuntimeProvider;

impl HybridGiRuntimeProvider for PluginHybridGiRuntimeProvider {
    fn create_state(&self) -> Box<dyn RuntimeStateContract> {
        Box::<PluginHybridGiRuntimeState>::default()
    }
}

#[derive(Debug)]
struct PluginHybridGiRuntimeState {
    state: HybridGiRuntimeState,
    radiance_cache_instance_id: u64,
    last_surface_cache_depth_sample_count: usize,
    last_probe_trace_tile_count: usize,
    last_probe_trace_dispatch_group_count: [usize; 3],
    last_radiance_cache_gpu_stage_dispatch_counts:
        [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    last_global_sdf_stats: RenderHybridGiGlobalSdfStats,
    last_resolved_settings: Option<RenderHybridGiResolvedSettings>,
}

impl Default for PluginHybridGiRuntimeState {
    fn default() -> Self {
        Self {
            state: HybridGiRuntimeState::default(),
            radiance_cache_instance_id: NEXT_RADIANCE_CACHE_INSTANCE_ID
                .fetch_add(1, Ordering::Relaxed),
            last_surface_cache_depth_sample_count: 0,
            last_probe_trace_tile_count: 0,
            last_probe_trace_dispatch_group_count: [0; 3],
            last_radiance_cache_gpu_stage_dispatch_counts: Default::default(),
            last_global_sdf_stats: Default::default(),
            last_resolved_settings: None,
        }
    }
}

impl RuntimeStateContract for PluginHybridGiRuntimeState {
    fn prepare_frame(
        &mut self,
        input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput {
        self.last_global_sdf_stats = Default::default();
        self.state.register_scene_extract_with_view_state(
            input.extract(),
            input.meshes(),
            input.directional_lights(),
            input.point_lights(),
            input.spot_lights(),
            input.baked_lighting(),
            input.has_baked_probe_grid(),
            input.camera_position(),
            input.history_invalidated(),
        );
        if let Some(plan) = input.update_plan() {
            self.state.ingest_plan(input.generation(), plan);
        }
        let prepare = self.state.build_prepare_frame();
        let resolve_runtime = self.state.build_resolve_runtime();
        let resolved_settings = input
            .extract()
            .filter(|extract| extract.enabled)
            .map(|_| self.state.resolved_settings());
        self.last_resolved_settings = resolved_settings;
        let scene_prepare_frame = self.state.build_scene_prepare_frame();
        let prepared_frame = neutral_prepared_frame_from_prepare(
            &prepare,
            &resolve_runtime,
            &scene_prepare_frame,
            self.radiance_cache_instance_id,
            self.state.composite_policy(),
            resolved_settings,
        );
        self.last_surface_cache_depth_sample_count = 0;
        self.last_probe_trace_tile_count = 0;
        self.last_probe_trace_dispatch_group_count = [0; 3];
        HybridGiRuntimePrepareOutput::new(prepare.evictable_probe_ids.clone())
            .with_renderer_outputs(RenderPluginRendererOutputs::default())
            .with_prepared_frame((!prepared_frame.is_empty()).then_some(prepared_frame))
    }

    fn update_after_render(&mut self, feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate {
        if let Some(completion) = feedback.gpu_completion() {
            self.apply_gpu_completion(completion, feedback.evictable_probe_ids());
        } else if let Some(feedback) = feedback.visibility_feedback() {
            self.state.consume_feedback(feedback);
        }

        let snapshot = self.state.snapshot();
        HybridGiRuntimeUpdate::new(
            HybridGiRuntimeStats::new(
                snapshot.cache_entry_count(),
                snapshot.resident_probe_count(),
                snapshot.pending_update_count(),
                snapshot.scheduled_trace_region_count(),
                snapshot.scene_card_count(),
                snapshot.scene_screen_probe_count(),
                snapshot.scene_radiance_cache_entry_count(),
                snapshot.radiance_cache_resident_probe_count(),
                snapshot.radiance_cache_update_probe_count(),
                snapshot.radiance_cache_truncated_demand_count(),
                snapshot.radiance_cache_generation(),
                snapshot.radiance_cache_scroll_count(),
                snapshot.radiance_cache_history_clear_count(),
                snapshot.surface_cache_resident_page_count(),
                snapshot.surface_cache_dirty_page_count(),
                snapshot.surface_cache_feedback_card_count(),
                snapshot.surface_cache_capture_slot_count(),
                snapshot.surface_cache_invalidated_page_count(),
                self.last_surface_cache_depth_sample_count,
                self.last_probe_trace_tile_count,
                self.last_probe_trace_dispatch_group_count,
                snapshot.voxel_resident_clipmap_count(),
                snapshot.voxel_dirty_clipmap_count(),
                snapshot.voxel_invalidated_clipmap_count(),
            )
            .with_radiance_cache_gpu_stage_dispatch_counts(
                self.last_radiance_cache_gpu_stage_dispatch_counts,
            )
            .with_global_sdf_stats(self.last_global_sdf_stats)
            .with_resolved_settings(self.last_resolved_settings),
        )
    }
}

impl PluginHybridGiRuntimeState {
    fn apply_gpu_completion(
        &mut self,
        completion: &RuntimeHybridGiGpuCompletion,
        evictable_probe_ids: &[u32],
    ) {
        self.state
            .apply_gpu_cache_entries(completion.cache_entries());
        self.last_radiance_cache_gpu_stage_dispatch_counts =
            completion.radiance_cache_gpu_stage_dispatch_counts();
        if let Some(stats) = completion.global_sdf_stats() {
            self.last_global_sdf_stats = stats;
        }
        if let Some(resources) = scene_prepare_resources_from_readback(completion.scene_prepare()) {
            self.state.apply_scene_prepare_resources(&resources);
        }
        if let Some(scene_prepare) = completion.scene_prepare() {
            self.last_surface_cache_depth_sample_count =
                scene_prepare.surface_cache_depth_samples.len();
            self.last_probe_trace_tile_count = scene_prepare.probe_trace_tiles.len();
            self.last_probe_trace_dispatch_group_count =
                scene_prepare_dispatch_as_usize(scene_prepare.probe_trace_dispatch);
        }
        self.state.complete_gpu_updates(
            completion.completed_probe_ids().iter().copied(),
            completion.completed_trace_region_ids().iter().copied(),
            completion.probe_irradiance_rgb(),
            completion.probe_trace_lighting_rgb(),
            evictable_probe_ids,
        );
    }
}

fn neutral_prepared_frame_from_prepare(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: &HybridGiResolveRuntime,
    scene_prepare: &HybridGiScenePrepareFrame,
    radiance_cache_instance_id: u64,
    composite_policy: zircon_runtime::core::framework::render::RenderHybridGiCompositePolicy,
    resolved_settings: Option<
        zircon_runtime::core::framework::render::RenderHybridGiResolvedSettings,
    >,
) -> RenderHybridGiPreparedFrame {
    let probe_ids = prepare
        .resident_probes
        .iter()
        .map(|probe| probe.probe_id)
        .chain(prepare.pending_updates.iter().map(|update| update.probe_id))
        .collect::<BTreeSet<_>>();
    let probe_scene_data = probe_ids
        .iter()
        .copied()
        .filter_map(|probe_id| {
            let scene_data = resolve_runtime.probe_scene_data(probe_id)?;
            Some(RenderHybridGiPreparedProbeSceneData {
                probe_id,
                position_x_q: scene_data.position_x_q(),
                position_y_q: scene_data.position_y_q(),
                position_z_q: scene_data.position_z_q(),
                radius_q: scene_data.radius_q(),
            })
        })
        .collect();
    let probe_rt_lighting_rgb = probe_ids
        .iter()
        .copied()
        .filter_map(|probe_id| {
            Some(RenderHybridGiPreparedProbeRtLighting {
                probe_id,
                rt_lighting_rgb: resolve_runtime.probe_rt_lighting_rgb(probe_id)?,
            })
        })
        .collect();

    RenderHybridGiPreparedFrame {
        composite_policy,
        resolved_settings,
        radiance_cache_instance_id,
        scene_prepare: Some(RenderHybridGiPreparedSceneFrame {
            card_capture_requests: scene_prepare
                .card_capture_requests
                .iter()
                .map(|request| RenderHybridGiPreparedCardCaptureRequest {
                    card_id: request.card_id,
                    page_id: request.page_id,
                    atlas_slot_id: request.atlas_slot_id,
                    capture_slot_id: request.capture_slot_id,
                    bounds_center: request.bounds_center.to_array(),
                    bounds_radius: request.bounds_radius,
                })
                .collect(),
            surface_cache_page_contents: scene_prepare
                .surface_cache_page_contents
                .iter()
                .map(|page| RenderHybridGiPreparedSurfaceCachePageContent {
                    page_id: page.page_id,
                    owner_card_id: page.owner_card_id,
                    atlas_slot_id: page.atlas_slot_id,
                    capture_slot_id: page.capture_slot_id,
                    bounds_center: page.bounds_center.to_array(),
                    bounds_radius: page.bounds_radius,
                    atlas_sample_rgba: page.atlas_sample_rgba,
                    capture_sample_rgba: page.capture_sample_rgba,
                })
                .collect(),
            voxel_clipmaps: scene_prepare
                .voxel_clipmaps
                .iter()
                .map(|clipmap| RenderHybridGiPreparedVoxelClipmap {
                    clipmap_id: clipmap.clipmap_id,
                    center: clipmap.center.to_array(),
                    half_extent: clipmap.half_extent,
                })
                .collect(),
            voxel_cells: scene_prepare
                .voxel_cells
                .iter()
                .map(|cell| RenderHybridGiPreparedVoxelCell {
                    clipmap_id: cell.clipmap_id,
                    cell_index: cell.cell_index,
                    occupancy_count: cell.occupancy_count,
                    dominant_card_id: cell.dominant_card_id,
                    radiance_present: cell.radiance_present,
                    radiance_rgb: cell.radiance_rgb,
                })
                .collect(),
            card_owners: scene_prepare
                .card_owner_stable_instance_keys
                .iter()
                .map(
                    |(card_id, stable_instance_key)| RenderHybridGiPreparedCardOwner {
                        card_id: *card_id,
                        stable_instance_key: *stable_instance_key,
                    },
                )
                .collect(),
        }),
        radiance_cache_bootstrap_updates: scene_prepare
            .radiance_cache_bootstrap_updates
            .iter()
            .map(|update| RenderHybridGiPreparedRadianceCacheUpdate {
                slot: update.slot,
                generation: update.generation,
                radiance_rgb: update.radiance_rgb,
                confidence_q8: update.confidence_q8,
                reuse_committed_radiance: false,
            })
            .collect(),
        radiance_cache_updates: scene_prepare
            .radiance_cache_updates
            .iter()
            .map(|update| RenderHybridGiPreparedRadianceCacheUpdate {
                slot: update.slot,
                generation: update.generation,
                radiance_rgb: update.radiance_rgb,
                confidence_q8: update.confidence_q8,
                reuse_committed_radiance: update.reuse_committed_radiance,
            })
            .collect(),
        radiance_cache_consumes: scene_prepare
            .radiance_cache_consumes
            .iter()
            .map(|consume| RenderHybridGiPreparedRadianceCacheConsume {
                probe_id: consume.probe_id,
                generation: consume.generation,
                slots: consume.slots,
                weights_q16: consume.weights_q16,
            })
            .collect(),
        resident_probes: prepare
            .resident_probes
            .iter()
            .map(|probe| RenderHybridGiPreparedProbe {
                probe_id: probe.probe_id,
                slot: probe.slot,
                stable_instance_key: probe.stable_instance_key,
                source_mask: probe.source_mask,
                dynamic_weight_q8: probe.dynamic_weight_q8,
                ray_budget: probe.ray_budget,
                irradiance_rgb: probe.irradiance_rgb,
            })
            .collect(),
        pending_updates: prepare
            .pending_updates
            .iter()
            .map(|update| RenderHybridGiPreparedUpdateRequest {
                probe_id: update.probe_id,
                ray_budget: update.ray_budget,
                generation: update.generation,
            })
            .collect(),
        scheduled_trace_region_ids: prepare.scheduled_trace_region_ids.clone(),
        evictable_probe_ids: prepare.evictable_probe_ids.clone(),
        probe_scene_data,
        probe_rt_lighting_rgb,
        trace_region_scene_data: prepare
            .scheduled_trace_region_ids
            .iter()
            .filter_map(|region_id| {
                let scene_data = resolve_runtime.trace_region_scene_data(*region_id)?;
                Some(RenderHybridGiPreparedTraceRegionSceneData {
                    region_id: *region_id,
                    center_x_q: scene_data.center_x_q(),
                    center_y_q: scene_data.center_y_q(),
                    center_z_q: scene_data.center_z_q(),
                    radius_q: scene_data.radius_q(),
                    coverage_q: scene_data.coverage_q(),
                    rt_lighting_rgb: scene_data.rt_lighting_rgb(),
                })
            })
            .collect(),
    }
}

fn scene_prepare_dispatch_as_usize(dispatch: [u32; 3]) -> [usize; 3] {
    [
        dispatch[0] as usize,
        dispatch[1] as usize,
        dispatch[2] as usize,
    ]
}

fn scene_prepare_resources_from_readback(
    readback: Option<&RenderHybridGiScenePrepareReadbackOutputs>,
) -> Option<HybridGiRuntimeScenePrepareResources> {
    let readback = readback?;
    let atlas_samples = readback
        .atlas_samples
        .iter()
        .map(|sample| (sample.index, sample.rgba8))
        .collect::<Vec<_>>();
    let capture_samples = readback
        .capture_samples
        .iter()
        .map(|sample| (sample.index, sample.rgba8))
        .collect::<Vec<_>>();
    let voxel_cells = scene_prepare_voxel_cells_from_readback(readback);
    (!atlas_samples.is_empty() || !capture_samples.is_empty() || !voxel_cells.is_empty()).then(
        || {
            HybridGiRuntimeScenePrepareResources::new(atlas_samples, capture_samples)
                .with_voxel_cells(voxel_cells)
        },
    )
}

fn scene_prepare_voxel_cells_from_readback(
    readback: &RenderHybridGiScenePrepareReadbackOutputs,
) -> Vec<HybridGiPrepareVoxelCell> {
    let occupancy_by_cell = voxel_cell_occupancy_by_key(readback);
    let dominant_card_by_cell = readback
        .voxel_cell_dominant_nodes
        .iter()
        .map(|cell| {
            (
                (cell.clipmap_id, cell.cell_id),
                u32::try_from(cell.dominant_node_id).unwrap_or_default(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let cell_sample_by_cell = voxel_cell_samples_by_key(&readback.voxel_cell_samples);
    let dominant_sample_by_cell = voxel_cell_samples_by_key(&readback.voxel_cell_dominant_samples);
    let clipmap_sample_by_id = readback
        .voxel_samples
        .iter()
        .map(|sample| (sample.index, sample.rgba8))
        .collect::<BTreeMap<_, _>>();
    let mut keys = occupancy_by_cell.keys().copied().collect::<BTreeSet<_>>();
    keys.extend(dominant_card_by_cell.keys().copied());
    keys.extend(cell_sample_by_cell.keys().copied());
    keys.extend(dominant_sample_by_cell.keys().copied());

    keys.into_iter()
        .map(|(clipmap_id, cell_id)| {
            let key = (clipmap_id, cell_id);
            let dominant_card_id = dominant_card_by_cell.get(&key).copied().unwrap_or_default();
            let cell_sample_present = cell_sample_by_cell
                .get(&key)
                .copied()
                .map(rgba_sample_is_present)
                .unwrap_or(false);
            let inferred_occupancy = if dominant_card_id != 0 || cell_sample_present {
                1
            } else {
                0
            };
            let (radiance_present, radiance_rgb) = preferred_voxel_cell_radiance_sample(
                &dominant_sample_by_cell,
                &cell_sample_by_cell,
                &clipmap_sample_by_id,
                key,
            );

            HybridGiPrepareVoxelCell {
                clipmap_id,
                cell_index: cell_id,
                occupancy_count: occupancy_by_cell
                    .get(&key)
                    .copied()
                    .unwrap_or(inferred_occupancy),
                dominant_card_id,
                radiance_present,
                radiance_rgb,
            }
        })
        .collect()
}

fn voxel_cell_occupancy_by_key(
    readback: &RenderHybridGiScenePrepareReadbackOutputs,
) -> BTreeMap<(u32, u32), u32> {
    let mut occupancy_by_cell = readback
        .voxel_cells
        .iter()
        .map(|cell| ((cell.clipmap_id, cell.cell_id), cell.occupancy))
        .collect::<BTreeMap<_, _>>();
    let mut cell_level_clipmap_ids = occupancy_by_cell
        .keys()
        .map(|(clipmap_id, _)| *clipmap_id)
        .collect::<BTreeSet<_>>();

    for mask in &readback.voxel_occupancy_masks {
        let mut mask_has_occupied_cell = false;
        for cell_id in 0..u64::BITS {
            if mask.occupancy_mask & (1_u64 << cell_id) == 0 {
                continue;
            }
            mask_has_occupied_cell = true;
            occupancy_by_cell
                .entry((mask.clipmap_id, cell_id))
                .or_insert(1);
        }
        if mask_has_occupied_cell {
            cell_level_clipmap_ids.insert(mask.clipmap_id);
        }
    }

    for (clipmap_id, occupancy_count) in readback
        .voxel_clipmap_ids
        .iter()
        .copied()
        .zip(readback.voxel_occupancy.iter().copied())
    {
        if occupancy_count == 0 || cell_level_clipmap_ids.contains(&clipmap_id) {
            continue;
        }

        occupancy_by_cell
            .entry((clipmap_id, LOW_DETAIL_VOXEL_FALLBACK_CELL_INDEX))
            .or_insert(occupancy_count);
    }

    occupancy_by_cell
}

fn voxel_cell_samples_by_key(
    samples: &[RenderHybridGiVoxelCellSampleRecord],
) -> BTreeMap<(u32, u32), [u8; 4]> {
    samples
        .iter()
        .map(|sample| ((sample.clipmap_id, sample.cell_id), sample.rgba8))
        .collect()
}

fn preferred_voxel_cell_radiance_sample(
    dominant_sample_by_cell: &BTreeMap<(u32, u32), [u8; 4]>,
    cell_sample_by_cell: &BTreeMap<(u32, u32), [u8; 4]>,
    clipmap_sample_by_id: &BTreeMap<u32, [u8; 4]>,
    key: (u32, u32),
) -> (bool, [u8; 3]) {
    let Some(rgba) = dominant_sample_by_cell
        .get(&key)
        .copied()
        .filter(|rgba| rgba_sample_is_present(*rgba))
        .or_else(|| {
            cell_sample_by_cell
                .get(&key)
                .copied()
                .filter(|rgba| rgba_sample_is_present(*rgba))
        })
        .or_else(|| {
            clipmap_sample_by_id
                .get(&key.0)
                .copied()
                .filter(|rgba| rgba_sample_is_present(*rgba))
        })
    else {
        return (false, [0, 0, 0]);
    };

    (true, [rgba[0], rgba[1], rgba[2]])
}

fn rgba_sample_is_present(rgba: [u8; 4]) -> bool {
    rgba[3] > 0
}

#[cfg(test)]
mod tests;
