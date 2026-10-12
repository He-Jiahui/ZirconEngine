use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi::residency_management) fn take_free_slot(&mut self) -> Option<u32> {
        let slot = self.first_free_slot()?;
        self.remove_free_slot(slot);
        Some(slot)
    }
}
