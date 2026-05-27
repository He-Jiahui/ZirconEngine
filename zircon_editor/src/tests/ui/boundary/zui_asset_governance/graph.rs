use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::template::UiStyleScope;

use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

fn is_lower_snake_case_identifier(value: &str) -> bool {
    let mut previous_was_underscore = false;
    for (index, character) in value.chars().enumerate() {
        let valid = character.is_ascii_lowercase()
            || (index > 0 && character.is_ascii_digit())
            || (index > 0 && character == '_' && !previous_was_underscore);
        if !valid {
            return false;
        }
        previous_was_underscore = character == '_';
    }
    !value.is_empty() && !previous_was_underscore
}

fn class_list_offenders(classes: &[String]) -> Vec<String> {
    let mut counts = BTreeMap::<&str, usize>::new();
    let mut offenders = Vec::new();

    for class in classes {
        let trimmed = class.trim();
        if trimmed.is_empty() {
            offenders.push("empty class token".to_string());
            continue;
        }
        if trimmed != class {
            offenders.push(format!("whitespace-padded class token `{class}`"));
        }
        *counts.entry(trimmed).or_default() += 1;
    }

    offenders.extend(
        counts
            .into_iter()
            .filter_map(|(class, count)| (count > 1).then(|| format!("duplicate `{class}`"))),
    );
    offenders
}

fn string_metadata_offender(value: &str, label: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Some(format!("empty {label}"));
    }
    (trimmed != value).then(|| format!("whitespace-padded {label} `{value}`"))
}

fn attribute_key_offenders(
    attributes: &BTreeMap<String, Value>,
    label: &str,
) -> (usize, Vec<String>) {
    let mut checked_keys = 0usize;
    let mut offenders = Vec::new();

    for (key, value) in attributes {
        checked_keys += 1;
        if let Some(invalid_key) = string_metadata_offender(key, label) {
            offenders.push(format!("{invalid_key} at `{key}`"));
        }
        collect_nested_attribute_key_offenders(
            value,
            label,
            key,
            &mut checked_keys,
            &mut offenders,
        );
    }

    (checked_keys, offenders)
}

