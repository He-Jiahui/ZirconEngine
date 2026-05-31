use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::{attribute_key_offenders, resource_path_string_offenders};
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

#[test]
fn production_zui_node_attribute_keys_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_attribute_maps = 0usize;
    let mut checked_attribute_keys = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                for (label, attributes) in [
                    ("props", &node.props),
                    ("state", &node.state),
                    ("style.self", &node.style.self_values),
                    ("style.slot", &node.style.slot),
                ] {
                    if attributes.is_empty() {
                        continue;
                    }
                    checked_attribute_maps += 1;
                    let (checked_keys, invalid_keys) =
                        attribute_key_offenders(attributes, "node attribute key");
                    checked_attribute_keys += checked_keys;
                    if !invalid_keys.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` {label} contain {invalid_keys:?}",
                            path.display(),
                            node_id
                        ));
                    }
                }

                let Some(layout) = &node.layout else {
                    continue;
                };
                if layout.is_empty() {
                    continue;
                }
                checked_attribute_maps += 1;
                let (checked_keys, invalid_keys) =
                    attribute_key_offenders(layout, "node attribute key");
                checked_attribute_keys += checked_keys;
                if !invalid_keys.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` layout contains {invalid_keys:?}",
                        path.display(),
                        node_id
                    ));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_attribute_maps > 0 && checked_attribute_keys > 0,
        "production .zui assets should declare node attribute maps"
    );
    assert!(
        offenders.is_empty(),
        "production .zui node attribute keys must be non-empty and trimmed for stable inspector, style, and layout state: {offenders:#?}"
    );
}

#[test]
fn production_zui_resource_path_strings_are_portable() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_attribute_maps = 0usize;
    let mut checked_resource_strings = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                for (label, attributes) in [
                    ("props", &node.props),
                    ("state", &node.state),
                    ("style.self", &node.style.self_values),
                    ("style.slot", &node.style.slot),
                ] {
                    if attributes.is_empty() {
                        continue;
                    }
                    checked_attribute_maps += 1;
                    let (checked_strings, invalid_paths) =
                        resource_path_string_offenders(attributes, label);
                    checked_resource_strings += checked_strings;
                    if !invalid_paths.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` {invalid_paths:?}",
                            path.display(),
                            node_id
                        ));
                    }
                }

                let Some(layout) = &node.layout else {
                    continue;
                };
                if layout.is_empty() {
                    continue;
                }
                checked_attribute_maps += 1;
                let (checked_strings, invalid_paths) =
                    resource_path_string_offenders(layout, "layout");
                checked_resource_strings += checked_strings;
                if !invalid_paths.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` {invalid_paths:?}",
                        path.display(),
                        node_id
                    ));
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_attribute_maps > 0 && checked_resource_strings > 0,
        "production .zui assets should declare resource-like node attribute strings"
    );
    assert!(
        offenders.is_empty(),
        "production .zui resource-like strings must use portable asset paths, not dev-tree paths, parent-relative paths, absolute drive paths, or backslashes: {offenders:#?}"
    );
}
