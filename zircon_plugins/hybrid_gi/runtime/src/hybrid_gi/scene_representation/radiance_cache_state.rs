use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::math::Vec3;

use crate::hybrid_gi::HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT;

use super::{
    screen_probe_state::HybridGiScreenProbeDescriptor,
    surface_cache_state::HybridGiSurfaceCacheState, voxel_scene_state::HybridGiVoxelSceneState,
};

mod gpu_prepare;
mod interpolation;
mod sample_interpolation;
mod update_pipeline;

#[cfg(test)]
use interpolation::RADIANCE_CACHE_CLIPMAP_RESOLUTION;
use interpolation::{
    radiance_probe_demands_for_position, radiance_probe_interpolation_corners,
    rebuild_radiance_cache_clipmaps, HybridGiRadianceCacheClipmapDescriptor,
    HybridGiRadianceProbeDemand,
};
use sample_interpolation::HybridGiRadianceCacheInterpolationAccumulator;
#[cfg(test)]
use update_pipeline::HybridGiRadianceCacheUpdateStage;
use update_pipeline::{advance_radiance_cache_update_to_mips, HybridGiRadianceCacheUpdateReport};

const SURFACE_CACHE_CAPTURE_CONFIDENCE_Q8: u8 = 255;
const SURFACE_CACHE_ATLAS_CONFIDENCE_Q8: u8 = 220;
const VOXEL_FALLBACK_CONFIDENCE_Q8: u8 = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HybridGiRadianceCacheSource {
    SurfaceCache,
    VoxelFallback,
    Missing,
}

