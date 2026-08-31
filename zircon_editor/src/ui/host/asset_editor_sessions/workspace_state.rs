use std::collections::BTreeMap;
use std::path::PathBuf;

use blake3::Hash;

use crate::ui::asset_editor::UiAssetEditorSession;

pub(crate) type UiAssetSourceDigest = Hash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetExternalConflict {
    pub(crate) asset_id: String,
    pub(crate) source_path: PathBuf,
    pub(crate) baseline_digest: UiAssetSourceDigest,
    pub(crate) local_digest: UiAssetSourceDigest,
    pub(crate) external_digest: UiAssetSourceDigest,
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
    pub baseline_digest: UiAssetSourceDigest,
    pub local_digest: UiAssetSourceDigest,
    pub external_digest: UiAssetSourceDigest,
    pub local_source: String,
    pub external_source: String,
    pub summary: String,
}

impl From<&UiAssetExternalConflict> for UiAssetDiffSnapshot {
    fn from(conflict: &UiAssetExternalConflict) -> Self {
        Self {
            asset_id: conflict.asset_id.clone(),
            baseline_digest: conflict.baseline_digest,
            local_digest: conflict.local_digest,
            external_digest: conflict.external_digest,
            local_source: conflict.local_source.clone(),
            external_source: conflict.external_source.clone(),
            summary: format!(
                "External change detected for {} (local {}, external {})",
                conflict.asset_id,
                conflict.local_digest.to_hex(),
                conflict.external_digest.to_hex()
            ),
        }
    }
}

impl UiAssetExternalConflict {
    pub(crate) fn new(
        asset_id: String,
        source_path: PathBuf,
        baseline_digest: UiAssetSourceDigest,
        local_source: String,
        external_source: String,
    ) -> Self {
        Self {
            asset_id,
            source_path,
            baseline_digest,
            local_digest: ui_asset_source_digest(&local_source),
            external_digest: ui_asset_source_digest(&external_source),
            local_source,
            external_source,
        }
    }
}

pub(crate) struct UiAssetWorkspaceEntry {
    pub(crate) source_path: PathBuf,
    pub(crate) session: UiAssetEditorSession,
    pub(crate) disk_source: String,
    pub(crate) disk_source_digest: UiAssetSourceDigest,
    pub(crate) conflict: Option<UiAssetExternalConflict>,
    pub(crate) stale_imports: BTreeMap<String, UiAssetStaleImportDiagnostic>,
    pub(crate) diff_snapshot: Option<UiAssetDiffSnapshot>,
}

impl UiAssetWorkspaceEntry {
    pub(crate) fn new(source_path: PathBuf, source: String, session: UiAssetEditorSession) -> Self {
        let disk_source_digest = ui_asset_source_digest(&source);
        Self {
            source_path,
            session,
            disk_source: source,
            disk_source_digest,
            conflict: None,
            stale_imports: BTreeMap::new(),
            diff_snapshot: None,
        }
    }

    pub(crate) fn update_disk_baseline(&mut self, source: String) {
        self.disk_source_digest = ui_asset_source_digest(&source);
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

pub(crate) fn ui_asset_source_digest(source: &str) -> UiAssetSourceDigest {
    blake3::hash(source.as_bytes())
}
