use super::super::RuntimeSessionMetadata;

/// 当次捕获快照的摘要；替换标志依据当前档案，报告本身不锁定槽位或供后续提交消费。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotCapturePreviewReport {
    pub slot_id: String,
    pub will_replace_existing: bool,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}
