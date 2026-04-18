#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiProbeUpdateRequest {
    pub(crate) probe_id: u32,
    pub(crate) ray_budget: u32,
    pub(crate) generation: u64,
}
