#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveMergeReport {
    pub inserted_slot_ids: Vec<String>,
    pub replaced_slot_ids: Vec<String>,
    pub skipped_slot_ids: Vec<String>,
}

impl RuntimeSessionArchiveMergeReport {
    pub fn is_empty(&self) -> bool {
        self.inserted_slot_ids.is_empty()
            && self.replaced_slot_ids.is_empty()
            && self.skipped_slot_ids.is_empty()
    }
}
