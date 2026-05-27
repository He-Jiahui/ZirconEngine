use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::ui::v2::UiV2AssetLoader;

pub(super) const BUILTIN_ZUI_ASSET_ID_ALIASES: &[(&str, &str)] = &[(
    "res://ui/editor/host/activity_drawer_window.zui",
    "editor.host.activity_drawer_window",
)];

pub(super) fn editor_asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub(super) fn runtime_asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under workspace root")
        .join("zircon_runtime/assets")
}

pub(super) fn collect_v2_ui_toml_files(root: &Path) -> Vec<PathBuf> {
    collect_files_with_suffix(root, ".v2.ui.toml")
}

pub(super) fn collect_zui_files(root: &Path) -> Vec<PathBuf> {
    collect_files_with_suffix(root, ".zui")
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

pub(super) fn builtin_zui_asset_id_alias_for(locator: &str) -> Option<&'static str> {
    BUILTIN_ZUI_ASSET_ID_ALIASES
        .iter()
        .find_map(|(alias_locator, asset_id)| (*alias_locator == locator).then_some(*asset_id))
}

pub(super) fn builtin_zui_asset_id_alias_locator_for(asset_id: &str) -> Option<&'static str> {
    BUILTIN_ZUI_ASSET_ID_ALIASES
        .iter()
        .find_map(|(locator, alias_asset_id)| (*alias_asset_id == asset_id).then_some(*locator))
}

pub(super) fn zui_component_import_path(
    asset_id: &str,
    asset_roots: &[PathBuf],
) -> Option<PathBuf> {
    if let Some(locator) = builtin_zui_asset_id_alias_locator_for(asset_id) {
        return resolve_res_locator(locator, asset_roots);
    }

    asset_id
        .to_ascii_lowercase()
        .contains(".zui")
        .then(|| resolve_res_locator(asset_id, asset_roots))
        .flatten()
}

pub(super) fn is_zui_component_import_asset_id(asset_id: &str) -> bool {
    asset_id.to_ascii_lowercase().contains(".zui")
        || builtin_zui_asset_id_alias_locator_for(asset_id).is_some()
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

pub(super) fn production_widget_import_asset_ids(asset_roots: &[PathBuf]) -> BTreeSet<String> {
    let mut asset_ids = BTreeSet::new();
    for asset_root in asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for import in &document.imports.widgets {
                let asset_id = import
                    .split_once('#')
                    .map_or(import.as_str(), |(asset_id, _)| asset_id)
                    .trim();
                if !asset_id.is_empty() {
                    asset_ids.insert(asset_id.to_string());
                }
            }
        }
    }
    asset_ids
}

pub(super) fn production_widget_import_zui_locators(asset_roots: &[PathBuf]) -> BTreeSet<String> {
    let mut locators = BTreeSet::new();
    for asset_root in asset_roots {
        for path in collect_v2_ui_toml_files(&asset_root.join("ui")) {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiV2AssetLoader::load_toml_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for import in &document.imports.widgets {
                let Some((asset_id, _component_name)) = split_widget_component_import(import)
                else {
                    continue;
                };
                if let Some(locator) = builtin_zui_asset_id_alias_locator_for(asset_id) {
                    locators.insert(locator.to_string());
                    continue;
                }
                if asset_id.to_ascii_lowercase().contains(".zui") {
                    locators.insert(asset_id.to_string());
                }
            }
        }
    }
    locators
}
