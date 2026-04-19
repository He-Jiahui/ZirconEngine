#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiPrepareUpdateRequest {
    pub(crate) probe_id: u32,
    pub(crate) ray_budget: u32,
    pub(crate) generation: u64,
}
