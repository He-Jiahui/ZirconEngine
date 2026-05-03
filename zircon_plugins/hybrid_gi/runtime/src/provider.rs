use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::{
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiVoxelCellSampleRecord,
};
use zircon_runtime::graphics::{
    HybridGiGpuCompletion as RuntimeHybridGiGpuCompletion, HybridGiRuntimeFeedback,
    HybridGiRuntimePrepareInput, HybridGiRuntimePrepareOutput, HybridGiRuntimeProvider,
    HybridGiRuntimeState as RuntimeStateContract, HybridGiRuntimeStats, HybridGiRuntimeUpdate,
};

use crate::hybrid_gi::{
    HybridGiPrepareVoxelCell, HybridGiRuntimeScenePrepareResources, HybridGiRuntimeState,
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
        HybridGiRuntimePrepareOutput::new(prepare.evictable_probe_ids)
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
        self.state.complete_gpu_updates(
            completion.completed_probe_ids().iter().copied(),
            completion.completed_trace_region_ids().iter().copied(),
            completion.probe_irradiance_rgb(),
            completion.probe_trace_lighting_rgb(),
            evictable_probe_ids,
        );
    }
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
    use crate::hybrid_gi::HybridGiScenePrepareResourceSamples;
    use zircon_runtime::core::framework::render::{
        RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiScenePrepareSample,
        RenderHybridGiTraceRegion, RenderHybridGiVoxelCellDominantNodeRecord,
        RenderHybridGiVoxelCellRecord, RenderHybridGiVoxelCellSampleRecord,
        RenderHybridGiVoxelOccupancyMaskRecord,
    };
    use zircon_runtime::core::math::Vec3;
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
}
