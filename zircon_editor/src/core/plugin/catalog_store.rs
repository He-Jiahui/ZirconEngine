//! Generation owner for immutable editor-plugin catalog snapshots.

use std::sync::{Arc, RwLock};

use super::catalog::EditorPluginCatalog;
use super::catalog_snapshot::EditorPluginCatalogSnapshot;

#[derive(Debug)]
// 目录代次的只读快照槽；管理器在序列化生命周期变更时构造下一代，读者借 Arc 保留旧代。
pub(crate) struct EditorPluginCatalogStore {
    snapshot: RwLock<Arc<EditorPluginCatalogSnapshot>>,
}

impl EditorPluginCatalogStore {
    pub(super) fn new(catalog: EditorPluginCatalog) -> Self {
        Self {
            snapshot: RwLock::new(Arc::new(EditorPluginCatalogSnapshot::from_catalog(
                1, catalog,
            ))),
        }
    }

    pub(super) fn snapshot(&self) -> Arc<EditorPluginCatalogSnapshot> {
        Arc::clone(
            &self
                .snapshot
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    /// Reserves the next catalog generation for an already materialized manager candidate.
    pub(super) fn next_generation(&self) -> u64 {
        self.snapshot().generation().saturating_add(1)
    }

    /// Publishes a candidate prepared by the manager's serialized lifecycle transaction.
    pub(super) fn publish_prepared(
        &self,
        snapshot: Arc<EditorPluginCatalogSnapshot>,
    ) -> Arc<EditorPluginCatalogSnapshot> {
        let mut snapshot_slot = self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(
            snapshot.generation(),
            snapshot_slot.generation().saturating_add(1),
            "manager must publish exactly the next catalog generation"
        );
        *snapshot_slot = Arc::clone(&snapshot);
        snapshot
    }
}

#[cfg(test)]
#[path = "tests/catalog_store.rs"]
mod tests;
