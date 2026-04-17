use crate::types::HybridGiPrepareFrame;

use super::super::hybrid_gi_runtime_state::HybridGiRuntimeState;
use super::collect_pending_updates::collect_pending_updates;
use super::collect_resident_probes::collect_resident_probes;

impl HybridGiRuntimeState {
    pub(crate) fn build_prepare_frame(&self) -> HybridGiPrepareFrame {
        HybridGiPrepareFrame {
            resident_probes: collect_resident_probes(self),
            pending_updates: collect_pending_updates(self),
            scheduled_trace_region_ids: self.scheduled_trace_regions.clone(),
            evictable_probe_ids: self.evictable_probes.clone(),
        }
    }
}
