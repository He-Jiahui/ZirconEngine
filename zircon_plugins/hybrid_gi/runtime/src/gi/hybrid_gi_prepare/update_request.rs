#[derive(Clone, Debug, PartialEq, Eq)]
/// 待追踪探针的射线预算和请求代数；准备帧投影时保留三者的对应关系。
pub struct HybridGiPrepareUpdateRequest {
    pub probe_id: u32,
    pub ray_budget: u32,
    pub generation: u64,
}
