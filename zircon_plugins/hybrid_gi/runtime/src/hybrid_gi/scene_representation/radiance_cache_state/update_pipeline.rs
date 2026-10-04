use std::collections::BTreeMap;

use super::{HybridGiRadianceCacheSample, HybridGiRadianceProbeDemand};

/// The update sequence mirrors the GPU ownership boundary even while the CPU scene owner
/// supplies the first bounded RC implementation. A generation becomes visible only after
/// every stage has completed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum HybridGiRadianceCacheUpdateStage {
    #[default]
    Idle,
    Marked,
    TraceAllocated,
    Traced,
    Filtered,
    BordersFixed,
    MipsGenerated,
    Complete,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HybridGiRadianceCacheUpdateReport {
    generation: u64,
    stage: HybridGiRadianceCacheUpdateStage,
    marked_demand_count: usize,
    trace_tile_count: usize,
    filtered_probe_count: usize,
    border_fixed_probe_count: usize,
    mip_completed_probe_count: usize,
}

impl HybridGiRadianceCacheUpdateReport {
    pub(super) fn begin(&mut self, generation: u64, marked_demand_count: usize) {
        *self = Self {
            generation,
            stage: HybridGiRadianceCacheUpdateStage::Marked,
            marked_demand_count,
            ..Self::default()
        };
    }

    pub(super) fn allocate_trace_tiles(&mut self) {
        debug_assert_eq!(self.stage, HybridGiRadianceCacheUpdateStage::Marked);
        self.trace_tile_count = self.marked_demand_count;
        self.stage = HybridGiRadianceCacheUpdateStage::TraceAllocated;
    }

    pub(super) fn trace(&mut self) {
        debug_assert_eq!(self.stage, HybridGiRadianceCacheUpdateStage::TraceAllocated);
        self.stage = HybridGiRadianceCacheUpdateStage::Traced;
    }

    pub(super) fn filter(&mut self) {
        debug_assert_eq!(self.stage, HybridGiRadianceCacheUpdateStage::Traced);
        self.filtered_probe_count = self.trace_tile_count;
        self.stage = HybridGiRadianceCacheUpdateStage::Filtered;
    }

    pub(super) fn fixup_borders(&mut self) {
        debug_assert_eq!(self.stage, HybridGiRadianceCacheUpdateStage::Filtered);
        self.border_fixed_probe_count = self.filtered_probe_count;
        self.stage = HybridGiRadianceCacheUpdateStage::BordersFixed;
    }

    pub(super) fn generate_mips(&mut self) {
        debug_assert_eq!(self.stage, HybridGiRadianceCacheUpdateStage::BordersFixed);
        self.mip_completed_probe_count = self.border_fixed_probe_count;
        self.stage = HybridGiRadianceCacheUpdateStage::MipsGenerated;
    }

    pub(super) fn complete(&mut self) {
        debug_assert_eq!(self.stage, HybridGiRadianceCacheUpdateStage::MipsGenerated);
        self.stage = HybridGiRadianceCacheUpdateStage::Complete;
    }

    pub(super) fn mark_stable_generation(&mut self, generation: u64) {
        *self = Self {
            generation,
            stage: HybridGiRadianceCacheUpdateStage::Complete,
            ..Self::default()
        };
    }

    pub(super) fn generation_is_visible(&self, generation: u64) -> bool {
        self.stage == HybridGiRadianceCacheUpdateStage::Complete && self.generation == generation
    }

    pub(super) fn marked_demand_count(&self) -> usize {
        self.marked_demand_count
    }

    #[cfg(test)]
    pub(super) fn stage(&self) -> HybridGiRadianceCacheUpdateStage {
        self.stage
    }

    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.marked_demand_count,
            self.trace_tile_count,
            self.filtered_probe_count,
            self.border_fixed_probe_count,
            self.mip_completed_probe_count,
        )
    }
}

pub(super) fn advance_radiance_cache_update_to_mips(
    report: &mut HybridGiRadianceCacheUpdateReport,
    generation: u64,
    marked_demand_count: usize,
    samples: BTreeMap<HybridGiRadianceProbeDemand, HybridGiRadianceCacheSample>,
) -> BTreeMap<HybridGiRadianceProbeDemand, HybridGiRadianceCacheSample> {
    report.begin(generation, marked_demand_count);
    report.allocate_trace_tiles();
    report.trace();
    report.filter();
    report.fixup_borders();
    report.generate_mips();
    samples
}

#[cfg(test)]
#[path = "tests/update_pipeline.rs"]
mod tests;