fn collect_nested_attribute_key_offenders(
    value: &Value,
    label: &str,
    path: &str,
    checked_keys: &mut usize,
    offenders: &mut Vec<String>,
) {
    match value {
        Value::Table(table) => {
            for (key, nested_value) in table {
                *checked_keys += 1;
                let key_path = format!("{path}.{key}");
                if let Some(invalid_key) = string_metadata_offender(key, label) {
                    offenders.push(format!("{invalid_key} at `{key_path}`"));
                }
                collect_nested_attribute_key_offenders(
                    nested_value,
                    label,
                    &key_path,
                    checked_keys,
                    offenders,
                );
            }
        }
        Value::Array(values) => {
            for (index, nested_value) in values.iter().enumerate() {
                collect_nested_attribute_key_offenders(
                    nested_value,
                    label,
                    &format!("{path}[{}]", index + 1),
                    checked_keys,
                    offenders,
                );
            }
        }
        _ => {}
    }
}

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
fn production_zui_node_component_names_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_node_components = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                checked_node_components += 1;
                let component = node.component.trim();
                if component.is_empty() {
                    offenders.push(format!(
                        "{} node `{}` declares an empty component type",
                        path.display(),
                        node_id
                    ));
                    continue;
                }
                if component != node.component.as_str() {
                    offenders.push(format!(
                        "{} node `{}` declares whitespace-padded component type `{}`",
                        path.display(),
                        node_id,
                        node.component
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
        checked_node_components > 0,
        "production .zui assets should declare node component types"
    );
    assert!(
        offenders.is_empty(),
        "production .zui node component types must be non-empty and trimmed for stable prototype expansion and authoring diagnostics: {offenders:#?}"
    );
}

#[test]
fn production_zui_slot_metadata_names_are_non_empty_and_trimmed() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_slot_metadata = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                for slot_name in component.slots.keys() {
                    checked_slot_metadata += 1;
                    if let Some(invalid_slot_name) =
                        string_metadata_offender(slot_name, "declared slot name")
                    {
                        offenders.push(format!(
                            "{} component `{}` declares {invalid_slot_name}",
                            path.display(),
                            component_name
                        ));
                    }
                }
            }

            for (node_id, node) in &document.nodes {
                for slot_key in node.slots.keys() {
                    checked_slot_metadata += 1;
                    if let Some(invalid_slot_key) =
                        string_metadata_offender(slot_key, "node slot key")
                    {
                        offenders.push(format!(
                            "{} node `{}` declares {invalid_slot_key}",
                            path.display(),
                            node_id
                        ));
                    }
                }

                for (child_index, child) in node.children.iter().enumerate() {
                    for slot_key in child.slot.keys() {
                        checked_slot_metadata += 1;
                        if let Some(invalid_slot_key) =
                            string_metadata_offender(slot_key, "child mount slot key")
                        {
                            offenders.push(format!(
                                "{} node `{}` child mount #{} declares {invalid_slot_key}",
                                path.display(),
                                node_id,
                                child_index + 1
                            ));
                        }
                    }

                    let Some(slot_name_value) = child.slot.get("name") else {
                        continue;
                    };
                    checked_slot_metadata += 1;
                    let Some(slot_name) = slot_name_value.as_str() else {
                        offenders.push(format!(
                            "{} node `{}` child mount #{} declares a non-string slot.name value",
                            path.display(),
                            node_id,
                            child_index + 1
                        ));
                        continue;
                    };
                    if let Some(invalid_slot_name) =
                        string_metadata_offender(slot_name, "child mount slot.name")
                    {
                        offenders.push(format!(
                            "{} node `{}` child mount #{} declares {invalid_slot_name}",
                            path.display(),
                            node_id,
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
        checked_slot_metadata > 0,
        "production .zui assets should declare slot metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui slot metadata names and keys must be non-empty and trimmed for stable hierarchy and inspector state: {offenders:#?}"
    );
}

#[test]
fn production_zui_event_bindings_are_authorable_and_unique() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_bindings = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let mut binding_ids = BTreeMap::<String, Vec<String>>::new();

            for (node_id, node) in &document.nodes {
                for (binding_index, binding) in node.events.iter().enumerate() {
                    checked_bindings += 1;
                    let binding_label =
                        format!("node `{node_id}` event binding #{}", binding_index + 1);
                    if let Some(invalid_binding_id) =
                        string_metadata_offender(&binding.id, "event binding id")
                    {
                        offenders.push(format!(
                            "{} {binding_label} declares {invalid_binding_id}",
                            path.display()
                        ));
                    } else {
                        binding_ids
                            .entry(binding.id.trim().to_string())
                            .or_default()
                            .push(binding_label.clone());
                    }

                    let has_clean_route = if let Some(route) = binding.route.as_deref() {
                        if let Some(invalid_route) =
                            string_metadata_offender(route, "event binding route")
                        {
                            offenders.push(format!(
                                "{} {binding_label} declares {invalid_route}",
                                path.display()
                            ));
                            false
                        } else {
                            true
                        }
                    } else {
                        false
                    };

                    if !has_clean_route && binding.action.is_none() {
                        offenders.push(format!(
                            "{} {binding_label} declares no route or action target",
                            path.display()
                        ));
                    }
                }
            }

            for (binding_id, binding_labels) in binding_ids {
                if binding_labels.len() > 1 {
                    offenders.push(format!(
                        "{} declares duplicate event binding id `{}` on {binding_labels:?}",
                        path.display(),
                        binding_id
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
        checked_bindings > 0,
        "production .zui assets should declare event bindings"
    );
    assert!(
        offenders.is_empty(),
        "production .zui event bindings must have clean ids, clean route metadata, unique ids inside an asset, and at least one dispatch target: {offenders:#?}"
    );
}

#[test]
fn production_zui_component_style_scopes_are_closed() {
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
                if component.style_scope != UiStyleScope::Closed {
                    offenders.push(format!(
                        "{} component `{}` opens style_scope before public style parts are governed",
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
        "production .zui assets should declare component prototypes"
    );
    assert!(
        offenders.is_empty(),
        "production .zui component style scopes must remain closed until public style parts and style penetration contracts are governed: {offenders:#?}"
    );
}

#[test]
fn production_zui_class_lists_are_clean_and_unique() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_class_lists = 0usize;
    let mut checked_class_tokens = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (component_name, component) in &document.components {
                if !component.default_classes.is_empty() {
                    checked_class_lists += 1;
                    checked_class_tokens += component.default_classes.len();
                    let invalid_classes = class_list_offenders(&component.default_classes);
                    if !invalid_classes.is_empty() {
                        offenders.push(format!(
                            "{} component `{}` default_classes contain {invalid_classes:?}",
                            path.display(),
                            component_name
                        ));
                    }
                }
            }

            for (node_id, node) in &document.nodes {
                if !node.classes.is_empty() {
                    checked_class_lists += 1;
                    checked_class_tokens += node.classes.len();
                    let invalid_classes = class_list_offenders(&node.classes);
                    if !invalid_classes.is_empty() {
                        offenders.push(format!(
                            "{} node `{}` classes contain {invalid_classes:?}",
                            path.display(),
                            node_id
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
        checked_class_lists > 0 && checked_class_tokens > 0,
        "production .zui assets should declare component or node class tokens"
    );
    assert!(
        offenders.is_empty(),
        "production .zui class lists must not contain empty, whitespace-padded, or duplicate class tokens: {offenders:#?}"
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

#[test]
fn production_zui_control_ids_are_unique_within_each_component_asset() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_control_ids = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let mut control_ids = BTreeMap::<String, Vec<String>>::new();

            for (node_id, node) in &document.nodes {
                let Some(control_id) = node.control_id.as_deref() else {
                    continue;
                };
                if let Some(invalid_control_id) = string_metadata_offender(control_id, "control_id")
                {
                    offenders.push(format!(
                        "{} node `{}` declares {invalid_control_id}",
                        path.display(),
                        node_id
                    ));
                    continue;
                }
                checked_control_ids += 1;
                control_ids
                    .entry(control_id.trim().to_string())
                    .or_default()
                    .push(node_id.clone());
            }

            for (control_id, node_ids) in control_ids {
                if node_ids.len() > 1 {
                    offenders.push(format!(
                        "{} declares duplicate control_id `{}` on nodes {node_ids:?}",
                        path.display(),
                        control_id
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
        checked_control_ids > 0,
        "production .zui assets should expose author-facing control ids"
    );
    assert!(
        offenders.is_empty(),
        "production .zui control_id values must be non-empty, trimmed, and unique within each component asset: {offenders:#?}"
    );
}
