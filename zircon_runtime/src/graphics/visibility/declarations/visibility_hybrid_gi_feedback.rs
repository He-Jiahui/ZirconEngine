/// 供 Hybrid GI runtime 反馈读取的可见性侧结果；其有效性取决于本帧是否填充探针计划。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityHybridGiFeedback {
    pub active_probe_ids: Vec<u32>,
    pub requested_probe_ids: Vec<u32>,
    pub scheduled_trace_region_ids: Vec<u32>,
    pub evictable_probe_ids: Vec<u32>,
}
