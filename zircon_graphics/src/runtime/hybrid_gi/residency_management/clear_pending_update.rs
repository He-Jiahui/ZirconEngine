use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::runtime::hybrid_gi::residency_management) fn clear_pending_update(
        &mut self,
        probe_id: u32,
    ) {
        self.pending_probes.remove(&probe_id);
        self.pending_updates
            .retain(|update| update.probe_id != probe_id);
    }
}
