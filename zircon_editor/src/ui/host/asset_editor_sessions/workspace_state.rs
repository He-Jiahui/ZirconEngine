use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::ui::asset_editor::UiAssetEditorSession;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalConflict {
    pub(crate) asset_id: String,
    pub(crate) source_path: PathBuf,
    pub(crate) baseline_hash: u64,
    pub(crate) local_hash: u64,
    pub(crate) external_hash: u64,
    pub(crate) local_source: String,
    pub(crate) external_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetStaleImportDiagnostic {
    pub(crate) reference: String,
    pub(crate) message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetDiffSnapshot {
    pub asset_id: String,
    pub baseline_hash: u64,
    pub local_hash: u64,
    pub external_hash: u64,
    pub local_source: String,
    pub external_source: String,
    pub summary: String,
}

impl From<&UiAssetExternalConflict> for UiAssetDiffSnapshot {
    fn from(conflict: &UiAssetExternalConflict) -> Self {
        Self {
            asset_id: conflict.asset_id.clone(),
            baseline_hash: conflict.baseline_hash,
            local_hash: conflict.local_hash,
            external_hash: conflict.external_hash,
            local_source: conflict.local_source.clone(),
            external_source: conflict.external_source.clone(),
            summary: format!(
                "External change detected for {} (local {}, external {})",
                conflict.asset_id, conflict.local_hash, conflict.external_hash
            ),
        }
    }
}

impl UiAssetExternalConflict {
    pub(crate) fn new(
        asset_id: String,
        source_path: PathBuf,
        baseline_hash: u64,
        local_source: String,
        external_source: String,
    ) -> Self {
        Self {
            asset_id,
            source_path,
            baseline_hash,
            local_hash: ui_asset_source_hash(&local_source),
            external_hash: ui_asset_source_hash(&external_source),
            local_source,
            external_source,
        }
    }
}

pub(crate) struct UiAssetWorkspaceEntry {
    pub(crate) source_path: PathBuf,
    pub(crate) session: UiAssetEditorSession,
    pub(crate) disk_source: String,
    pub(crate) disk_source_hash: u64,
    pub(crate) conflict: Option<UiAssetExternalConflict>,
    pub(crate) stale_imports: BTreeMap<String, UiAssetStaleImportDiagnostic>,
    pub(crate) diff_snapshot: Option<UiAssetDiffSnapshot>,
}

impl UiAssetWorkspaceEntry {
    pub(crate) fn new(source_path: PathBuf, source: String, session: UiAssetEditorSession) -> Self {
        let disk_source_hash = ui_asset_source_hash(&source);
        Self {
            source_path,
            session,
            disk_source: source,
            disk_source_hash,
            conflict: None,
            stale_imports: BTreeMap::new(),
            diff_snapshot: None,
        }
    }

    pub(crate) fn update_disk_baseline(&mut self, source: String) {
        self.disk_source_hash = ui_asset_source_hash(&source);
        self.disk_source = source;
    }

    pub(crate) fn has_external_conflict(&self) -> bool {
        self.conflict.is_some()
    }

    pub(crate) fn external_conflict_summary(&self) -> String {
        self.conflict
            .as_ref()
            .map(|conflict| UiAssetDiffSnapshot::from(conflict).summary)
            .unwrap_or_default()
    }

    pub(crate) fn stale_import_items(&self) -> Vec<String> {
        self.stale_imports
            .values()
            .map(|diagnostic| format!("{}: {}", diagnostic.reference, diagnostic.message))
            .collect()
    }
}

pub(crate) fn ui_asset_source_hash(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
