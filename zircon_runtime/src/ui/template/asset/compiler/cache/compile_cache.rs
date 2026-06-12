use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiAssetCompileCacheEvictionReport {
    pub entries_removed: usize,
    pub snapshots_removed: usize,
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

    pub fn evict_asset(&mut self, asset_id: &str) -> UiAssetCompileCacheEvictionReport {
        self.evict_assets([asset_id])
    }

    pub fn evict_assets<'a>(
        &mut self,
        asset_ids: impl IntoIterator<Item = &'a str>,
    ) -> UiAssetCompileCacheEvictionReport {
        let asset_ids = asset_ids
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        if asset_ids.is_empty() {
            return UiAssetCompileCacheEvictionReport::default();
        }

        // Entries are keyed by compile options, while snapshots are keyed by the
        // asset id slot used for invalidation reports. Eviction must clear both.
        let entry_keys = self
            .entries
            .iter()
            .filter_map(|(key, compiled)| {
                asset_ids
                    .contains(&compiled.asset.id)
                    .then_some(key.clone())
            })
            .collect::<Vec<_>>();
        let snapshot_keys = self
            .last_snapshots
            .keys()
            .filter(|slot| asset_ids.contains(snapshot_slot_asset_id(slot)))
            .cloned()
            .collect::<Vec<_>>();

        let entries_removed = entry_keys.len();
        for key in entry_keys {
            self.entries.remove(&key);
        }
        let snapshots_removed = snapshot_keys.len();
        for key in snapshot_keys {
            self.last_snapshots.remove(&key);
        }

        UiAssetCompileCacheEvictionReport {
            entries_removed,
            snapshots_removed,
        }
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

fn snapshot_slot_asset_id(slot: &str) -> &str {
    slot.split_once(':')
        .map(|(_, asset_id)| asset_id)
        .unwrap_or(slot)
}
