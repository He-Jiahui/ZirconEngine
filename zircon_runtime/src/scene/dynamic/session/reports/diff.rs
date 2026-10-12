/// 槽位场景与目标 World 快照的整体相等性及规模摘要；matches 不提供逐实体差异。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotDiffReport {
    pub slot_id: String,
    pub matches: bool,
    pub slot_entity_count: usize,
    pub target_entity_count: usize,
    pub slot_resource_count: usize,
    pub target_resource_count: usize,
}
