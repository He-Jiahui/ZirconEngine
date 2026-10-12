#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchivePruneReport {
    pub retained_slot_ids: Vec<String>,
    pub removed_slot_ids: Vec<String>,
}

impl RuntimeSessionArchivePruneReport {
    pub fn removed_count(&self) -> usize {
        self.removed_slot_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.removed_slot_ids.is_empty()
    }
}
