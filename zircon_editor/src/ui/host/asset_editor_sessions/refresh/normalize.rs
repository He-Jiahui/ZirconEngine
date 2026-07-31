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
    changed_asset_ids
        .into_iter()
        .map(|asset_id| normalize_ui_asset_asset_id(asset_id.as_ref()).to_string())
        .collect()
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
mod performance_tests {
    #[test]
    fn changed_import_lookup_borrows_the_normalized_asset_id() {
        let source = include_str!("imports.rs");
        let owned_lookup = [
            "contains(&normalize_ui_asset_asset_id(reference)",
            ".to_string())",
        ]
        .concat();

        assert!(!source.contains(&owned_lookup));
    }
}
