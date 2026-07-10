use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetKind};
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;

use super::super::project_access::normalize_ui_asset_asset_id;
use super::{is_v2_backed_ui_asset_id, legacy_asset_kind_for_v2, parse_ui_asset_document_source};

#[derive(Default)]
pub(super) struct UiAssetImportDocuments {
    pub(super) widgets: BTreeMap<String, UiAssetDocument>,
    pub(super) styles: BTreeMap<String, UiAssetDocument>,
    pub(super) v2_widgets: BTreeMap<String, UiV2AssetDocument>,
    pub(super) v2_styles: BTreeMap<String, UiV2AssetDocument>,
}

impl EditorUiHost {
    pub(super) fn collect_ui_asset_import_document(
        &self,
        reference: &str,
        expected_kind: UiAssetKind,
        documents: &mut UiAssetImportDocuments,
        visited: &mut BTreeSet<String>,
    ) -> Result<(), EditorError> {
        let source_path = self.resolve_ui_asset_path(reference)?;
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let normalized_id = normalize_ui_asset_asset_id(reference);
        let (document, v2_document) = if is_v2_backed_ui_asset_id(normalized_id) {
            let v2_document = UiZuiAssetLoader::load_zui_str(&source)
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            let actual_kind = legacy_asset_kind_for_v2(v2_document.asset.kind);
            if actual_kind != expected_kind {
                return Err(EditorError::UiAsset(format!(
                    "ui import {reference} expected {expected_kind:?} but parsed {:?}",
                    v2_document.asset.kind
                )));
            }
            let document = crate::ui::asset_editor::project_v2_document_to_authoring(&v2_document)
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (document, Some(v2_document))
        } else {
            let document = parse_ui_asset_document_source(&source)
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (document, None)
        };
        if document.asset.kind != expected_kind {
            return Err(EditorError::UiAsset(format!(
                "ui import {reference} expected {:?} but parsed {:?}",
                expected_kind, document.asset.kind
            )));
        }

        match expected_kind {
            UiAssetKind::Widget => {
                documents
                    .widgets
                    .insert(reference.to_string(), document.clone());
                if let Some(v2_document) = v2_document {
                    documents
                        .v2_widgets
                        .insert(reference.to_string(), v2_document);
                }
            }
            UiAssetKind::Style => {
                documents
                    .styles
                    .insert(reference.to_string(), document.clone());
                if let Some(v2_document) = v2_document {
                    documents
                        .v2_styles
                        .insert(reference.to_string(), v2_document);
                }
            }
            UiAssetKind::Layout => {}
        }

        let visited_key = normalize_ui_asset_asset_id(reference).to_string();
        if !visited.insert(visited_key) {
            return Ok(());
        }

        for nested in &document.imports.widgets {
            self.collect_ui_asset_import_document(nested, UiAssetKind::Widget, documents, visited)?;
        }
        for nested in &document.imports.styles {
            self.collect_ui_asset_import_document(nested, UiAssetKind::Style, documents, visited)?;
        }
        Ok(())
    }

    pub(super) fn try_collect_ui_asset_import_document(
        &self,
        reference: &str,
        expected_kind: UiAssetKind,
        documents: &mut UiAssetImportDocuments,
        visited: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        self.collect_ui_asset_import_document(reference, expected_kind, documents, visited)
            .map_err(|error| error.to_string())
    }
}
