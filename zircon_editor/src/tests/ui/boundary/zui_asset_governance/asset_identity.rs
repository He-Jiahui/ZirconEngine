use std::collections::BTreeMap;

use zircon_runtime_interface::ui::v2::{UI_V2_ASSET_SCHEMA_VERSION, UiV2AssetKind};

use super::push_asset_header_metadata_offenders;
use super::support::{
    collect_zui_document_files, editor_asset_root, load_zui_document, resource_locator_for_path,
    runtime_asset_root,
};

#[test]
fn production_zui_asset_ids_match_res_locator_exactly() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_asset_ids = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_document_files(&asset_root.join("ui")) {
            checked_asset_ids += 1;
            let expected_locator = resource_locator_for_path(asset_root, &path);
            let document = load_zui_document(&path);
            let actual_asset_id = document.asset.id.as_str();

            if actual_asset_id != expected_locator {
                offenders.push(format!(
                    "{} declares asset.id `{}` but expected exact locator `{}`",
                    path.display(),
                    actual_asset_id,
                    expected_locator
                ));
            }
        }
    }

    assert!(
        checked_asset_ids > 0,
        "production asset roots should contain .zui documents"
    );
    assert!(
        offenders.is_empty(),
        ".zui document asset ids must match their res:// locator exactly: {offenders:#?}"
    );
}

#[test]
fn production_ui_asset_ids_are_unique_across_zui_documents() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut asset_ids = BTreeMap::<String, Vec<PathBuf>>::new();

    for asset_root in &asset_roots {
        for path in collect_zui_document_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let document = load_zui_document(&path);

            asset_ids
                .entry(document.asset.id.clone())
                .or_default()
                .push(path);
        }
    }

    let offenders = asset_ids
        .into_iter()
        .filter(|(_asset_id, paths)| paths.len() > 1)
        .map(|(asset_id, paths)| {
            let paths = paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>();
            format!("{asset_id} -> {paths:#?}")
        })
        .collect::<Vec<_>>();

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui assets"
    );
    assert!(
        offenders.is_empty(),
        "production UI asset ids must be globally unique across .zui documents: {offenders:#?}"
    );
}

#[test]
fn production_ui_asset_headers_are_authorable_and_current() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_document_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let document = load_zui_document(&path);

            if document.asset.version != UI_V2_ASSET_SCHEMA_VERSION {
                offenders.push(format!(
                    "{} declares schema version {}, expected {}",
                    path.display(),
                    document.asset.version,
                    UI_V2_ASSET_SCHEMA_VERSION
                ));
            }
            push_asset_header_metadata_offenders(
                &path,
                &document.asset.id,
                &document.asset.display_name,
                &mut offenders,
            );
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain UI v2 or .zui assets"
    );
    assert!(
        offenders.is_empty(),
        "production UI asset headers must use the current schema version and non-empty, trimmed author-facing asset id/display_name fields: {offenders:#?}"
    );
}

#[test]
fn builtin_template_registry_does_not_register_zui_component_assets() {
    let offenders = crate::ui::template_runtime::builtin::builtin_template_documents()
        .into_iter()
        .filter_map(|(document_id, path)| {
            let is_zui = path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|file_name| file_name.ends_with(".zui"));
            if !is_zui {
                return None;
            }
            let document = load_zui_document(&path);
            (document.asset.kind == UiV2AssetKind::Component)
                .then(|| format!("{document_id} -> {}", path.display()))
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "builtin template documents may register .zui view/style roots after suffix migration, but must not directly register .zui component prototypes: {offenders:#?}"
    );
}

#[test]
fn builtin_template_registry_keys_match_zui_asset_ids() {
    let offenders = crate::ui::template_runtime::builtin::builtin_template_documents()
        .into_iter()
        .filter_map(|(document_id, path)| {
            let document = load_zui_document(&path);
            (document_id != document.asset.id).then(|| {
                format!(
                    "registry key `{document_id}` does not match `{}` from {}",
                    document.asset.id,
                    path.display()
                )
            })
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "builtin ZUI documents must use their asset locator as the only runtime registry identity: {offenders:#?}"
    );
}
