use std::collections::BTreeSet;

use crate::ui::asset_editor::{UiAssetEditorRoute, UiAssetEditorSession};
use crate::ui::host::editor_error::EditorError;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;

use super::super::{build_ui_asset_editor_session_from_source, preview_size_for_preset};

pub(in crate::ui::host::asset_editor_sessions) fn normalize_ui_asset_change_set<I, S>(
    changed_asset_ids: I,
) -> BTreeSet<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut normalized = BTreeSet::new();
    for asset_id in changed_asset_ids {
        insert_normalized_ui_asset_id(&mut normalized, asset_id.as_ref());
    }
    normalized
}

pub(super) fn insert_normalized_ui_asset_id(
    normalized: &mut BTreeSet<String>,
    asset_id: &str,
) -> bool {
    let asset_id = normalize_ui_asset_asset_id(asset_id);
    if normalized.contains(asset_id) {
        return false;
    }
    normalized.insert(asset_id.to_owned())
}

pub(super) fn rebuild_ui_asset_session_from_source(
    route: UiAssetEditorRoute,
    source: String,
) -> Result<UiAssetEditorSession, EditorError> {
    let preview_size = preview_size_for_preset(route.preview_preset);
    build_ui_asset_editor_session_from_source(route, source, preview_size)
        .map_err(|error| EditorError::UiAsset(error.to_string()))
}

#[cfg(test)]
#[path = "tests/normalize_performance_tests.rs"]
mod performance_tests;
