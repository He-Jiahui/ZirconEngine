use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    EditorEventRecord, EditorEventRetentionBudgets, EditorEventRetentionBudgetsSnapshot,
    EditorEventRetentionDiagnostics, EditorEventRetentionStore, SharedEditorEventRecord,
};

#[cfg(test)]
#[path = "journal/shared_snapshot_cache_tests.rs"]
mod shared_snapshot_cache_tests;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorEventJournal {
    records: Arc<[EditorEventRecord]>,
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
    cached_generation: u64,
    cached_records: Arc<[EditorEventRecord]>,
}

impl EditorEventJournalStore {
    pub(crate) fn new(budgets: EditorEventRetentionBudgets) -> Self {
        Self {
            records: EditorEventRetentionStore::new(budgets),
            cached_generation: 0,
            cached_records: Arc::default(),
        }
    }

    pub(crate) fn push(&mut self, record: Arc<SharedEditorEventRecord>) {
        self.records.push(record);
    }

    pub(crate) fn snapshot(&mut self) -> EditorEventJournal {
        let generation = self.records.generation_after_prune();
        if generation != self.cached_generation {
            let shared_records = self.records.records();
            self.cached_records = shared_records
                .iter()
                .map(|record| record.record().clone())
                .collect::<Vec<_>>()
                .into();
            self.cached_generation = self.records.generation();
        }
        EditorEventJournal {
            records: Arc::clone(&self.cached_records),
            retention_diagnostics: self.records.diagnostics(),
            retention_budgets: self.records.budgets(),
        }
    }
}
