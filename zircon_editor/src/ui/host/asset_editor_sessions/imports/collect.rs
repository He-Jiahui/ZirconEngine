use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::ui::template::UiAssetKind;

use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;

use super::parsed_document::parse_ui_asset_import_source;
use super::UiAssetImportTraversal;

impl EditorUiHost {
    pub(super) fn collect_ui_asset_import_document(
        &self,
        reference: &str,
        expected_kind: UiAssetKind,
        traversal: &mut UiAssetImportTraversal,
    ) -> Result<(), EditorError> {
        collect_ui_asset_import_document(
            &|asset_id| self.resolve_ui_asset_path(asset_id),
            reference,
            expected_kind,
            traversal,
        )
    }

    pub(in crate::ui::host::asset_editor_sessions) fn try_collect_ui_asset_import_document(
        &self,
        reference: &str,
        expected_kind: UiAssetKind,
        traversal: &mut UiAssetImportTraversal,
    ) -> Result<(), String> {
        self.collect_ui_asset_import_document(reference, expected_kind, traversal)
            .map_err(|error| error.to_string())
    }
}

pub(in crate::ui::host::asset_editor_sessions) fn collect_ui_asset_import_document(
    resolve: &impl Fn(&str) -> Result<PathBuf, EditorError>,
    reference: &str,
    expected_kind: UiAssetKind,
    traversal: &mut UiAssetImportTraversal,
) -> Result<(), EditorError> {
    // The logical edge must survive resolution/read/parse failure so a later
    // watcher event for the repaired dependency can target this consumer.
    traversal.record_dependency(reference);
    let source_path = resolve(reference)?;
    let physical_path = canonical_ui_asset_import_path(&source_path);
    let physical_asset_id = physical_path.to_string_lossy();
    let parsed = traversal
        .generation_mut()
        .load_physical_document(&physical_path, || {
            let source = fs::read_to_string(&physical_path).map_err(|error| error.to_string())?;
            parse_ui_asset_import_source(&physical_asset_id, &source)
        })?;

    if !traversal.materialize_reference(reference, expected_kind, &physical_path, &parsed)? {
        return Ok(());
    }

    for nested in &parsed.document.imports.widgets {
        collect_ui_asset_import_document(resolve, nested, UiAssetKind::Widget, traversal)?;
    }
    for nested in &parsed.document.imports.styles {
        collect_ui_asset_import_document(resolve, nested, UiAssetKind::Style, traversal)?;
    }
    Ok(())
}

fn canonical_ui_asset_import_path(source_path: &Path) -> PathBuf {
    fs::canonicalize(source_path).unwrap_or_else(|_| source_path.to_path_buf())
}
