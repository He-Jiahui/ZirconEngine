#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HybridGiPrepareUpdateRequest {
    pub probe_id: u32,
    pub ray_budget: u32,
    pub generation: u64,
}
