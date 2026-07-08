use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::{
    RenderHybridGiExtract, RenderHybridGiPreparedFrame, RenderHybridGiPreparedProbe,
    RenderHybridGiPreparedProbeRtLighting, RenderHybridGiPreparedProbeSceneData,
    RenderHybridGiPreparedTraceRegionSceneData, RenderHybridGiPreparedUpdateRequest,
    RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
    RenderHybridGiScenePrepareSample, RenderHybridGiTraceTileRecord,
    RenderHybridGiVoxelCellDominantNodeRecord, RenderHybridGiVoxelCellRecord,
    RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelOccupancyMaskRecord,
    RenderPluginRendererOutputs,
};
use zircon_runtime::core::math::Vec3;
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

#[derive(Clone, Debug, Default)]
pub struct PluginHybridGiRuntimeProvider;

impl HybridGiRuntimeProvider for PluginHybridGiRuntimeProvider {
    fn create_state(&self) -> Box<dyn RuntimeStateContract> {
        Box::<PluginHybridGiRuntimeState>::default()
    }
}

#[derive(Debug, Default)]
struct PluginHybridGiRuntimeState {
    state: HybridGiRuntimeState,
    last_surface_cache_depth_sample_count: usize,
    last_probe_trace_tile_count: usize,
    last_probe_trace_dispatch_group_count: [usize; 3],
}

impl RuntimeStateContract for PluginHybridGiRuntimeState {
    fn prepare_frame(
        &mut self,
        input: HybridGiRuntimePrepareInput<'_>,
    ) -> HybridGiRuntimePrepareOutput {
        self.state.register_scene_extract(
            input.extract(),
            input.meshes(),
            input.directional_lights(),
            input.point_lights(),
            input.spot_lights(),
        );
        if let Some(plan) = input.update_plan() {
            self.state.ingest_plan(input.generation(), plan);
        }
        let prepare = self.state.build_prepare_frame();
        let resolve_runtime = self.state.build_resolve_runtime();
        let prepared_frame = neutral_prepared_frame_from_prepare(&prepare, &resolve_runtime);
        let scene_prepare_frame = self.state.build_scene_prepare_frame();
        let renderer_outputs =
            renderer_outputs_from_scene_prepare_frame(&scene_prepare_frame, input.extract());
        let scene_prepare_outputs = &renderer_outputs.hybrid_gi.scene_prepare;
        self.last_surface_cache_depth_sample_count =
            scene_prepare_outputs.surface_cache_depth_samples.len();
        self.last_probe_trace_tile_count = scene_prepare_outputs.probe_trace_tiles.len();
        self.last_probe_trace_dispatch_group_count =
            scene_prepare_dispatch_as_usize(scene_prepare_outputs.probe_trace_dispatch);
        HybridGiRuntimePrepareOutput::new(prepare.evictable_probe_ids.clone())
            .with_renderer_outputs(renderer_outputs)
            .with_prepared_frame((!prepared_frame.is_empty()).then_some(prepared_frame))
    }

