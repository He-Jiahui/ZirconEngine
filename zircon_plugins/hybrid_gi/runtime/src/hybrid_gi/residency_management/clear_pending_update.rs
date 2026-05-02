use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi::residency_management) fn clear_pending_update(
        &mut self,
        probe_id: u32,
    ) {
        self.remove_pending_probe(probe_id);
        self.retain_pending_update_requests(|update| update.probe_id() != probe_id);
    }
}
