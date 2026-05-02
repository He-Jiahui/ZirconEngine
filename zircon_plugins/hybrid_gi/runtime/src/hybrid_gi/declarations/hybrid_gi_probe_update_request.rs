#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiProbeUpdateRequest {
    probe_id: u32,
    ray_budget: u32,
    generation: u64,
}

impl HybridGiProbeUpdateRequest {
    pub(in crate::hybrid_gi) fn new(probe_id: u32, ray_budget: u32, generation: u64) -> Self {
        Self {
            probe_id,
            ray_budget,
            generation,
        }
    }

    pub(crate) fn probe_id(&self) -> u32 {
        self.probe_id
    }

    pub(crate) fn ray_budget(&self) -> u32 {
        self.ray_budget
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}
