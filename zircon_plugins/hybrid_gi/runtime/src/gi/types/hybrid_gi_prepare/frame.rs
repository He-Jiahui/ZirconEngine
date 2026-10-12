use super::{HybridGiPrepareProbe, HybridGiPrepareUpdateRequest};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 每帧探针提交快照；驻留项保留历史，待更新项带代数，回收候选交由渲染反馈处理。
pub struct HybridGiPrepareFrame {
    pub resident_probes: Vec<HybridGiPrepareProbe>,
    pub pending_updates: Vec<HybridGiPrepareUpdateRequest>,
    pub scheduled_trace_region_ids: Vec<u32>,
    pub evictable_probe_ids: Vec<u32>,
}
