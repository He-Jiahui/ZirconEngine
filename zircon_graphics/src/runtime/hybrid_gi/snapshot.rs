use super::hybrid_gi_runtime_snapshot::HybridGiRuntimeSnapshot;
use super::hybrid_gi_runtime_state::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn snapshot(&self) -> HybridGiRuntimeSnapshot {
        HybridGiRuntimeSnapshot {
            cache_entry_count: self.resident_slots.len(),
            resident_probe_count: self.resident_slots.len(),
            pending_update_count: self.pending_updates.len(),
            scheduled_trace_region_count: self.scheduled_trace_regions.len(),
        }
    }
}
