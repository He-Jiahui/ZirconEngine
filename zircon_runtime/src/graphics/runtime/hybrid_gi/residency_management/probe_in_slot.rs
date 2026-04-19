use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi::residency_management) fn probe_in_slot(
        &self,
        slot: u32,
    ) -> Option<u32> {
        self.resident_slots
            .iter()
            .find_map(|(&probe_id, &resident_slot)| (resident_slot == slot).then_some(probe_id))
    }
}
