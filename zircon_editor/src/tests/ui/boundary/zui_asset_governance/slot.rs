use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::metadata::{string_metadata_offender, string_token_metadata_offender};
use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

const CHILD_MOUNT_SLOT_KEYS: &[&str] = &["name", "slot_name"];

fn slot_alias_consistency_offender(
    attributes: &BTreeMap<String, toml::Value>,
    label: &str,
) -> Option<String> {
    let (Some(name), Some(slot_name)) = (attributes.get("name"), attributes.get("slot_name"))
    else {
        return None;
    };
    let Some(name) = name.as_str() else {
        return Some(format!("{label} declares non-string name alias"));
    };
    let Some(slot_name) = slot_name.as_str() else {
        return Some(format!("{label} declares non-string slot_name alias"));
    };
    (name != slot_name).then(|| {
        format!("{label} declares conflicting name `{name}` and slot_name `{slot_name}` aliases")
    })
}

#[test]
fn production_zui_child_mount_slot_metadata_uses_known_keys() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_child_slot_maps = 0usize;
    let mut checked_child_slot_keys = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                for (child_index, child) in node.children.iter().enumerate() {
                    if child.slot.is_empty() {
                        continue;
                    }
                    checked_child_slot_maps += 1;
                    for slot_key in child.slot.keys() {
                        checked_child_slot_keys += 1;
                        if !CHILD_MOUNT_SLOT_KEYS.contains(&slot_key.as_str()) {
                            offenders.push(format!(
                                "{} node `{}` child mount #{} declares unknown child mount slot key `{slot_key}`; expected one of {CHILD_MOUNT_SLOT_KEYS:?}",
                                path.display(),
                                node_id,
                                child_index + 1
                            ));
                        }
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
        checked_child_slot_maps > 0 && checked_child_slot_keys > 0,
        "production .zui assets should declare child mount slot metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui child mount slot metadata must stay inside the named-slot vocabulary so projection metadata is not confused with future parent-container layout slot fields: {offenders:#?}"
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
fn production_zui_slot_name_aliases_are_consistent() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_slot_name_sources = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                if node.component == "Slot"
                    && (node.props.contains_key("name") || node.props.contains_key("slot_name"))
                {
                    checked_slot_name_sources += 1;
                    if let Some(offender) =
                        slot_alias_consistency_offender(&node.props, "Slot placeholder")
                    {
                        offenders.push(format!("{} node `{}` {offender}", path.display(), node_id));
                    }
                }

                for (child_index, child) in node.children.iter().enumerate() {
                    if !child.slot.contains_key("name") && !child.slot.contains_key("slot_name") {
                        continue;
                    }
                    checked_slot_name_sources += 1;
                    if let Some(offender) =
                        slot_alias_consistency_offender(&child.slot, "child mount slot")
                    {
                        offenders.push(format!(
                            "{} node `{}` child mount #{} {offender}",
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
        checked_slot_name_sources > 0,
        "production .zui assets should declare slot name metadata"
    );
    assert!(
        offenders.is_empty(),
        "production .zui slot name aliases must not disagree when both name and slot_name are present: {offenders:#?}"
    );
}

#[test]
fn production_zui_slot_placeholders_and_declared_component_slots_are_bidirectional() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let mut checked_assets = 0usize;
    let mut checked_slot_placeholders = 0usize;
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
            let mut projected_slot_names = BTreeSet::new();

            for (node_id, node) in &document.nodes {
                if node.component != "Slot" {
                    continue;
                }
                checked_slot_placeholders += 1;
                let slot_name_value = node
                    .props
                    .get("name")
                    .or_else(|| node.props.get("slot_name"));
                let Some(slot_name_value) = slot_name_value else {
                    offenders.push(format!(
                        "{} component `{}` Slot node `{}` declares no props.name or props.slot_name",
                        path.display(),
                        component_name,
                        node_id
                    ));
                    continue;
                };
                let Some(slot_name) = slot_name_value.as_str() else {
                    offenders.push(format!(
                        "{} component `{}` Slot node `{}` declares a non-string slot placeholder name",
                        path.display(),
                        component_name,
                        node_id
                    ));
                    continue;
                };
                if let Some(invalid_slot_name) =
                    string_token_metadata_offender(slot_name, "slot placeholder name")
                {
                    offenders.push(format!(
                        "{} component `{}` Slot node `{}` declares {invalid_slot_name}",
                        path.display(),
                        component_name,
                        node_id
                    ));
                    continue;
                }
                if !component.slots.contains_key(slot_name) {
                    offenders.push(format!(
                        "{} component `{}` Slot node `{}` references undeclared slot `{slot_name}`",
                        path.display(),
                        component_name,
                        node_id
                    ));
                    continue;
                }
                projected_slot_names.insert(slot_name.to_owned());
            }

            for slot_name in component.slots.keys() {
                if !projected_slot_names.contains(slot_name) {
                    offenders.push(format!(
                        "{} component `{}` declares slot `{slot_name}` without a Slot placeholder node",
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
        checked_slot_placeholders > 0,
        "production .zui assets should declare Slot placeholder nodes"
    );
    assert!(
        offenders.is_empty(),
        "production .zui Slot placeholder nodes and declared component slots must stay bidirectional for stable projection: {offenders:#?}"
    );
}
