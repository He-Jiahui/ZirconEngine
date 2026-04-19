use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use crate::{EditorError, EditorManager};
use zircon_ui::{UiAssetDocument, UiAssetKind, template::UiAssetLoader};

use super::super::project_access::normalize_ui_asset_asset_id;

impl EditorManager {
    pub(super) fn collect_ui_asset_import_document(
        &self,
        reference: &str,
        expected_kind: UiAssetKind,
        widget_docs: &mut BTreeMap<String, UiAssetDocument>,
        style_docs: &mut BTreeMap<String, UiAssetDocument>,
        visited: &mut BTreeSet<String>,
    ) -> Result<(), EditorError> {
        let source_path = self.resolve_ui_asset_path(reference)?;
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let document = UiAssetLoader::load_toml_str(&source)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        if document.asset.kind != expected_kind {
            return Err(EditorError::UiAsset(format!(
                "ui import {reference} expected {:?} but parsed {:?}",
                expected_kind, document.asset.kind
            )));
        }

        match expected_kind {
            UiAssetKind::Widget => {
                widget_docs.insert(reference.to_string(), document.clone());
            }
            UiAssetKind::Style => {
                style_docs.insert(reference.to_string(), document.clone());
            }
            UiAssetKind::Layout => {}
        }

        let visited_key = normalize_ui_asset_asset_id(reference).to_string();
        if !visited.insert(visited_key) {
            return Ok(());
        }

        for nested in &document.imports.widgets {
            self.collect_ui_asset_import_document(
                nested,
                UiAssetKind::Widget,
                widget_docs,
                style_docs,
                visited,
            )?;
        }
        for nested in &document.imports.styles {
            self.collect_ui_asset_import_document(
                nested,
                UiAssetKind::Style,
                widget_docs,
                style_docs,
                visited,
            )?;
        }
        Ok(())
    }
}

