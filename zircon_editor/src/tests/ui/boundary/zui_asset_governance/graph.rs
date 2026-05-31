use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::{is_lower_snake_case_identifier, string_metadata_offender};
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

#[test]
fn production_zui_component_node_tables_are_reachable_from_the_component_root() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let (component_name, component) = document
                .components
                .iter()
                .next()
                .expect("UiZuiAssetLoader validates exactly one component");

            let mut reachable_nodes = BTreeSet::new();
            let mut stack = vec![component.root.as_str()];
            while let Some(node_id) = stack.pop() {
                if !reachable_nodes.insert(node_id.to_string()) {
                    continue;
                }
                let Some(node) = document.nodes.get(node_id) else {
                    offenders.push(format!(
                        "{} component `{}` references missing node `{}`",
                        path.display(),
                        component_name,
                        node_id
                    ));
                    continue;
                };
                for child in &node.children {
                    stack.push(child.node.as_str());
                }
            }

            let unreachable_nodes = document
                .nodes
                .keys()
                .filter(|node_id| !reachable_nodes.contains(*node_id))
                .cloned()
                .collect::<Vec<_>>();
            if !unreachable_nodes.is_empty() {
                offenders.push(format!(
                    "{} component `{}` declares unreachable nodes {unreachable_nodes:?}",
                    path.display(),
                    component_name
                ));
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component node tables must be reachable from the single component root: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_root_references_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                checked_components += 1;
                if let Some(invalid_root) =
                    string_metadata_offender(&component.root, "component root node reference")
                {
                    offenders.push(format!(
                        "{} component `{}` declares {invalid_root}",
                        path.display(),
                        component_name
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
        checked_components > 0,
        "production .zui assets should declare components"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component root node references must be non-empty and trimmed for stable prototype entry points: {offenders:#?}"
    );
}

#[test]
fn production_zui_node_ids_are_lower_snake_case() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_node_ids = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for node_id in document.nodes.keys() {
                checked_node_ids += 1;
                if !is_lower_snake_case_identifier(node_id) {
                    offenders.push(format!(
                        "{} declares non-lower_snake_case node id `{}`",
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
        checked_node_ids > 0,
        "production .zui assets should declare persistent node ids"
    );
    assert!(
        offenders.is_empty(),
        "production .zui node ids must stay lower_snake_case for stable hierarchy diffs and authoring selection state: {offenders:#?}"
    );
}

#[test]
fn production_zui_child_mount_node_references_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_child_mounts = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (parent_node_id, node) in &document.nodes {
                for (child_index, child) in node.children.iter().enumerate() {
                    checked_child_mounts += 1;
                    if let Some(invalid_child_node) =
                        string_metadata_offender(&child.node, "child mount node reference")
                    {
                        offenders.push(format!(
                            "{} node `{}` child mount #{} declares {invalid_child_node}",
                            path.display(),
                            parent_node_id,
                            child_index + 1
                        ));
                    }
                }
            }
        }
    }

    assert!(
        checked_assets > 0,
        "production asset roots should contain .zui component assets"
    );
    assert!(
        checked_child_mounts > 0,
        "production .zui assets should declare child mounts"
    );
    assert!(
        offenders.is_empty(),
        "production .zui child mount node references must be non-empty and trimmed for stable hierarchy links: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_node_graphs_are_single_root_trees() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_child_mounts = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let (component_name, component) = document
                .components
                .iter()
                .next()
                .expect("UiZuiAssetLoader validates exactly one component");
            let root_node_id = component.root.as_str();
            let mut mounted_by_node = BTreeMap::<String, Vec<String>>::new();

            for (parent_node_id, node) in &document.nodes {
                for child in &node.children {
                    checked_child_mounts += 1;
                    if child.node == root_node_id {
                        offenders.push(format!(
                            "{} component `{}` mounts root node `{}` under `{}`",
                            path.display(),
                            component_name,
                            root_node_id,
                            parent_node_id
                        ));
                    }
                    mounted_by_node
                        .entry(child.node.clone())
                        .or_default()
                        .push(parent_node_id.clone());
                }
            }

            for (child_node_id, parent_node_ids) in mounted_by_node {
                if parent_node_ids.len() > 1 {
                    offenders.push(format!(
                        "{} component `{}` mounts node `{}` under multiple parents {parent_node_ids:?}",
                        path.display(),
                        component_name,
                        child_node_id
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
        checked_child_mounts > 0,
        "production .zui assets should declare child mounts"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component node graphs must be single-root trees; root cannot be mounted as a child and each child node may have only one parent: {offenders:#?}"
    );
}
