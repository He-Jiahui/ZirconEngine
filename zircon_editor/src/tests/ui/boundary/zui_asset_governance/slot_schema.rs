use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::support::{collect_zui_files, editor_asset_root, runtime_asset_root};

#[derive(Clone, Copy, Debug)]
struct KnownSlotSchema {
    required: bool,
    multiple: bool,
}

type KnownComponentSlotSchemas = BTreeMap<String, BTreeMap<String, KnownSlotSchema>>;

fn known_component_slot_schemas() -> KnownComponentSlotSchemas {
    let registries = [
        UiComponentDescriptorRegistry::editor_showcase(),
        UiComponentDescriptorRegistry::material_editor_foundation(),
    ];
    let mut component_slots = KnownComponentSlotSchemas::new();

    for registry in registries {
        for descriptor in registry.descriptors() {
            let slot_schemas = component_slots.entry(descriptor.id.clone()).or_default();
            for slot in &descriptor.slot_schema {
                slot_schemas
                    .entry(slot.name.clone())
                    .and_modify(|schema| {
                        schema.required |= slot.required;
                        schema.multiple |= slot.multiple;
                    })
                    .or_insert(KnownSlotSchema {
                        required: slot.required,
                        multiple: slot.multiple,
                    });
            }
        }
    }

    component_slots
}

fn slot_schema_names(slot_schemas: &BTreeMap<String, KnownSlotSchema>) -> Vec<String> {
    slot_schemas.keys().cloned().collect()
}

fn slot_schema_name_set(slot_schemas: &BTreeMap<String, KnownSlotSchema>) -> BTreeSet<String> {
    slot_schemas.keys().cloned().collect()
}

