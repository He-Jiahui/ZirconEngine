use crate::hybrid_gi::{
    HybridGiPrepareRadianceCacheConsume, HybridGiPrepareRadianceCacheUpdate,
    HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT,
};

use super::super::screen_probe_state::HybridGiScreenProbeDescriptor;
use super::{radiance_probe_interpolation_corners, HybridGiRadianceCacheState};

impl HybridGiRadianceCacheState {
    pub(in crate::hybrid_gi::scene_representation) fn gpu_bootstrap_updates(
        &self,
    ) -> Vec<HybridGiPrepareRadianceCacheUpdate> {
        let Some(input_revision) = self.input_revision else {
            return Vec::new();
        };
        if !self.update_report.generation_is_visible(self.generation) {
            return Vec::new();
        }

        self.selected_demands
            .iter()
            .filter_map(|demand| {
                let resident = self.resident_probes.get(demand)?;
                (resident.generation == self.generation
                    && resident.participation_epoch == input_revision.participation_epoch)
                    .then_some(HybridGiPrepareRadianceCacheUpdate {
                        slot: resident.slot,
                        generation: resident.generation,
                        radiance_rgb: resident.sample.radiance_rgb,
                        confidence_q8: resident.sample.confidence_q8,
                        reuse_committed_radiance: false,
                    })
            })
            .collect()
    }

    pub(in crate::hybrid_gi::scene_representation) fn gpu_updates(
        &self,
    ) -> Vec<HybridGiPrepareRadianceCacheUpdate> {
        let Some(input_revision) = self.input_revision else {
            return Vec::new();
        };
        if self.gpu_update_demands.is_empty()
            || !self.update_report.generation_is_visible(self.generation)
        {
            return Vec::new();
        }

        self.gpu_update_demands
            .iter()
            .filter_map(|demand| {
                let resident = self.resident_probes.get(demand)?;
                (resident.generation == self.generation
                    && resident.participation_epoch == input_revision.participation_epoch)
                    .then_some(HybridGiPrepareRadianceCacheUpdate {
                        slot: resident.slot,
                        generation: resident.generation,
                        radiance_rgb: resident.sample.radiance_rgb,
                        confidence_q8: resident.sample.confidence_q8,
                        reuse_committed_radiance: resident.last_traced_frame != self.frame_index,
                    })
            })
            .collect()
    }

    pub(in crate::hybrid_gi::scene_representation) fn gpu_consumes(
        &self,
        probes: &[HybridGiScreenProbeDescriptor],
    ) -> Vec<HybridGiPrepareRadianceCacheConsume> {
        let Some(input_revision) = self.input_revision else {
            return Vec::new();
        };
        if !self.update_report.generation_is_visible(self.generation) {
            return Vec::new();
        }

        probes
            .iter()
            .filter_map(|probe| {
                let corners =
                    radiance_probe_interpolation_corners(probe.bounds_center(), &self.clipmaps);
                if corners.len() != HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT {
                    return None;
                }

                let mut slots = [0; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT];
                let mut weights_q16 = [0; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT];
                for (corner_index, corner) in corners.iter().enumerate() {
                    let resident = self.resident_probes.get(&corner.demand)?;
                    if resident.generation != self.generation
                        || resident.participation_epoch != input_revision.participation_epoch
                    {
                        return None;
                    }
                    slots[corner_index] = resident.slot;
                    weights_q16[corner_index] = corner.weight_q16.min(u64::from(u16::MAX)) as u16;
                }

                Some(HybridGiPrepareRadianceCacheConsume {
                    probe_id: probe.probe_id(),
                    generation: self.generation,
                    slots,
                    weights_q16,
                })
            })
            .collect()
    }
}
