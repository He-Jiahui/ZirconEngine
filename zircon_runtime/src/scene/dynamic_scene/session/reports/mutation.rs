use super::super::RuntimeSessionMetadata;

/// 当次预览的槽位身份、元数据与场景规模；不携带档案版本，也不是可保存后复用的提交凭据。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotMutationPreviewReport {
    pub source_slot_id: String,
    /// 仅改名预览提供规范化后的目标 ID；其他变更没有目标 ID。
    pub destination_slot_id: Option<String>,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}
