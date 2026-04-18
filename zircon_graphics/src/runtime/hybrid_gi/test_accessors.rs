use super::{HybridGiProbeResidencyState, HybridGiProbeUpdateRequest, HybridGiRuntimeState};

impl HybridGiRuntimeState {
    pub(crate) fn probe_slot(&self, probe_id: u32) -> Option<u32> {
        self.resident_slots.get(&probe_id).copied()
    }

    pub(crate) fn probe_residency(&self, probe_id: u32) -> Option<HybridGiProbeResidencyState> {
        if self.resident_slots.contains_key(&probe_id) {
            return Some(HybridGiProbeResidencyState::Resident);
        }
        if self.pending_probes.contains(&probe_id) {
            return Some(HybridGiProbeResidencyState::PendingUpdate);
        }
        None
    }

    pub(crate) fn pending_updates(&self) -> Vec<HybridGiProbeUpdateRequest> {
        self.pending_updates.clone()
    }

    pub(crate) fn scheduled_trace_regions(&self) -> Vec<u32> {
        self.scheduled_trace_regions.clone()
    }

    pub(crate) fn evictable_probes(&self) -> Vec<u32> {
        self.evictable_probes.clone()
    }

    pub(crate) fn apply_evictions(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if let Some(slot) = self.resident_slots.remove(&probe_id) {
                self.free_slots.insert(slot);
            }
        }
        self.evictable_probes
            .retain(|probe_id| self.resident_slots.contains_key(probe_id));
    }

    pub(crate) fn fulfill_updates(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if !self.pending_probes.remove(&probe_id) {
                continue;
            }

            self.pending_updates
                .retain(|update| update.probe_id != probe_id);
            self.promote_to_resident(probe_id);
        }
    }
}