#[test]
fn production_zui_child_mount_slot_names_target_declared_component_slots() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let known_component_slots = known_component_slot_schemas();
    let mut checked_assets = 0usize;
    let mut checked_known_child_slots = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let mut component_slots = known_component_slots.clone();
            for (component_name, component) in &document.components {
                component_slots.insert(
                    component_name.clone(),
                    component
                        .slots
                        .iter()
                        .map(|(slot_name, slot_schema)| {
                            (
                                slot_name.clone(),
                                KnownSlotSchema {
                                    required: slot_schema.required,
                                    multiple: slot_schema.multiple,
                                },
                            )
                        })
                        .collect::<BTreeMap<_, _>>(),
                );
            }

            for (parent_node_id, parent_node) in &document.nodes {
                let Some(valid_slot_schemas) = component_slots.get(&parent_node.component) else {
                    continue;
                };

                for (child_index, child) in parent_node.children.iter().enumerate() {
                    let Some(slot_name_value) = child
                        .slot
                        .get("name")
                        .or_else(|| child.slot.get("slot_name"))
                    else {
                        continue;
                    };
                    checked_known_child_slots += 1;
                    let Some(slot_name) = slot_name_value.as_str() else {
                        continue;
                    };
                    if !valid_slot_schemas.contains_key(slot_name) {
                        let valid_slot_names = slot_schema_names(valid_slot_schemas);
                        offenders.push(format!(
                            "{} node `{}` component `{}` child mount #{} targets undeclared slot `{slot_name}`; valid slots are {valid_slot_names:?}",
                            path.display(),
                            parent_node_id,
                            parent_node.component,
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
        checked_known_child_slots > 0,
        "production .zui assets should mount children into known component slot schemas"
    );
    assert!(
        offenders.is_empty(),
        "production .zui child mount slot names must target slots declared by the parent component schema when that schema is known: {offenders:#?}"
    );
}

#[test]
fn production_zui_child_mount_slot_counts_follow_known_component_slot_schemas() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let known_component_slots = known_component_slot_schemas();
    let known_required_slot_schemas = known_component_slots
        .values()
        .flat_map(|slots| slots.values())
        .filter(|slot_schema| slot_schema.required)
        .count();
    let mut checked_assets = 0usize;
    let mut checked_schema_bound_child_slots = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
            let mut component_slots = known_component_slots.clone();
            for (component_name, component) in &document.components {
                component_slots.insert(
                    component_name.clone(),
                    component
                        .slots
                        .iter()
                        .map(|(slot_name, slot_schema)| {
                            (
                                slot_name.clone(),
                                KnownSlotSchema {
                                    required: slot_schema.required,
                                    multiple: slot_schema.multiple,
                                },
                            )
                        })
                        .collect::<BTreeMap<_, _>>(),
                );
            }

            for (parent_node_id, parent_node) in &document.nodes {
                let Some(valid_slot_schemas) = component_slots.get(&parent_node.component) else {
                    continue;
                };
                let mut child_counts = BTreeMap::<String, usize>::new();

                for (child_index, child) in parent_node.children.iter().enumerate() {
                    let Some(slot_name_value) = child
                        .slot
                        .get("name")
                        .or_else(|| child.slot.get("slot_name"))
                    else {
                        continue;
                    };
                    let Some(slot_name) = slot_name_value.as_str() else {
                        continue;
                    };
                    let Some(slot_schema) = valid_slot_schemas.get(slot_name) else {
                        continue;
                    };
                    checked_schema_bound_child_slots += 1;
                    let child_count = child_counts.entry(slot_name.to_owned()).or_default();
                    *child_count += 1;

                    if !slot_schema.multiple && *child_count > 1 {
                        offenders.push(format!(
                            "{} node `{}` component `{}` child mount #{} is the {}th child targeting non-multiple slot `{slot_name}`",
                            path.display(),
                            parent_node_id,
                            parent_node.component,
                            child_index + 1,
                            child_count
                        ));
                    }
                }

                if child_counts.is_empty() {
                    continue;
                }

                for (slot_name, slot_schema) in valid_slot_schemas {
                    if !slot_schema.required {
                        continue;
                    }
                    if !child_counts.contains_key(slot_name) {
                        offenders.push(format!(
                            "{} node `{}` component `{}` fills known slots {child_counts:?} but omits required slot `{slot_name}`",
                            path.display(),
                            parent_node_id,
                            parent_node.component
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
        checked_schema_bound_child_slots > 0,
        "production .zui assets should mount children into known component slot schemas"
    );
    assert!(
        known_required_slot_schemas > 0,
        "known component registries should expose required slot metadata for production .zui governance"
    );
    assert!(
        offenders.is_empty(),
        "production .zui child mounts must respect known parent component slot cardinality constraints: {offenders:#?}"
    );
}

#[test]
fn production_zui_slot_props_target_known_component_slots() {
    let asset_roots = [editor_asset_root(), runtime_asset_root()];
    let known_component_slots = known_component_slot_schemas();
    let mut checked_assets = 0usize;
    let mut checked_slot_props = 0usize;
    let mut offenders = Vec::new();

    for asset_root in &asset_roots {
        for path in collect_zui_files(&asset_root.join("ui")) {
            checked_assets += 1;
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
            let document = UiZuiAssetLoader::load_zui_str(&source)
                .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

            for (node_id, node) in &document.nodes {
                let Some(slot_props_value) = node.props.get("slotProps") else {
                    continue;
                };
                let Some(valid_slot_schemas) = known_component_slots.get(&node.component) else {
                    continue;
                };
                let valid_slot_names = slot_schema_name_set(valid_slot_schemas);
                let Some(slot_props) = slot_props_value.as_table() else {
                    offenders.push(format!(
                        "{} node `{}` component `{}` declares non-table props.slotProps",
                        path.display(),
                        node_id,
                        node.component
                    ));
                    continue;
                };

                for slot_name in slot_props.keys() {
                    checked_slot_props += 1;
                    if !valid_slot_names.contains(slot_name) {
                        offenders.push(format!(
                            "{} node `{}` component `{}` props.slotProps targets unknown slot `{slot_name}`; valid slots are {valid_slot_names:?}",
                            path.display(),
                            node_id,
                            node.component
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
        checked_slot_props > 0,
        "production .zui assets should declare slotProps for known component descriptors"
    );
    assert!(
        offenders.is_empty(),
        "production .zui props.slotProps keys must target slots declared by the known parent component descriptor schema: {offenders:#?}"
    );
}
