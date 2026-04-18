use std::collections::BTreeMap;

use zircon_editor_ui::UiDesignerSelectionModel;
use zircon_ui::{UiAssetDocument, UiAssetKind};

use super::hierarchy_projection::{parent_for_node, selection_for_node};
use super::UiAssetEditorSessionError;

#[derive(Default)]
pub(super) struct UiAssetCompilerImports {
    pub(super) widgets: BTreeMap<String, UiAssetDocument>,
    pub(super) styles: BTreeMap<String, UiAssetDocument>,
}

pub(super) fn ensure_asset_kind(
    expected: UiAssetKind,
    actual: UiAssetKind,
) -> Result<(), UiAssetEditorSessionError> {
    if expected != actual {
        return Err(UiAssetEditorSessionError::UnexpectedKind { expected, actual });
    }
    Ok(())
}

pub(super) fn default_selection(document: &UiAssetDocument) -> UiDesignerSelectionModel {
    document
        .root
        .as_ref()
        .map(|root| selection_for_node(document, &root.node))
        .unwrap_or_default()
}

pub(super) fn reconcile_selection(
    document: &UiAssetDocument,
    current: &UiDesignerSelectionModel,
) -> UiDesignerSelectionModel {
    let primary = current.primary_node_id.as_deref();
    if let Some(node_id) = primary {
        if document.nodes.contains_key(node_id) {
            let mut selection = selection_for_node(document, node_id);
            let parent = selection.parent_node_id.clone();
            for sibling in &current.sibling_node_ids {
                if sibling == node_id || !document.nodes.contains_key(sibling) {
                    continue;
                }
                if parent_for_node(document, sibling)
                    .map(|(parent_id, _)| Some(parent_id) == parent)
                    .unwrap_or(false)
                {
                    selection = selection.with_sibling(sibling.clone());
                }
            }
            return selection;
        }
    }
    default_selection(document)
}
