use std::collections::BTreeMap;

use crate::ui::template::{UiCompiledDocument, UiInvalidationGraph};
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetKind, UiCompileCacheKey, UiInvalidationReport,
    UiInvalidationSnapshot,
};

#[derive(Clone, Debug, Default)]
pub struct UiAssetCompileCache {
    entries: BTreeMap<UiCompileCacheKey, UiCompiledDocument>,
    last_snapshots: BTreeMap<String, UiInvalidationSnapshot>,
}

impl UiAssetCompileCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_snapshots.clear();
    }

    pub fn get(&mut self, key: &UiCompileCacheKey) -> Option<UiCompiledDocument> {
        let compiled = self.entries.get(key).cloned()?;
        self.last_snapshots.insert(
            snapshot_slot_for_header(&compiled.asset),
            key.invalidation_snapshot(),
        );
        Some(compiled)
    }

    pub fn store(&mut self, key: UiCompileCacheKey, compiled: UiCompiledDocument) {
        self.last_snapshots.insert(
            snapshot_slot_for_header(&compiled.asset),
            key.invalidation_snapshot(),
        );
        let _ = self.entries.insert(key, compiled);
    }

    pub fn report_for_miss(
        &self,
        key: &UiCompileCacheKey,
        document: &UiAssetDocument,
    ) -> UiInvalidationReport {
        let next = key.invalidation_snapshot();
        UiInvalidationGraph::classify(
            self.last_snapshots
                .get(&snapshot_slot_for_header(&document.asset)),
            &next,
            document,
        )
    }
}

fn snapshot_slot_for_header(asset: &UiAssetHeader) -> String {
    let kind = match asset.kind {
        UiAssetKind::Layout => "layout",
        UiAssetKind::Widget => "widget",
        UiAssetKind::Style => "style",
    };
    format!("{kind}:{}", asset.id)
}
