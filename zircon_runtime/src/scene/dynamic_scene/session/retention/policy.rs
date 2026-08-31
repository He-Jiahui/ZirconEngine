#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveRetentionPolicy {
    pub max_slots: Option<usize>,
    pub protected_slot_ids: Vec<String>,
}

impl RuntimeSessionArchiveRetentionPolicy {
    pub fn keep_latest(max_slots: usize) -> Self {
        Self {
            max_slots: Some(max_slots),
            protected_slot_ids: Vec::new(),
        }
    }

    pub fn with_protected_slot(mut self, slot_id: impl Into<String>) -> Self {
        self.protected_slot_ids.push(slot_id.into());
        self.normalize();
        self
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        normalize_protected_slot_ids(&mut self.protected_slot_ids);
    }
}

fn normalize_protected_slot_ids(slot_ids: &mut Vec<String>) {
    for slot_id in slot_ids.iter_mut() {
        trim_slot_id_in_place(slot_id);
    }
    slot_ids.retain(|slot_id| !slot_id.is_empty());
    slot_ids.sort();
    slot_ids.dedup();
}

fn trim_slot_id_in_place(slot_id: &mut String) {
    let trimmed_end = slot_id.trim_end().len();
    slot_id.truncate(trimmed_end);

    let trimmed_start = slot_id.len() - slot_id.trim_start().len();
    if trimmed_start != 0 {
        slot_id.drain(..trimmed_start);
    }
}

#[cfg(test)]
#[path = "policy/in_place_tests.rs"]
mod in_place_tests;
