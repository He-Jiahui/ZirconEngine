use std::collections::BTreeSet;

use crate::ui::asset_editor::UiAssetEditorSession;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::super::imports::UiAssetImportResolution;
use super::super::super::UiAssetStaleImportDiagnostic;
use super::plan::{UiAssetDirectRefreshPlan, UiAssetImportRefreshPlan};

pub(in crate::ui::host::asset_editor_sessions) struct UiAssetRefreshBatch {
    pub(super) generation: u64,
    pub(super) dependency_generation: u64,
    pub(super) changed_asset_ids: BTreeSet<String>,
    pub(super) project_root: Option<std::path::PathBuf>,
    pub(super) direct_results: Vec<UiAssetDirectRefreshResult>,
    pub(super) import_results: Vec<UiAssetImportRefreshResult>,
}

pub(super) struct UiAssetDirectRefreshResult {
    pub(super) plan: UiAssetDirectRefreshPlan,
    pub(super) outcome: UiAssetDirectRefreshOutcome,
}

pub(super) enum UiAssetDirectRefreshOutcome {
    Unchanged,
    Missing,
    Conflict {
        external_source: String,
    },
    Invalid {
        external_source: String,
        message: String,
    },
    Failed {
        message: String,
    },
    Reloaded {
        external_source: String,
        session: UiAssetEditorSession,
        imports: UiAssetImportResolution,
        import_errors: Vec<UiAssetStaleImportDiagnostic>,
    },
}

pub(super) struct UiAssetImportRefreshResult {
    pub(super) plan: UiAssetImportRefreshPlan,
    pub(super) imports: UiAssetImportResolution,
    pub(super) errors: Vec<UiAssetStaleImportDiagnostic>,
}

pub(in crate::ui::host::asset_editor_sessions) struct UiAssetRefreshCommitReport {
    pub(in crate::ui::host::asset_editor_sessions) sync_instances: BTreeSet<ViewInstanceId>,
    pub(in crate::ui::host::asset_editor_sessions) requeue_asset_ids: BTreeSet<String>,
    pub(in crate::ui::host::asset_editor_sessions) retry_asset_ids: BTreeSet<String>,
}