    fn update_after_render(&mut self, feedback: HybridGiRuntimeFeedback) -> HybridGiRuntimeUpdate {
        if let Some(completion) = feedback.gpu_completion() {
            self.apply_gpu_completion(completion, feedback.evictable_probe_ids());
        } else if let Some(feedback) = feedback.visibility_feedback() {
            self.state.consume_feedback(feedback);
        }

        let snapshot = self.state.snapshot();
        HybridGiRuntimeUpdate::new(HybridGiRuntimeStats::new(
            snapshot.cache_entry_count(),
            snapshot.resident_probe_count(),
            snapshot.pending_update_count(),
            snapshot.scheduled_trace_region_count(),
            snapshot.scene_card_count(),
            snapshot.scene_screen_probe_count(),
            snapshot.scene_radiance_cache_entry_count(),
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
        ))
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
        resident_probes: prepare
            .resident_probes
            .iter()
            .map(|probe| RenderHybridGiPreparedProbe {
                probe_id: probe.probe_id,
                slot: probe.slot,
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

fn renderer_outputs_from_scene_prepare_frame(
    frame: &HybridGiScenePrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> RenderPluginRendererOutputs {
    RenderPluginRendererOutputs {
        hybrid_gi: RenderHybridGiReadbackOutputs {
            scene_prepare: scene_prepare_readback_outputs_from_frame(frame, extract),
            ..RenderHybridGiReadbackOutputs::default()
        },
        ..RenderPluginRendererOutputs::default()
    }
}

fn scene_prepare_readback_outputs_from_frame(
    frame: &HybridGiScenePrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> RenderHybridGiScenePrepareReadbackOutputs {
    let voxel_clipmap_ids = scene_prepare_feedback_clipmap_ids(frame);
    let probe_trace_tiles = probe_trace_tiles_from_frame(frame, extract);
    let probe_trace_dispatch = probe_trace_dispatch_from_tile_count(probe_trace_tiles.len());
    RenderHybridGiScenePrepareReadbackOutputs {
        occupied_atlas_slots: occupied_atlas_slots(frame),
        occupied_capture_slots: occupied_capture_slots(frame),
        atlas_samples: atlas_samples_from_frame(frame),
        capture_samples: capture_samples_from_frame(frame),
        surface_cache_depth_samples: surface_cache_depth_samples_from_frame(frame),
        voxel_occupancy: voxel_clipmap_ids
            .iter()
            .map(|clipmap_id| voxel_occupancy_for_clipmap(frame, *clipmap_id))
            .collect(),
        voxel_samples: voxel_samples_from_frame(frame, &voxel_clipmap_ids),
        voxel_occupancy_masks: voxel_occupancy_masks_from_frame(frame, &voxel_clipmap_ids),
        voxel_cells: voxel_cells_from_frame(frame),
        voxel_cell_samples: voxel_cell_samples_from_frame(frame),
        voxel_cell_dominant_nodes: voxel_cell_dominant_nodes_from_frame(frame),
        voxel_cell_dominant_samples: voxel_cell_dominant_samples_from_frame(frame),
        probe_trace_tiles,
        probe_trace_dispatch,
        voxel_clipmap_ids,
        ..RenderHybridGiScenePrepareReadbackOutputs::default()
    }
}

fn scene_prepare_dispatch_as_usize(dispatch: [u32; 3]) -> [usize; 3] {
    [
        dispatch[0] as usize,
        dispatch[1] as usize,
        dispatch[2] as usize,
    ]
}

fn occupied_atlas_slots(frame: &HybridGiScenePrepareFrame) -> Vec<u32> {
    let mut slots = frame
        .card_capture_requests
        .iter()
        .map(|request| request.atlas_slot_id)
        .chain(
            frame
                .surface_cache_page_contents
                .iter()
                .map(|page| page.atlas_slot_id),
        )
        .collect::<BTreeSet<_>>();
    slots.retain(|slot| *slot != u32::MAX);
    slots.into_iter().collect()
}

fn occupied_capture_slots(frame: &HybridGiScenePrepareFrame) -> Vec<u32> {
    let mut slots = frame
        .card_capture_requests
        .iter()
        .map(|request| request.capture_slot_id)
        .chain(
            frame
                .surface_cache_page_contents
                .iter()
                .map(|page| page.capture_slot_id),
        )
        .collect::<BTreeSet<_>>();
    slots.retain(|slot| *slot != u32::MAX);
    slots.into_iter().collect()
}

fn atlas_samples_from_frame(
    frame: &HybridGiScenePrepareFrame,
) -> Vec<RenderHybridGiScenePrepareSample> {
    frame
        .surface_cache_page_contents
        .iter()
        .filter(|page| rgba_sample_is_present(page.atlas_sample_rgba))
        .map(|page| RenderHybridGiScenePrepareSample {
            index: page.atlas_slot_id,
            rgba8: page.atlas_sample_rgba,
        })
        .collect()
}

fn capture_samples_from_frame(
    frame: &HybridGiScenePrepareFrame,
) -> Vec<RenderHybridGiScenePrepareSample> {
    frame
        .surface_cache_page_contents
        .iter()
        .filter(|page| rgba_sample_is_present(page.capture_sample_rgba))
        .map(|page| RenderHybridGiScenePrepareSample {
            index: page.capture_slot_id,
            rgba8: page.capture_sample_rgba,
        })
        .collect()
}

fn surface_cache_depth_samples_from_frame(
    frame: &HybridGiScenePrepareFrame,
) -> Vec<RenderHybridGiScenePrepareSample> {
    let mut depth_by_atlas_slot = frame
        .surface_cache_page_contents
        .iter()
        .filter(|page| rgba_sample_is_present(page.atlas_sample_rgba))
        .filter(|page| page.atlas_slot_id != u32::MAX)
        .map(|page| {
            (
                page.atlas_slot_id,
                depth_rgba_from_bounds(page.bounds_center, page.bounds_radius),
            )
        })
        .collect::<BTreeMap<_, _>>();

    depth_by_atlas_slot.extend(
        frame
            .card_capture_requests
            .iter()
            .filter(|request| request.atlas_slot_id != u32::MAX)
            .map(|request| {
                (
                    request.atlas_slot_id,
                    depth_rgba_from_bounds(request.bounds_center, request.bounds_radius),
                )
            }),
    );

    occupied_atlas_slots(frame)
        .into_iter()
        .filter_map(|slot_id| {
            depth_by_atlas_slot.get(&slot_id).copied().map(|rgba8| {
                RenderHybridGiScenePrepareSample {
                    index: slot_id,
                    rgba8,
                }
            })
        })
        .collect()
}

fn depth_rgba_from_bounds(bounds_center: Vec3, bounds_radius: f32) -> [u8; 4] {
    let radius = bounds_radius.max(0.0);
    let depth = (bounds_center.z.abs() + radius)
        / (bounds_center.length() + radius + 1.0).max(f32::EPSILON);
    let encoded = (depth.clamp(0.0, 1.0) * 255.0).round() as u8;
    [encoded, encoded, encoded, u8::MAX]
}

fn probe_trace_tiles_from_frame(
    frame: &HybridGiScenePrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> Vec<RenderHybridGiTraceTileRecord> {
    let budget = probe_trace_tile_budget(extract);
    if budget == 0 {
        return Vec::new();
    }

    let mut tiles = frame
        .voxel_cells
        .iter()
        .filter(|cell| scene_prepare_voxel_cell_has_feedback(**cell))
        .map(|cell| {
            (
                cell.clipmap_id,
                cell.cell_index,
                probe_trace_ray_count(cell.occupancy_count, extract),
            )
        })
        .collect::<Vec<_>>();
    if tiles.is_empty() {
        tiles = surface_cache_trace_tiles_from_frame(frame, extract);
    }

    tiles.truncate(budget);
    tiles
        .into_iter()
        .enumerate()
        .map(
            |(tile_id, (probe_id, trace_region_id, ray_count))| RenderHybridGiTraceTileRecord {
                tile_id: tile_id as u32,
                probe_id,
                trace_region_id,
                ray_count,
            },
        )
        .collect()
}

fn surface_cache_trace_tiles_from_frame(
    frame: &HybridGiScenePrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> Vec<(u32, u32, u32)> {
    let mut tiles = frame
        .surface_cache_page_contents
        .iter()
        .filter(|page| rgba_sample_is_present(page.atlas_sample_rgba))
        .map(|page| {
            (
                page.owner_card_id,
                page.page_id,
                probe_trace_ray_count(1, extract),
            )
        })
        .collect::<BTreeSet<_>>();
    tiles.extend(frame.card_capture_requests.iter().map(|request| {
        (
            request.card_id,
            request.page_id,
            probe_trace_ray_count(1, extract),
        )
    }));
    tiles.into_iter().collect()
}

fn probe_trace_tile_budget(extract: Option<&RenderHybridGiExtract>) -> usize {
    extract
        .map(|extract| extract.trace_budget.max(extract.tracing_budget) as usize)
        .unwrap_or(usize::MAX)
}

fn probe_trace_ray_count(occupancy_count: u32, extract: Option<&RenderHybridGiExtract>) -> u32 {
    extract
        .map(|extract| extract.tracing_budget)
        .unwrap_or_default()
        .max(occupancy_count.max(1).saturating_mul(8))
        .max(1)
}

fn probe_trace_dispatch_from_tile_count(tile_count: usize) -> [u32; 3] {
    if tile_count == 0 {
        [0; 3]
    } else {
        [1, 1, tile_count as u32]
    }
}

fn scene_prepare_feedback_clipmap_ids(frame: &HybridGiScenePrepareFrame) -> Vec<u32> {
    frame
        .voxel_cells
        .iter()
        .filter(|cell| scene_prepare_voxel_cell_has_feedback(**cell))
        .map(|cell| cell.clipmap_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn voxel_occupancy_for_clipmap(frame: &HybridGiScenePrepareFrame, clipmap_id: u32) -> u32 {
    frame
        .voxel_cells
        .iter()
        .filter(|cell| cell.clipmap_id == clipmap_id)
        .fold(0_u32, |acc, cell| acc.saturating_add(cell.occupancy_count))
}

fn voxel_samples_from_frame(
    frame: &HybridGiScenePrepareFrame,
    clipmap_ids: &[u32],
) -> Vec<RenderHybridGiScenePrepareSample> {
    clipmap_ids
        .iter()
        .filter_map(|clipmap_id| {
            average_radiance_for_clipmap(frame, *clipmap_id).map(|rgba8| {
                RenderHybridGiScenePrepareSample {
                    index: *clipmap_id,
                    rgba8,
                }
            })
        })
        .collect()
}

fn average_radiance_for_clipmap(
    frame: &HybridGiScenePrepareFrame,
    clipmap_id: u32,
) -> Option<[u8; 4]> {
    let mut sample_count = 0_u32;
    let mut radiance_sum = [0_u32; 3];
    for cell in frame
        .voxel_cells
        .iter()
        .filter(|cell| cell.clipmap_id == clipmap_id && cell.radiance_present)
    {
        sample_count = sample_count.saturating_add(1);
        radiance_sum[0] = radiance_sum[0].saturating_add(u32::from(cell.radiance_rgb[0]));
        radiance_sum[1] = radiance_sum[1].saturating_add(u32::from(cell.radiance_rgb[1]));
        radiance_sum[2] = radiance_sum[2].saturating_add(u32::from(cell.radiance_rgb[2]));
    }
    (sample_count > 0).then(|| {
        [
            (radiance_sum[0] / sample_count) as u8,
            (radiance_sum[1] / sample_count) as u8,
            (radiance_sum[2] / sample_count) as u8,
            u8::MAX,
        ]
    })
}

fn voxel_occupancy_masks_from_frame(
    frame: &HybridGiScenePrepareFrame,
    clipmap_ids: &[u32],
) -> Vec<RenderHybridGiVoxelOccupancyMaskRecord> {
    clipmap_ids
        .iter()
        .filter_map(|clipmap_id| {
            let occupancy_mask = frame
                .voxel_cells
                .iter()
                .filter(|cell| cell.clipmap_id == *clipmap_id)
                .filter(|cell| cell.occupancy_count > 0 && cell.cell_index < u64::BITS)
                .fold(0_u64, |mask, cell| mask | (1_u64 << cell.cell_index));
            (occupancy_mask != 0).then_some(RenderHybridGiVoxelOccupancyMaskRecord {
                clipmap_id: *clipmap_id,
                occupancy_mask,
            })
        })
        .collect()
}

fn voxel_cells_from_frame(frame: &HybridGiScenePrepareFrame) -> Vec<RenderHybridGiVoxelCellRecord> {
    frame
        .voxel_cells
        .iter()
        .copied()
        .filter(|cell| scene_prepare_voxel_cell_has_feedback(*cell))
        .map(|cell| RenderHybridGiVoxelCellRecord {
            clipmap_id: cell.clipmap_id,
            cell_id: cell.cell_index,
            occupancy: cell.occupancy_count,
        })
        .collect()
}

fn voxel_cell_samples_from_frame(
    frame: &HybridGiScenePrepareFrame,
) -> Vec<RenderHybridGiVoxelCellSampleRecord> {
    frame
        .voxel_cells
        .iter()
        .filter(|cell| cell.radiance_present)
        .map(|cell| RenderHybridGiVoxelCellSampleRecord {
            clipmap_id: cell.clipmap_id,
            cell_id: cell.cell_index,
            rgba8: [
                cell.radiance_rgb[0],
                cell.radiance_rgb[1],
                cell.radiance_rgb[2],
                u8::MAX,
            ],
        })
        .collect()
}

fn voxel_cell_dominant_nodes_from_frame(
    frame: &HybridGiScenePrepareFrame,
) -> Vec<RenderHybridGiVoxelCellDominantNodeRecord> {
    frame
        .voxel_cells
        .iter()
        .filter(|cell| cell.dominant_card_id != 0)
        .map(|cell| RenderHybridGiVoxelCellDominantNodeRecord {
            clipmap_id: cell.clipmap_id,
            cell_id: cell.cell_index,
            dominant_node_id: u64::from(cell.dominant_card_id),
        })
        .collect()
}

fn voxel_cell_dominant_samples_from_frame(
    frame: &HybridGiScenePrepareFrame,
) -> Vec<RenderHybridGiVoxelCellSampleRecord> {
    frame
        .voxel_cells
        .iter()
        .filter(|cell| cell.dominant_card_id != 0 && cell.radiance_present)
        .map(|cell| RenderHybridGiVoxelCellSampleRecord {
            clipmap_id: cell.clipmap_id,
            cell_id: cell.cell_index,
            rgba8: [
                cell.radiance_rgb[0],
                cell.radiance_rgb[1],
                cell.radiance_rgb[2],
                u8::MAX,
            ],
        })
        .collect()
}

fn scene_prepare_voxel_cell_has_feedback(cell: HybridGiPrepareVoxelCell) -> bool {
    cell.occupancy_count > 0 || cell.dominant_card_id != 0 || cell.radiance_present
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
mod tests {
    use super::*;
    use crate::hybrid_gi::{
        HybridGiPrepareProbe, HybridGiResolveProbeSceneData, HybridGiScenePrepareResourceSamples,
    };
    use zircon_runtime::core::framework::render::{
        render_mesh_stable_instance_key, render_mesh_transform_revision, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderHybridGiScenePrepareSample, RenderHybridGiTraceRegion,
        RenderHybridGiVoxelCellDominantNodeRecord, RenderHybridGiVoxelCellRecord,
        RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelOccupancyMaskRecord,
        RenderLayerSet, RenderMeshSnapshot, RenderMeshStaticState,
    };
    use zircon_runtime::core::framework::scene::Mobility;
    use zircon_runtime::core::math::{Transform, Vec4};
    use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
    use zircon_runtime::graphics::{VisibilityHybridGiFeedback, VisibilityHybridGiUpdatePlan};

    #[test]
    fn provider_updates_plugin_runtime_state_through_neutral_contract() {
        let provider = PluginHybridGiRuntimeProvider;
        let mut state = provider.create_state();
        let extract = probe_extract();
        let plan = VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![100],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        };

        let prepare = state.prepare_frame(HybridGiRuntimePrepareInput::new(
            Some(&extract),
            &[],
            &[],
            &[],
            &[],
            Some(&plan),
            7,
        ));
        let update = state.update_after_render(HybridGiRuntimeFeedback::new(
            None,
            Some(VisibilityHybridGiFeedback {
                active_probe_ids: vec![100],
                requested_probe_ids: vec![100],
                scheduled_trace_region_ids: vec![40],
                evictable_probe_ids: prepare.into_evictable_probe_ids(),
            }),
        ));
        let stats = update.stats();

        assert_eq!(stats.resident_probe_count(), 1);
        assert_eq!(stats.scheduled_trace_region_count(), 1);
    }

    #[test]
    fn provider_projects_scene_screen_probes_into_neutral_prepared_frame_sideband() {
        let provider = PluginHybridGiRuntimeProvider;
        let mut state = provider.create_state();
        let mut extract = scene_prepare_extract();
        extract.trace_budget = 2;
        let meshes = vec![
            scene_prepare_mesh(11, Vec3::new(-1.0, 0.0, 0.0), Vec4::ONE),
            scene_prepare_mesh(22, Vec3::new(3.0, 0.0, 0.0), Vec4::ONE),
        ];

        let prepare = state.prepare_frame(HybridGiRuntimePrepareInput::new(
            Some(&extract),
            &meshes,
            &[],
            &[],
            &[],
            None,
            7,
        ));

        let prepared_frame = prepare
            .prepared_frame()
            .expect("scene screen probes should be projected into neutral prepared frame");
        assert_eq!(prepared_frame.resident_probes.len(), 2);
        assert_eq!(prepared_frame.resident_probes[0].probe_id, 0);
        assert_eq!(prepared_frame.resident_probes[0].slot, 0);
        assert_eq!(prepared_frame.resident_probes[0].ray_budget, 1);
        assert_eq!(prepared_frame.resident_probes[1].probe_id, 1);
        assert!(
            prepared_frame
                .resident_probes
                .iter()
                .all(|probe| probe.irradiance_rgb != [0, 0, 0]),
            "screen-probe prepared sideband should carry radiance cache seeds"
        );
        assert_eq!(prepared_frame.probe_scene_data.len(), 2);
        assert_eq!(prepared_frame.probe_scene_data[0].probe_id, 0);
        assert_eq!(prepared_frame.probe_scene_data[0].position_x_q, 1984);
        assert_eq!(prepared_frame.probe_scene_data[0].radius_q, 96);
        assert_eq!(prepared_frame.probe_scene_data[1].probe_id, 1);
        assert_eq!(prepared_frame.probe_scene_data[1].position_x_q, 2240);
        assert_eq!(prepared_frame.probe_scene_data[1].radius_q, 96);
    }

    #[test]
    fn provider_projects_probe_rt_lighting_history_into_neutral_prepared_frame_sideband() {
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 77,
                slot: 0,
                ray_budget: 24,
                irradiance_rgb: [4, 8, 12],
            }],
            ..HybridGiPrepareFrame::default()
        };
        let resolve_runtime = HybridGiResolveRuntime::new(
            BTreeMap::from([(77, HybridGiResolveProbeSceneData::new(2000, 2010, 2020, 96))]),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(77, [96, 48, 24])]),
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
        );

        let prepared_frame = neutral_prepared_frame_from_prepare(&prepare, &resolve_runtime);

        assert_eq!(prepared_frame.probe_scene_data[0].probe_id, 77);
        assert_eq!(prepared_frame.probe_rt_lighting_rgb.len(), 1);
        assert_eq!(prepared_frame.probe_rt_lighting_rgb[0].probe_id, 77);
        assert_eq!(
            prepared_frame.probe_rt_lighting_rgb[0].rt_lighting_rgb,
            [96, 48, 24]
        );
    }

    #[test]
    fn provider_projects_neutral_voxel_readback_into_scene_prepare_resources() {
        let readback = RenderHybridGiScenePrepareReadbackOutputs {
            voxel_cells: vec![RenderHybridGiVoxelCellRecord {
                clipmap_id: 4,
                cell_id: 9,
                occupancy: 3,
            }],
            voxel_cell_dominant_nodes: vec![RenderHybridGiVoxelCellDominantNodeRecord {
                clipmap_id: 4,
                cell_id: 9,
                dominant_node_id: 77,
            }],
            voxel_cell_samples: vec![RenderHybridGiVoxelCellSampleRecord {
                clipmap_id: 4,
                cell_id: 9,
                rgba8: [16, 24, 32, 255],
            }],
            voxel_cell_dominant_samples: vec![RenderHybridGiVoxelCellSampleRecord {
                clipmap_id: 4,
                cell_id: 9,
                rgba8: [48, 56, 64, 255],
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        };

        let resources = scene_prepare_resources_from_readback(Some(&readback))
            .expect("voxel cell readback should be runtime-consumable");

        assert_eq!(
            resources.voxel_cells(),
            &[HybridGiPrepareVoxelCell {
                clipmap_id: 4,
                cell_index: 9,
                occupancy_count: 3,
                dominant_card_id: 77,
                radiance_present: true,
                radiance_rgb: [48, 56, 64],
            }]
        );
    }

    #[test]
    fn provider_prepare_frame_projects_scene_prepare_frame_into_neutral_renderer_outputs() {
        let provider = PluginHybridGiRuntimeProvider;
        let mut state = provider.create_state();
        let extract = scene_prepare_extract();
        let mesh = scene_prepare_mesh(77, Vec3::ZERO, Vec4::new(1.0, 0.45, 0.2, 1.0));

        let prepare = state.prepare_frame(HybridGiRuntimePrepareInput::new(
            Some(&extract),
            &[mesh],
            &[],
            &[],
            &[],
            None,
            11,
        ));
        let scene_prepare = &prepare.renderer_outputs().hybrid_gi.scene_prepare;

        assert!(
            scene_prepare.has_runtime_feedback_payload(),
            "scene-representation voxel payload should cross the neutral prepare sideband"
        );
        assert_eq!(scene_prepare.voxel_clipmap_ids, vec![0]);
        assert!(scene_prepare
            .voxel_cells
            .iter()
            .any(|cell| cell.clipmap_id == 0 && cell.occupancy > 0));
        assert!(scene_prepare
            .voxel_occupancy_masks
            .iter()
            .any(|mask| mask.clipmap_id == 0 && mask.occupancy_mask != 0));
        assert!(scene_prepare
            .voxel_cell_dominant_nodes
            .iter()
            .any(|cell| cell.dominant_node_id == 77));
        assert!(scene_prepare
            .voxel_cell_dominant_samples
            .iter()
            .any(|sample| sample.rgba8[3] == u8::MAX));
        assert!(
            !scene_prepare.surface_cache_depth_samples.is_empty(),
            "surface-cache depth copy samples should cross the neutral prepare sideband"
        );
        assert!(scene_prepare
            .surface_cache_depth_samples
            .iter()
            .any(|sample| sample.rgba8[0] == sample.rgba8[1]
                && sample.rgba8[1] == sample.rgba8[2]
                && sample.rgba8[3] == u8::MAX));
        assert!(
            !scene_prepare.probe_trace_tiles.is_empty(),
            "probe trace tile schedule should cross the neutral prepare sideband"
        );
        assert_eq!(scene_prepare.probe_trace_dispatch, [1, 1, 1]);
    }

    #[test]
    fn provider_projects_neutral_voxel_mask_readback_into_fallback_cells() {
        let readback = RenderHybridGiScenePrepareReadbackOutputs {
            voxel_samples: vec![RenderHybridGiScenePrepareSample {
                index: 4,
                rgba8: [20, 40, 60, 255],
            }],
            voxel_occupancy_masks: vec![RenderHybridGiVoxelOccupancyMaskRecord {
                clipmap_id: 4,
                occupancy_mask: 0b1010,
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        };

        let resources = scene_prepare_resources_from_readback(Some(&readback))
            .expect("voxel occupancy mask readback should create fallback cells");

        assert_eq!(
            resources.voxel_cells(),
            &[
                HybridGiPrepareVoxelCell {
                    clipmap_id: 4,
                    cell_index: 1,
                    occupancy_count: 1,
                    dominant_card_id: 0,
                    radiance_present: true,
                    radiance_rgb: [20, 40, 60],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 4,
                    cell_index: 3,
                    occupancy_count: 1,
                    dominant_card_id: 0,
                    radiance_present: true,
                    radiance_rgb: [20, 40, 60],
                },
            ]
        );
    }

    #[test]
    fn provider_projects_neutral_voxel_aggregate_count_into_low_detail_fallback_cell() {
        let readback = RenderHybridGiScenePrepareReadbackOutputs {
            voxel_clipmap_ids: vec![8],
            voxel_occupancy: vec![5],
            voxel_samples: vec![RenderHybridGiScenePrepareSample {
                index: 8,
                rgba8: [72, 88, 104, 255],
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        };

        let resources = scene_prepare_resources_from_readback(Some(&readback))
            .expect("aggregate voxel occupancy should create a low-detail fallback cell");

        assert_eq!(
            resources.voxel_cells(),
            &[HybridGiPrepareVoxelCell {
                clipmap_id: 8,
                cell_index: LOW_DETAIL_VOXEL_FALLBACK_CELL_INDEX,
                occupancy_count: 5,
                dominant_card_id: 0,
                radiance_present: true,
                radiance_rgb: [72, 88, 104],
            }]
        );
    }

    fn probe_extract() -> RenderHybridGiExtract {
        RenderHybridGiExtract {
            enabled: true,
            probe_budget: 1,
            tracing_budget: 32,
            probes: vec![RenderHybridGiProbe {
                probe_id: 100,
                resident: true,
                ray_budget: 32,
                radius: 4.0,
                position: Vec3::ZERO,
                ..RenderHybridGiProbe::default()
            }],
            trace_regions: vec![RenderHybridGiTraceRegion {
                region_id: 40,
                bounds_radius: 4.0,
                screen_coverage: 1.0,
                rt_lighting_rgb: [96, 128, 160],
                ..RenderHybridGiTraceRegion::default()
            }],
            ..RenderHybridGiExtract::default()
        }
    }

    fn scene_prepare_extract() -> RenderHybridGiExtract {
        RenderHybridGiExtract {
            enabled: true,
            probe_budget: 0,
            tracing_budget: 0,
            trace_budget: 1,
            card_budget: 1,
            voxel_budget: 1,
            probes: Vec::new(),
            trace_regions: Vec::new(),
            ..RenderHybridGiExtract::default()
        }
    }

    fn scene_prepare_mesh(node_id: u64, translation: Vec3, tint: Vec4) -> RenderMeshSnapshot {
        let transform = Transform::from_translation(translation).with_scale(Vec3::splat(2.0));
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
            transform_revision: render_mesh_transform_revision(&transform),
            transform,
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/provider-scene-prepare-card.obj",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/provider-scene-prepare-card.mat",
            )),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint,
            mobility: Mobility::Static,
            static_state: RenderMeshStaticState::from_transform_static(true),
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        }
    }
}
