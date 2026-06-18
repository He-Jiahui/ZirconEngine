#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveStatistics {
    pub format_version: u32,
    pub slot_count: usize,
    pub total_entity_count: usize,
    pub total_resource_count: usize,
    pub max_slot_entity_count: usize,
    pub max_slot_resource_count: usize,
    pub earliest_updated_at_unix_millis: Option<u64>,
    pub latest_updated_at_unix_millis: Option<u64>,
    pub untimed_slot_count: usize,
}

impl RuntimeSessionArchiveStatistics {
    pub fn is_empty(&self) -> bool {
        self.slot_count == 0
    }

    pub fn has_untimed_slots(&self) -> bool {
        self.untimed_slot_count > 0
    }
}
