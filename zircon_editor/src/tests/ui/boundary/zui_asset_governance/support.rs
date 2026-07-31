use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

static PRODUCTION_ZUI_DOCUMENTS: OnceLock<BTreeMap<PathBuf, UiV2AssetDocument>> = OnceLock::new();

pub(super) fn editor_asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub(super) fn runtime_asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under workspace root")
        .join("zircon_runtime/assets")
}

pub(super) fn collect_zui_files(root: &Path) -> Vec<PathBuf> {
    // Existing component-governance modules intentionally use the component-only .zui view.
    collect_zui_component_files(root)
}

pub(super) fn collect_zui_document_files(root: &Path) -> Vec<PathBuf> {
    production_zui_documents()
        .keys()
        .filter(|path| path.starts_with(root))
        .cloned()
        .collect()
}

pub(super) fn collect_zui_component_files(root: &Path) -> Vec<PathBuf> {
    collect_zui_document_files(root)
        .into_iter()
        .filter(|path| load_zui_document(path).asset.kind == UiV2AssetKind::Component)
        .collect()
}

pub(super) fn collect_zui_view_style_files(root: &Path) -> Vec<PathBuf> {
    collect_zui_document_files(root)
        .into_iter()
        .filter(|path| is_ui_root_kind(load_zui_document(path).asset.kind))
        .collect()
}

pub(super) fn collect_ui_root_document_files(root: &Path) -> Vec<PathBuf> {
    let mut files = collect_zui_view_style_files(root);
    files.sort();
    files
}

pub(super) fn load_zui_document(path: &Path) -> &'static UiV2AssetDocument {
    production_zui_documents().get(path).unwrap_or_else(|| {
        panic!(
            "cached production .zui document `{}` should exist",
            path.display()
        )
    })
}

pub(super) fn is_ui_root_kind(kind: UiV2AssetKind) -> bool {
    matches!(
        kind,
        UiV2AssetKind::View | UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens
    )
}

fn collect_files_with_suffix(root: &Path, suffix: &str) -> Vec<PathBuf> {
    if !root.exists() {
        return Vec::new();
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()))
            {
                stack.push(
                    entry
                        .unwrap_or_else(|error| {
                            panic!("read entry under `{}`: {error}", path.display())
                        })
                        .path(),
                );
            }
            continue;
        }

        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|file_name| file_name.ends_with(suffix))
        {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn production_zui_documents() -> &'static BTreeMap<PathBuf, UiV2AssetDocument> {
    PRODUCTION_ZUI_DOCUMENTS.get_or_init(|| {
        [
            editor_asset_root().join("ui"),
            runtime_asset_root().join("ui"),
        ]
        .into_iter()
        .flat_map(|root| collect_files_with_suffix(&root, ".zui"))
        .map(|path| {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            (path, document)
        })
        .collect()
    })
}

pub(super) fn resource_locator_for_path(asset_root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(asset_root).unwrap_or_else(|error| {
        panic!(
            "strip `{}` from `{}`: {error}",
            asset_root.display(),
            path.display()
        )
    });
    let parts = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    format!("res://{}", parts.join("/"))
}

pub(super) fn split_widget_component_import(import: &str) -> Option<(&str, &str)> {
    let (asset_id, fragment) = import.split_once('#')?;
    let asset_id = asset_id.trim();
    let fragment = fragment.trim();
    (!asset_id.is_empty() && !fragment.is_empty()).then_some((asset_id, fragment))
}

pub(super) fn resolve_res_locator(locator: &str, asset_roots: &[PathBuf]) -> Option<PathBuf> {
    let relative = locator.strip_prefix("res://")?;
    asset_roots
        .iter()
        .map(|root| root.join(relative))
        .find(|candidate| candidate.exists())
}

pub(super) fn split_import_fragment(import: &str) -> (&str, Option<&str>) {
    import
        .split_once('#')
        .map_or((import, None), |(asset_id, fragment)| {
            (asset_id, Some(fragment))
        })
}

pub(super) fn zui_component_import_path(
    asset_id: &str,
    asset_roots: &[PathBuf],
) -> Option<PathBuf> {
    asset_id
        .to_ascii_lowercase()
        .contains(".zui")
        .then(|| resolve_res_locator(asset_id, asset_roots))
        .flatten()
}

pub(super) fn is_zui_component_import_asset_id(asset_id: &str) -> bool {
    asset_id.to_ascii_lowercase().contains(".zui")
}

pub(super) fn is_component_directory_path(asset_root: &Path, path: &Path) -> bool {
    path.strip_prefix(asset_root)
        .ok()
        .map(|relative| {
            relative.components().any(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .is_some_and(|part| part.ends_with("components"))
            })
        })
        .unwrap_or(false)
}

pub(super) fn pascal_case_file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .expect("production .zui assets should have UTF-8 file stems")
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut segment = String::new();
            segment.extend(first.to_uppercase());
            segment.push_str(chars.as_str());
            segment
        })
        .collect()
}

fn zui_import_locator_for(asset_id: &str) -> Option<String> {
    asset_id
        .to_ascii_lowercase()
        .contains(".zui")
        .then(|| asset_id.to_string())
}

fn push_zui_import_locator(
    asset_id: &str,
    locators: &mut BTreeSet<String>,
    pending: &mut Vec<String>,
) {
    let Some(locator) = zui_import_locator_for(asset_id.trim()) else {
        return;
    };
    if locators.insert(locator.clone()) {
        pending.push(locator);
    }
}

pub(super) fn production_widget_import_zui_locators(asset_roots: &[PathBuf]) -> BTreeSet<String> {
    let mut locators = BTreeSet::new();
    let mut pending = Vec::new();
    for asset_root in asset_roots {
        for path in collect_ui_root_document_files(&asset_root.join("ui")) {
            let document = load_zui_document(&path);

            for import in &document.imports.widgets {
                let Some((asset_id, _component_name)) = split_widget_component_import(import)
                else {
                    continue;
                };
                push_zui_import_locator(asset_id, &mut locators, &mut pending);
            }
        }
    }

    while let Some(locator) = pending.pop() {
        let Some(path) = resolve_res_locator(&locator, asset_roots) else {
            continue;
        };
        let document = load_zui_document(&path);

        for import in &document.imports.widgets {
            let (asset_id, _fragment) = split_import_fragment(import);
            push_zui_import_locator(asset_id, &mut locators, &mut pending);
        }
    }

    locators
}