impl HybridGiRadianceCacheSource {
    #[cfg(test)]
    fn label(self) -> &'static str {
        match self {
            Self::SurfaceCache => "surface-cache",
            Self::VoxelFallback => "voxel-fallback",
            Self::Missing => "missing",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HybridGiRadianceCacheSample {
    radiance_rgb: [u8; 3],
    confidence_q8: u8,
    source: HybridGiRadianceCacheSource,
}

impl HybridGiRadianceCacheSample {
    const MISSING: Self = Self {
        radiance_rgb: [0, 0, 0],
        confidence_q8: 0,
        source: HybridGiRadianceCacheSource::Missing,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
struct HybridGiRadianceCacheEntry {
    probe_id: u32,
    card_id: u32,
    surface_page_id: Option<u32>,
    radiance_rgb: [u8; 3],
    confidence_q8: u8,
    source: HybridGiRadianceCacheSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HybridGiRadianceCacheResidentProbe {
    slot: u32,
    last_used_frame: u64,
    last_traced_frame: u64,
    generation: u64,
    participation_epoch: u64,
    sample: HybridGiRadianceCacheSample,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HybridGiRadianceCacheInputRevision {
    surface_cache_revision: u32,
    voxel_scene_revision: u32,
    participation_epoch: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::hybrid_gi::scene_representation) struct HybridGiRadianceCacheState {
    clipmaps: Vec<HybridGiRadianceCacheClipmapDescriptor>,
    probe_demands: Vec<HybridGiRadianceProbeDemand>,
    selected_demands: Vec<HybridGiRadianceProbeDemand>,
    entries: Vec<HybridGiRadianceCacheEntry>,
    resident_probes: BTreeMap<HybridGiRadianceProbeDemand, HybridGiRadianceCacheResidentProbe>,
    free_slots: BTreeSet<u32>,
    frame_index: u64,
    generation: u64,
    input_revision: Option<HybridGiRadianceCacheInputRevision>,
    clipmap_anchor_cells: Vec<[i32; 3]>,
    scroll_count: u64,
    history_clear_count: u64,
    update_report: HybridGiRadianceCacheUpdateReport,
    gpu_update_demands: Vec<HybridGiRadianceProbeDemand>,
    #[cfg(test)]
    last_sampled_demand_count: usize,
}

impl Default for HybridGiRadianceCacheState {
    fn default() -> Self {
        Self {
            clipmaps: Vec::new(),
            probe_demands: Vec::new(),
            selected_demands: Vec::new(),
            entries: Vec::new(),
            resident_probes: BTreeMap::new(),
            free_slots: (0..HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT as u32).collect(),
            frame_index: 0,
            generation: 0,
            input_revision: None,
            clipmap_anchor_cells: Vec::new(),
            scroll_count: 0,
            history_clear_count: 0,
            update_report: HybridGiRadianceCacheUpdateReport::default(),
            gpu_update_demands: Vec::new(),
            #[cfg(test)]
            last_sampled_demand_count: 0,
        }
    }
}

impl HybridGiRadianceCacheState {
    pub(in crate::hybrid_gi::scene_representation) fn synchronize(
        &mut self,
        probes: &[HybridGiScreenProbeDescriptor],
        surface_cache: &HybridGiSurfaceCacheState,
        voxel_scene: &HybridGiVoxelSceneState,
        camera_position: Option<Vec3>,
        history_invalidated: bool,
        participation_epoch: u64,
    ) {
        self.frame_index = self.frame_index.saturating_add(1).max(1);
        if history_invalidated {
            self.clear_history();
            self.history_clear_count = self.history_clear_count.saturating_add(1);
        }
        if probes.is_empty() {
            self.clear_history();
            return;
        }

        rebuild_radiance_cache_clipmaps(&mut self.clipmaps, probes, camera_position);
        let clipmap_scrolled = !self.clipmap_anchor_cells.is_empty()
            && !self.clipmaps.is_empty()
            && (self.clipmap_anchor_cells.len() != self.clipmaps.len()
                || self
                    .clipmap_anchor_cells
                    .iter()
                    .zip(&self.clipmaps)
                    .any(|(previous, current)| *previous != current.anchor_cell));
        if clipmap_scrolled {
            self.scroll_count = self.scroll_count.saturating_add(1);
            let previous_anchor_cells = self.clipmap_anchor_cells.clone();
            self.propagate_resident_probes_for_scroll(&previous_anchor_cells);
        }
        self.clipmap_anchor_cells.clear();
        self.clipmap_anchor_cells
            .extend(self.clipmaps.iter().map(|clipmap| clipmap.anchor_cell));
        self.probe_demands = mark_radiance_probe_demands(probes, &self.clipmaps);
        self.refresh_selected_demands();
        // Move the retained allocation out while resident state mutates, then restore it below.
        let selected_demands = std::mem::take(&mut self.selected_demands);
        let input_revision = HybridGiRadianceCacheInputRevision {
            surface_cache_revision: surface_cache.scene_revision(),
            voxel_scene_revision: voxel_scene.scene_revision(),
            participation_epoch,
        };
        let source_inputs_changed = self.input_revision != Some(input_revision);
        let generation_changed = source_inputs_changed || clipmap_scrolled;
        if generation_changed {
            self.generation = self.generation.saturating_add(1).max(1);
            self.input_revision = Some(input_revision);
        }

        let demands_to_trace = selected_demands
            .iter()
            .copied()
            .filter(|demand| {
                source_inputs_changed
                    || self.resident_probes.get(demand).is_none_or(|resident| {
                        resident.participation_epoch != participation_epoch
                            || (!generation_changed && resident.generation != self.generation)
                    })
            })
            .collect::<Vec<_>>();
        self.gpu_update_demands.clear();
        if generation_changed {
            self.gpu_update_demands.extend_from_slice(&selected_demands);
        } else {
            self.gpu_update_demands.extend_from_slice(&demands_to_trace);
        }

        // Stable and overlapping scrolled cells consume committed samples without cloning sources.
        let samples = (!demands_to_trace.is_empty()).then(|| {
            radiance_cache_samples(
                probes,
                &self.clipmaps,
                &demands_to_trace,
                &surface_cache.page_contents_snapshot(),
                &voxel_scene.voxel_cells_snapshot(),
            )
        });
        let samples = samples.map(|samples| {
            advance_radiance_cache_update_to_mips(
                &mut self.update_report,
                self.generation,
                demands_to_trace.len(),
                samples,
            )
        });
        if samples.is_none() {
            self.update_report.mark_stable_generation(self.generation);
        }
        #[cfg(test)]
        {
            self.last_sampled_demand_count = samples.as_ref().map_or(0, BTreeMap::len);
        }
        self.synchronize_resident_probes(
            &selected_demands,
            &demands_to_trace,
            generation_changed,
            participation_epoch,
            samples.as_ref(),
        );
        self.selected_demands = selected_demands;
        if samples.is_some() {
            self.update_report.complete();
        }
        debug_assert!(self.update_report.generation_is_visible(self.generation));

        self.entries = probes
            .iter()
            .map(|probe| {
                let sample = self
                    .probe_current_sample(probe)
                    .unwrap_or(HybridGiRadianceCacheSample::MISSING);
                HybridGiRadianceCacheEntry {
                    probe_id: probe.probe_id(),
                    card_id: probe.card_id(),
                    surface_page_id: probe.surface_page_id(),
                    radiance_rgb: sample.radiance_rgb,
                    confidence_q8: sample.confidence_q8,
                    source: sample.source,
                }
            })
            .collect();
    }

    fn synchronize_resident_probes(
        &mut self,
        selected_demands: &[HybridGiRadianceProbeDemand],
        demands_to_trace: &[HybridGiRadianceProbeDemand],
        generation_changed: bool,
        participation_epoch: u64,
        samples: Option<&BTreeMap<HybridGiRadianceProbeDemand, HybridGiRadianceCacheSample>>,
    ) {
        let released_demands = self
            .resident_probes
            .keys()
            .filter(|demand| selected_demands.binary_search(*demand).is_err())
            .copied()
            .collect::<Vec<_>>();
        for demand in released_demands {
            let resident = self
                .resident_probes
                .remove(&demand)
                .expect("released radiance-cache demand remains resident");
            self.free_slots.insert(resident.slot);
        }

        for &demand in selected_demands {
            if !self.resident_probes.contains_key(&demand) {
                let slot = self
                    .free_slots
                    .pop_first()
                    .expect("bounded radiance-cache demand set has a free slot");
                self.resident_probes.insert(
                    demand,
                    HybridGiRadianceCacheResidentProbe {
                        slot,
                        last_used_frame: 0,
                        last_traced_frame: 0,
                        generation: 0,
                        participation_epoch: 0,
                        sample: HybridGiRadianceCacheSample::MISSING,
                    },
                );
            }
            let resident = self
                .resident_probes
                .get_mut(&demand)
                .expect("selected radiance-cache demand remains resident");
            resident.last_used_frame = self.frame_index;
            if demands_to_trace.binary_search(&demand).is_ok() {
                resident.last_traced_frame = self.frame_index;
                resident.generation = self.generation;
                resident.participation_epoch = participation_epoch;
                resident.sample = samples
                    .expect("radiance-cache retrace must provide source samples")
                    .get(&demand)
                    .copied()
                    .unwrap_or(HybridGiRadianceCacheSample::MISSING);
            } else if generation_changed {
                resident.generation = self.generation;
                resident.participation_epoch = participation_epoch;
            }
        }
    }

    fn refresh_selected_demands(&mut self) {
        self.selected_demands.clear();
        self.selected_demands.extend(
            self.probe_demands
                .iter()
                .copied()
                .take(HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT),
        );
    }

    fn propagate_resident_probes_for_scroll(&mut self, previous_anchor_cells: &[[i32; 3]]) {
        if previous_anchor_cells.len() != self.clipmaps.len() {
            return;
        }
        let anchor_delta_by_level = self
            .clipmaps
            .iter()
            .zip(previous_anchor_cells)
            .map(|(clipmap, previous)| {
                (
                    clipmap.level,
                    [
                        i64::from(clipmap.anchor_cell[0]) - i64::from(previous[0]),
                        i64::from(clipmap.anchor_cell[1]) - i64::from(previous[1]),
                        i64::from(clipmap.anchor_cell[2]) - i64::from(previous[2]),
                    ],
                )
            })
            .collect::<BTreeMap<_, _>>();
        let previous_residents = std::mem::take(&mut self.resident_probes);
        self.resident_probes = previous_residents
            .into_iter()
            .map(|(demand, resident)| {
                let probe_coord = anchor_delta_by_level
                    .get(&demand.clipmap_level)
                    .and_then(|delta| {
                        Some([
                            i32::try_from(i64::from(demand.probe_coord[0]) - delta[0]).ok()?,
                            i32::try_from(i64::from(demand.probe_coord[1]) - delta[1]).ok()?,
                            i32::try_from(i64::from(demand.probe_coord[2]) - delta[2]).ok()?,
                        ])
                    })
                    .unwrap_or(demand.probe_coord);
                (
                    HybridGiRadianceProbeDemand {
                        clipmap_level: demand.clipmap_level,
                        probe_coord,
                    },
                    resident,
                )
            })
            .collect();
    }

    fn probe_current_sample(
        &self,
        probe: &HybridGiScreenProbeDescriptor,
    ) -> Option<HybridGiRadianceCacheSample> {
        let Some(input_revision) = self.input_revision else {
            return None;
        };
        if !self.update_report.generation_is_visible(self.generation) {
            return None;
        }
        let corners = radiance_probe_interpolation_corners(probe.bounds_center(), &self.clipmaps);
        if corners.is_empty() {
            return None;
        }

        let mut interpolation = HybridGiRadianceCacheInterpolationAccumulator::default();
        for corner in corners {
            let resident = self.resident_probes.get(&corner.demand)?;
            // A mixed generation must never expose partially refreshed indirect light.
            if resident.generation != self.generation
                || resident.participation_epoch != input_revision.participation_epoch
            {
                return None;
            }
            interpolation.add(resident.sample, corner.weight_q16);
        }
        Some(interpolation.finish())
    }

    pub(in crate::hybrid_gi::scene_representation) fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub(in crate::hybrid_gi::scene_representation) fn resident_probe_count(&self) -> usize {
        self.resident_probes.len()
    }

    pub(in crate::hybrid_gi::scene_representation) fn truncated_demand_count(&self) -> usize {
        self.probe_demands
            .len()
            .saturating_sub(self.resident_probes.len())
    }

    pub(in crate::hybrid_gi::scene_representation) fn generation(&self) -> u64 {
        self.generation
    }

    pub(in crate::hybrid_gi::scene_representation) fn scroll_count(&self) -> u64 {
        self.scroll_count
    }

    pub(in crate::hybrid_gi::scene_representation) fn history_clear_count(&self) -> u64 {
        self.history_clear_count
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn update_stage(
        &self,
    ) -> HybridGiRadianceCacheUpdateStage {
        self.update_report.stage()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn update_counts(
        &self,
    ) -> (usize, usize, usize, usize, usize) {
        self.update_report.counts()
    }

    pub(in crate::hybrid_gi::scene_representation) fn radiance_rgb(
        &self,
        probe_id: u32,
    ) -> Option<[u8; 3]> {
        self.entries
            .iter()
            .find_map(|entry| (entry.probe_id == probe_id).then_some(entry.radiance_rgb))
    }

    pub(in crate::hybrid_gi::scene_representation) fn update_probe_count(&self) -> usize {
        self.update_report.marked_demand_count()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn resident_probes(
        &self,
    ) -> Vec<(u32, [i32; 3], u32, u64, u64, u64, u64)> {
        self.resident_probes
            .iter()
            .map(|(demand, resident)| {
                (
                    demand.clipmap_level,
                    demand.probe_coord,
                    resident.slot,
                    resident.generation,
                    resident.last_used_frame,
                    resident.last_traced_frame,
                    resident.participation_epoch,
                )
            })
            .collect()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn resident_samples(
        &self,
    ) -> Vec<(u32, [i32; 3], [u8; 3], u8, &'static str)> {
        self.resident_probes
            .iter()
            .map(|(demand, resident)| {
                (
                    demand.clipmap_level,
                    demand.probe_coord,
                    resident.sample.radiance_rgb,
                    resident.sample.confidence_q8,
                    resident.sample.source.label(),
                )
            })
            .collect()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn last_sampled_demand_count(&self) -> usize {
        self.last_sampled_demand_count
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn clipmap_topology(
        &self,
    ) -> Vec<(u32, u32, f32)> {
        self.clipmaps
            .iter()
            .map(|clipmap| (clipmap.level, clipmap.resolution, clipmap.cell_size))
            .collect()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn probe_demands(&self) -> Vec<(u32, [i32; 3])> {
        self.probe_demands
            .iter()
            .map(|demand| (demand.clipmap_level, demand.probe_coord))
            .collect()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn entries(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, [u8; 3], u8, &'static str)> {
        self.entries
            .iter()
            .map(|entry| {
                (
                    entry.probe_id,
                    entry.card_id,
                    entry.surface_page_id,
                    entry.radiance_rgb,
                    entry.confidence_q8,
                    entry.source.label(),
                )
            })
            .collect()
    }

    fn clear_history(&mut self) {
        self.clipmaps.clear();
        self.clipmap_anchor_cells.clear();
        self.probe_demands.clear();
        self.selected_demands.clear();
        self.entries.clear();
        self.resident_probes.clear();
        self.free_slots = (0..HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT as u32).collect();
        self.input_revision = None;
        self.update_report = HybridGiRadianceCacheUpdateReport::default();
        self.gpu_update_demands.clear();
        #[cfg(test)]
        {
            self.last_sampled_demand_count = 0;
        }
    }
}

fn mark_radiance_probe_demands(
    probes: &[HybridGiScreenProbeDescriptor],
    clipmaps: &[HybridGiRadianceCacheClipmapDescriptor],
) -> Vec<HybridGiRadianceProbeDemand> {
    let mut demands = BTreeSet::new();

    for probe in probes {
        demands.extend(radiance_probe_demands_for_position(
            probe.bounds_center(),
            clipmaps,
        ));
    }

    demands.into_iter().collect()
}

fn radiance_cache_samples(
    probes: &[HybridGiScreenProbeDescriptor],
    clipmaps: &[HybridGiRadianceCacheClipmapDescriptor],
    selected_demands: &[HybridGiRadianceProbeDemand],
    surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
    voxel_cells: &[crate::hybrid_gi::HybridGiPrepareVoxelCell],
) -> BTreeMap<HybridGiRadianceProbeDemand, HybridGiRadianceCacheSample> {
    let mut samples = BTreeMap::new();
    for probe in probes {
        let demands = radiance_probe_demands_for_position(probe.bounds_center(), clipmaps);
        if !demands
            .iter()
            .any(|demand| selected_demands.binary_search(demand).is_ok())
        {
            continue;
        }
        let sample = radiance_sample_for_probe(probe, surface_cache_page_contents, voxel_cells);
        for demand in demands
            .into_iter()
            .filter(|demand| selected_demands.binary_search(demand).is_ok())
        {
            let previous = samples.entry(demand).or_insert(sample);
            if sample.confidence_q8 > previous.confidence_q8 {
                *previous = sample;
            }
        }
    }
    samples
}

fn radiance_sample_for_probe(
    probe: &HybridGiScreenProbeDescriptor,
    surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
    voxel_cells: &[crate::hybrid_gi::HybridGiPrepareVoxelCell],
) -> HybridGiRadianceCacheSample {
    probe
        .surface_page_id()
        .and_then(|surface_page_id| {
            surface_cache_radiance(surface_page_id, surface_cache_page_contents)
        })
        .or_else(|| voxel_fallback_radiance(probe.card_id(), voxel_cells))
        .unwrap_or(HybridGiRadianceCacheSample::MISSING)
}

fn surface_cache_radiance(
    surface_page_id: u32,
    surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
) -> Option<HybridGiRadianceCacheSample> {
    surface_cache_page_contents
        .iter()
        .find(|(page_id, _, _, _, _, _)| *page_id == surface_page_id)
        .and_then(|(_, _, _, _, atlas_sample_rgba, capture_sample_rgba)| {
            if capture_sample_rgba[3] > 0 {
                return Some(HybridGiRadianceCacheSample {
                    radiance_rgb: [
                        capture_sample_rgba[0],
                        capture_sample_rgba[1],
                        capture_sample_rgba[2],
                    ],
                    confidence_q8: SURFACE_CACHE_CAPTURE_CONFIDENCE_Q8,
                    source: HybridGiRadianceCacheSource::SurfaceCache,
                });
            }
            if atlas_sample_rgba[3] > 0 {
                return Some(HybridGiRadianceCacheSample {
                    radiance_rgb: [
                        atlas_sample_rgba[0],
                        atlas_sample_rgba[1],
                        atlas_sample_rgba[2],
                    ],
                    confidence_q8: SURFACE_CACHE_ATLAS_CONFIDENCE_Q8,
                    source: HybridGiRadianceCacheSource::SurfaceCache,
                });
            }
            None
        })
}

fn voxel_fallback_radiance(
    card_id: u32,
    voxel_cells: &[crate::hybrid_gi::HybridGiPrepareVoxelCell],
) -> Option<HybridGiRadianceCacheSample> {
    let mut best_radiance = None;
    let mut best_strength = 0_u32;

    for cell in voxel_cells {
        if cell.dominant_card_id != card_id || !cell.radiance_present {
            continue;
        }

        let strength = radiance_strength(cell.radiance_rgb);
        if best_radiance.is_none() || strength > best_strength {
            best_radiance = Some(cell.radiance_rgb);
            best_strength = strength;
        }
    }

    best_radiance.map(|radiance_rgb| HybridGiRadianceCacheSample {
        radiance_rgb,
        confidence_q8: VOXEL_FALLBACK_CONFIDENCE_Q8,
        source: HybridGiRadianceCacheSource::VoxelFallback,
    })
}

fn radiance_strength(radiance_rgb: [u8; 3]) -> u32 {
    radiance_rgb[0] as u32 + radiance_rgb[1] as u32 + radiance_rgb[2] as u32
}

#[cfg(test)]
mod tests;
