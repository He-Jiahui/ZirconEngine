use std::collections::BTreeMap;

use super::super::ui_asset_editor_session::{
    UiAssetEditorSession, UiAssetEditorSessionError, UiAssetSourceSchema,
};
use super::{
    reconcile_selected_palette_index, v2_projection::v2_document_to_legacy_projection_document,
};
use crate::ui::asset_editor::palette::{UiAssetPaletteCatalog, UiAssetPaletteEntry};
use zircon_runtime_interface::ui::{template::UiAssetDocument, v2::UiV2AssetDocument};

#[cfg(test)]
#[path = "palette_catalog/selection_fast_path_tests.rs"]
mod selection_fast_path_tests;

pub(super) fn build_layout(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> UiAssetPaletteCatalog {
    UiAssetPaletteCatalog::build(document, widget_imports)
}

pub(super) fn build_v2(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiV2AssetDocument>,
) -> Result<UiAssetPaletteCatalog, UiAssetEditorSessionError> {
    let reference_imports = v2_palette_reference_imports(widget_imports)?;
    Ok(UiAssetPaletteCatalog::build(document, &reference_imports))
}

pub(super) fn refresh_palette_catalog(
    session: &mut UiAssetEditorSession,
) -> Result<(), UiAssetEditorSessionError> {
    session.palette_catalog = match session.source_schema {
        UiAssetSourceSchema::LayoutDocument => build_layout(
            &session.last_valid_document,
            &session.compiler_imports.widgets,
        ),
        UiAssetSourceSchema::V2 => build_v2(
            &session.last_valid_document,
            &session.v2_compiler_imports.widgets,
        )?,
    };
    #[cfg(test)]
    {
        session.palette_catalog_build_count += 1;
    }
    Ok(())
}

fn v2_palette_reference_imports(
    widget_imports: &BTreeMap<String, UiV2AssetDocument>,
) -> Result<BTreeMap<String, UiAssetDocument>, UiAssetEditorSessionError> {
    let mut references = BTreeMap::new();
    for (asset_reference, imported_document) in widget_imports {
        let document = v2_document_to_legacy_projection_document(imported_document)?;
        if let Some((_, component_name)) = asset_reference.rsplit_once('#') {
            if document.components.contains_key(component_name) {
                let _ = references.insert(asset_reference.clone(), document);
            }
            continue;
        }
        for component_name in document.components.keys() {
            let _ = references.insert(
                format!("{asset_reference}#{component_name}"),
                document.clone(),
            );
        }
    }
    Ok(references)
}

fn selected_palette_entry_index(
    entries: &[UiAssetPaletteEntry],
    selected_entry: &UiAssetPaletteEntry,
    previous_index: Option<usize>,
) -> Option<usize> {
    if let Some(previous_index) = previous_index {
        if entries.get(previous_index) == Some(selected_entry) {
            return Some(previous_index);
        }
    }
    entries.iter().position(|entry| entry == selected_entry)
}

pub(super) fn reconcile_palette_catalog_selection(session: &mut UiAssetEditorSession) {
    let selected_palette_index = session
        .selected_palette_entry
        .as_ref()
        .and_then(|selected_entry| {
            selected_palette_entry_index(
                session.palette_catalog.entries(),
                selected_entry,
                session.selected_palette_index,
            )
        })
        .or_else(|| {
            reconcile_selected_palette_index(
                session.palette_catalog.entries(),
                session.selected_palette_index,
            )
        });
    session.selected_palette_index = selected_palette_index;
    session.selected_palette_entry = session
        .selected_palette_index
        .and_then(|index| session.palette_catalog.entry(index).cloned());
    session.clear_palette_drag_state();
}
