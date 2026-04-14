use serde::{Deserialize, Serialize};

use super::EditorEventRecord;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorEventJournal {
    records: Vec<EditorEventRecord>,
}

impl EditorEventJournal {
    pub fn push(&mut self, record: EditorEventRecord) {
        self.records.push(record);
    }

    pub fn records(&self) -> &[EditorEventRecord] {
        &self.records
    }
}
