use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    EditorEventRecord, EditorEventRetentionBudgets, EditorEventRetentionBudgetsSnapshot,
    EditorEventRetentionDiagnostics, EditorEventRetentionStore, SharedEditorEventRecord,
};

#[cfg(test)]
#[path = "journal/tests/shared_snapshot_cache_tests.rs"]
mod shared_snapshot_cache_tests;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
/// 事件服务当前保留记录的不可变快照，供诊断、序列化与回放读取。
/// 保留策略可能淘汰或合并历史；是否存在缺口应结合诊断与预算判断，不能据此假定持有完整操作历史。
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
// 由事件服务日志锁独占维护；快照共享缓存后的记录数组，只有保留内容的代次变化才重新复制记录。
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

    // 读取也会触发保留期裁剪；缓存比较使用裁剪后的代次，使快照内容与此次诊断来自同一保留状态。
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
