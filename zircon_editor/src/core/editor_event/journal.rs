use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    EditorEventRecord, EditorEventRetentionBudgets, EditorEventRetentionBudgetsSnapshot,
    EditorEventRetentionDiagnostics, EditorEventRetentionStore, SharedEditorEventRecord,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorEventJournal {
    records: Vec<EditorEventRecord>,
    #[serde(default)]
    retention_diagnostics: EditorEventRetentionDiagnostics,
    #[serde(default)]
    retention_budgets: EditorEventRetentionBudgetsSnapshot,
}

impl EditorEventJournal {
    pub fn records(&self) -> &[EditorEventRecord] {
        &self.records
    }

    pub fn retention_diagnostics(&self) -> &EditorEventRetentionDiagnostics {
        &self.retention_diagnostics
    }

    pub fn retention_budgets(&self) -> &EditorEventRetentionBudgetsSnapshot {
        &self.retention_budgets
    }
}

#[derive(Debug)]
pub(crate) struct EditorEventJournalStore {
    records: EditorEventRetentionStore,
}

impl EditorEventJournalStore {
    pub(crate) fn new(budgets: EditorEventRetentionBudgets) -> Self {
        Self {
            records: EditorEventRetentionStore::new(budgets),
        }
    }

    pub(crate) fn push(&mut self, record: Arc<SharedEditorEventRecord>) {
        self.records.push(record);
    }

    pub(crate) fn snapshot(&mut self) -> EditorEventJournal {
        let records = self
            .records
            .records()
            .into_iter()
            .map(|record| record.record().clone())
            .collect();
        EditorEventJournal {
            records,
            retention_diagnostics: self.records.diagnostics(),
            retention_budgets: self.records.budgets(),
        }
    }
}
