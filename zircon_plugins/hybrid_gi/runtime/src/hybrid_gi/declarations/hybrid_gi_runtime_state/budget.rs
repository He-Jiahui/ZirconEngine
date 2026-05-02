use super::runtime_state::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi) fn probe_budget(&self) -> usize {
        self.probe_budget
    }

    pub(in crate::hybrid_gi) fn set_probe_budget(&mut self, probe_budget: usize) {
        self.probe_budget = probe_budget;
    }
}
