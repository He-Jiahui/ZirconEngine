use std::collections::BTreeSet;
use std::path::Path;

use zircon_runtime_interface::ui::template::UiAssetKind;

use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::host::EditorError;

use super::parsed_document::ParsedUiAssetImportDocument;
use super::{UiAssetImportDocuments, UiAssetImportGeneration, UiAssetImportResolution};

#[derive(Default)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetImportTraversal {
    documents: UiAssetImportDocuments,
    dependencies: BTreeSet<String>,
    expanded_physical_paths: BTreeSet<std::path::PathBuf>,
    generation: UiAssetImportGeneration,
}

impl UiAssetImportTraversal {
    pub(in crate::ui::host::asset_editor_sessions) fn into_documents(
        self,
    ) -> UiAssetImportDocuments {
        self.documents
    }

    pub(in crate::ui::host::asset_editor_sessions) fn finish_resolution(
        &mut self,
    ) -> UiAssetImportResolution {
        let resolution = UiAssetImportResolution {
            documents: std::mem::take(&mut self.documents),
            dependencies: std::mem::take(&mut self.dependencies),
        };
        self.expanded_physical_paths.clear();
        resolution
    }

    pub(super) fn generation_mut(&mut self) -> &mut UiAssetImportGeneration {
        &mut self.generation
    }

    pub(super) fn record_dependency(&mut self, reference: &str) {
        self.dependencies
            .insert(normalize_ui_asset_asset_id(reference).to_string());
    }

    pub(super) fn materialize_reference(
        &mut self,
        reference: &str,
        expected_kind: UiAssetKind,
        physical_path: &Path,
        parsed: &ParsedUiAssetImportDocument,
    ) -> Result<bool, EditorError> {
        if let Some(v2_document) = &parsed.v2_document {
            let actual_kind = super::super::legacy_asset_kind_for_v2(v2_document.asset.kind);
            if actual_kind != expected_kind {
                return Err(EditorError::UiAsset(format!(
                    "ui import {reference} expected {expected_kind:?} but parsed {:?}",
                    v2_document.asset.kind
                )));
            }
        }
        if parsed.document.asset.kind != expected_kind {
            return Err(EditorError::UiAsset(format!(
                "ui import {reference} expected {:?} but parsed {:?}",
                expected_kind, parsed.document.asset.kind
            )));
        }

        match expected_kind {
            UiAssetKind::Widget => {
                self.documents
                    .widgets
                    .insert(reference.to_string(), parsed.document.clone());
                if let Some(v2_document) = &parsed.v2_document {
                    self.documents
                        .v2_widgets
                        .insert(reference.to_string(), v2_document.clone());
                }
            }
            UiAssetKind::Style => {
                self.documents
                    .styles
                    .insert(reference.to_string(), parsed.document.clone());
                if let Some(v2_document) = &parsed.v2_document {
                    self.documents
                        .v2_styles
                        .insert(reference.to_string(), v2_document.clone());
                }
            }
            UiAssetKind::Layout => {}
        }

        Ok(self
            .expanded_physical_paths
            .insert(physical_path.to_path_buf()))
    }
}
