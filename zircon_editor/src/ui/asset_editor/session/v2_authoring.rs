use std::collections::BTreeMap;

use crate::ui::asset_editor::preview::preview_host::UiAssetPreviewHost;
use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2PrototypeStore};
use zircon_runtime_interface::ui::{
    layout::UiSize,
    v2::{UiV2AssetDocument, UiV2AssetKind},
};

use super::ui_asset_editor_session::{UiAssetEditorSessionError, UiAssetV2CompilerImports};

pub(super) fn build_v2_preview_host(
    document: &UiV2AssetDocument,
    preview_size: UiSize,
    imports: &UiAssetV2CompilerImports,
) -> Result<Option<UiAssetPreviewHost>, UiAssetEditorSessionError> {
    if is_v2_style_import_kind(document.asset.kind) {
        return Ok(None);
    }

    let (document, store) = v2_document_and_prototype_store(document, imports);
    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&document, &store)?;
    Ok(Some(UiAssetPreviewHost::new_v2(
        preview_size,
        &document,
        &compiled,
    )?))
}

pub(super) fn ensure_v2_asset_kind(
    expected: UiV2AssetKind,
    actual: UiV2AssetKind,
) -> Result<(), UiAssetEditorSessionError> {
    let matches_style_import = expected == UiV2AssetKind::Style && is_v2_style_import_kind(actual);
    if expected != actual && !matches_style_import {
        return Err(UiAssetEditorSessionError::UnexpectedV2Kind { expected, actual });
    }
    Ok(())
}

fn is_v2_style_import_kind(kind: UiV2AssetKind) -> bool {
    matches!(kind, UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens)
}

fn extend_cloned_map<K, V>(target: &mut BTreeMap<K, V>, source: &BTreeMap<K, V>)
where
    K: Clone + Ord,
    V: Clone,
{
    target.extend(
        source
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
}

fn extend_cloned_values<T: Clone>(target: &mut Vec<T>, source: &[T]) {
    target.extend_from_slice(source);
}

fn v2_document_and_prototype_store(
    document: &UiV2AssetDocument,
    imports: &UiAssetV2CompilerImports,
) -> (UiV2AssetDocument, UiV2PrototypeStore) {
    let document = v2_preview_document_with_imported_styles(document, &imports.styles);
    let mut store = UiV2PrototypeStore::new();
    let root = store.insert(document.clone());
    store.insert_alias(document.asset.id.clone(), root);
    for (reference, import) in imports.widgets.iter().chain(imports.styles.iter()) {
        let imported = store.insert(import.clone());
        if reference != &import.asset.id {
            store.insert_alias(reference.clone(), imported);
        }
    }
    (document, store)
}

fn v2_preview_document_with_imported_styles(
    document: &UiV2AssetDocument,
    styles: &BTreeMap<String, UiV2AssetDocument>,
) -> UiV2AssetDocument {
    let mut document = document.clone();
    if document.root.is_none() && matches!(document.asset.kind, UiV2AssetKind::Component) {
        document.root = document.components.values().next().map(|component| {
            zircon_runtime_interface::ui::v2::UiV2Root {
                node: component.root.clone(),
            }
        });
    }
    for style in styles.values() {
        extend_cloned_map(&mut document.tokens, &style.tokens);
        extend_cloned_values(&mut document.stylesheets, &style.stylesheets);
    }
    document
}

#[cfg(test)]
#[path = "v2_authoring/streaming_style_tests.rs"]
mod streaming_style_tests;
