#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionSlotDiffReport {
    pub slot_id: String,
    pub matches: bool,
    pub slot_entity_count: usize,
    pub target_entity_count: usize,
    pub slot_resource_count: usize,
    pub target_resource_count: usize,
}
