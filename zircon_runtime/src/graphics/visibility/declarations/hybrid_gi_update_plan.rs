/// 可见性侧提交给 Hybrid GI provider 的探针需求和逐帧变化计划。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityHybridGiUpdatePlan {
    pub resident_probe_ids: Vec<u32>,
    pub requested_probe_ids: Vec<u32>,
    pub dirty_requested_probe_ids: Vec<u32>,
    pub scheduled_trace_region_ids: Vec<u32>,
    pub evictable_probe_ids: Vec<u32>,
}
