use std::collections::{BTreeMap, BTreeSet};

use toml::Value;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiChildMount, UiComponentDefinition, UiNodeDefinition, UiNodeDefinitionKind,
    UiStyleDeclarationBlock,
};

use crate::ui::asset_editor::UiDesignerSelectionModel;

use super::native_slots::{default_native_mount, native_node_accepts_children};
use super::{PaletteInsertMode, UiAssetPaletteEntry, UiAssetPaletteEntryKind};

#[derive(Clone, Debug)]
struct ReferenceConversionPlan {
    component_ref: String,
    params: BTreeMap<String, Value>,
}

pub(crate) fn insert_palette_item_with_placement(
    document: &mut UiAssetDocument,
    target_node_id: &str,
    entry: &UiAssetPaletteEntry,
    mode: PaletteInsertMode,
    placement: &super::UiAssetPaletteInsertionPlacement,
) -> Option<String> {
    enum InsertLocation {
        ChildOf(String),
        AfterChild {
            parent_id: String,
            child_index: usize,
        },
        AppendTo(String),
    }

    let location = match mode {
        PaletteInsertMode::Child => {
            let parent_id = if document.contains_node(target_node_id) {
                target_node_id.to_string()
            } else {
                document.root_node_id()?.to_string()
            };
            document
                .node(&parent_id)
                .filter(|node| child_insert_allowed(node, placement))?;
            InsertLocation::ChildOf(parent_id)
        }
        PaletteInsertMode::After => {
            if let Some((parent_id, child_index)) = child_index_in_parent(document, target_node_id)
            {
                InsertLocation::AfterChild {
                    parent_id,
                    child_index,
                }
            } else {
                let root_id = document.root_node_id()?.to_string();
                document
                    .node(&root_id)
                    .filter(|node| node_accepts_children(node))?;
                InsertLocation::AppendTo(root_id)
            }
        }
    };

    let (node_id, node) = create_node_from_palette_entry(document, entry)?;
    match location {
        InsertLocation::ChildOf(parent_id) | InsertLocation::AppendTo(parent_id) => {
            let child_mount = new_child_mount_for_parent(document, &parent_id, node, placement);
            if !document.push_child(&parent_id, child_mount) {
                return None;
            }
        }
        InsertLocation::AfterChild {
            parent_id,
            child_index,
        } => {
            let child_mount = new_child_mount_with_placement(node, placement);
            if !document.insert_child(&parent_id, child_index + 1, child_mount) {
                return None;
            }
        }
    }
    Some(node_id)
}

pub(crate) fn can_convert_selected_node_to_reference(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    entry: &UiAssetPaletteEntry,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> bool {
    reference_conversion_plan(document, selection, entry, widget_imports).is_some()
}

pub(crate) fn convert_selected_node_to_reference(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    entry: &UiAssetPaletteEntry,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Option<String> {
    let plan = reference_conversion_plan(document, selection, entry, widget_imports)?;
    let node_id = selection.primary_node_id.as_deref()?.to_string();
    let node = document.node_mut(&node_id)?;
    node.kind = UiNodeDefinitionKind::Reference;
    node.widget_type = None;
    node.component = None;
    node.component_ref = Some(plan.component_ref);
    node.slot_name = None;
    node.params = plan.params;
    node.props.clear();
    node.layout = None;
    node.bindings.clear();
    Some(node_id)
}

fn create_node_from_palette_entry(
    document: &UiAssetDocument,
    entry: &UiAssetPaletteEntry,
) -> Option<(String, UiNodeDefinition)> {
    match &entry.kind {
        UiAssetPaletteEntryKind::Native {
            widget_type,
            default_node,
        } => {
            let prefix = if default_node.node_id_prefix.is_empty() {
                widget_type.as_str()
            } else {
                default_node.node_id_prefix.as_str()
            };
            let node_id = unique_node_id(document, &base_node_id(prefix));
            let control_id = Some(unique_control_id(
                document,
                default_node
                    .control_id_prefix
                    .as_deref()
                    .unwrap_or(widget_type),
            ));
            Some((
                node_id.clone(),
                default_node.instantiate(node_id, control_id),
            ))
        }
        UiAssetPaletteEntryKind::Component { component } => {
            let node_id = unique_node_id(document, &base_node_id(component));
            Some((
                node_id.clone(),
                UiNodeDefinition {
                    node_id,
                    kind: UiNodeDefinitionKind::Component,
                    widget_type: None,
                    component: Some(component.clone()),
                    component_ref: None,
                    component_api_version: None,
                    slot_name: None,
                    control_id: Some(unique_control_id(document, component)),
                    classes: Vec::new(),
                    params: BTreeMap::new(),
                    props: BTreeMap::new(),
                    layout: None,
                    bindings: Vec::new(),
                    style_overrides: UiStyleDeclarationBlock::default(),
                    children: Vec::new(),
                },
            ))
        }
        UiAssetPaletteEntryKind::Reference { component_ref } => {
            let label = component_ref
                .split_once('#')
                .map(|(_, component)| component)
                .unwrap_or(component_ref.as_str());
            let node_id = unique_node_id(document, &base_node_id(label));
            Some((
                node_id.clone(),
                UiNodeDefinition {
                    node_id,
                    kind: UiNodeDefinitionKind::Reference,
                    widget_type: None,
                    component: None,
                    component_ref: Some(component_ref.clone()),
                    component_api_version: None,
                    slot_name: None,
                    control_id: Some(unique_control_id(document, label)),
                    classes: Vec::new(),
                    params: BTreeMap::new(),
                    props: BTreeMap::new(),
                    layout: None,
                    bindings: Vec::new(),
                    style_overrides: UiStyleDeclarationBlock::default(),
                    children: Vec::new(),
                },
            ))
        }
    }
}

fn reference_conversion_plan(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    entry: &UiAssetPaletteEntry,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Option<ReferenceConversionPlan> {
    let UiAssetPaletteEntryKind::Reference { component_ref } = &entry.kind else {
        return None;
    };
    let node_id = selection.primary_node_id.as_deref()?;
    let node = document.node(node_id)?;
    if matches!(node.kind, UiNodeDefinitionKind::Slot) {
        return None;
    }
    if node.kind == UiNodeDefinitionKind::Reference
        && node.component_ref.as_deref() == Some(component_ref.as_str())
    {
        return None;
    }
    if !node.bindings.is_empty() || node.layout.is_some() {
        return None;
    }

    let imported = widget_imports.get(component_ref)?;
    let component_name = component_ref.split_once('#')?.1;
    let component = imported.components.get(component_name)?;
    validate_child_mounts_for_component(&node.children, component)?;
    let params = build_reference_params(node, component)?;
    Some(ReferenceConversionPlan {
        component_ref: component_ref.clone(),
        params,
    })
}

fn build_reference_params(
    node: &UiNodeDefinition,
    component: &UiComponentDefinition,
) -> Option<BTreeMap<String, Value>> {
    let allowed = component.params.keys().cloned().collect::<BTreeSet<_>>();
    if node.params.keys().any(|key| !allowed.contains(key)) {
        return None;
    }
    if node.props.keys().any(|key| !allowed.contains(key)) {
        return None;
    }

    let mut params = BTreeMap::new();
    for key in component.params.keys() {
        if let Some(value) = node
            .params
            .get(key)
            .cloned()
            .or_else(|| node.props.get(key).cloned())
        {
            let _ = params.insert(key.clone(), value);
        }
    }
    Some(params)
}

fn validate_child_mounts_for_component(
    children: &[UiChildMount],
    component: &UiComponentDefinition,
) -> Option<()> {
    let mut counts = BTreeMap::<String, usize>::new();
    for child in children {
        let slot_name = child.mount.clone().unwrap_or_default();
        let slot = component.slots.get(&slot_name)?;
        let count = counts.entry(slot_name.clone()).or_insert(0);
        *count += 1;
        if !slot.multiple && *count > 1 {
            return None;
        }
    }

    component
        .slots
        .iter()
        .all(|(slot_name, slot)| !slot.required || counts.contains_key(slot_name))
        .then_some(())
}

fn child_index_in_parent(document: &UiAssetDocument, child_id: &str) -> Option<(String, usize)> {
    document.child_index_in_parent(child_id)
}

fn child_insert_allowed(
    node: &UiNodeDefinition,
    placement: &super::UiAssetPaletteInsertionPlacement,
) -> bool {
    placement.mount.is_some() || node_accepts_children(node)
}

pub(crate) fn node_accepts_palette_children(node: &UiNodeDefinition) -> bool {
    native_node_accepts_children(node)
}

fn node_accepts_children(node: &UiNodeDefinition) -> bool {
    node_accepts_palette_children(node)
}

fn unique_node_id(document: &UiAssetDocument, base: &str) -> String {
    if !document.contains_node(base) {
        return base.to_string();
    }
    for index in 2.. {
        let candidate = format!("{base}_{index}");
        if !document.contains_node(&candidate) {
            return candidate;
        }
    }
    unreachable!("loop should always return a unique node id")
}

fn unique_control_id(document: &UiAssetDocument, label: &str) -> String {
    let base = label
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if !document
        .iter_nodes()
        .any(|node| node.control_id.as_deref() == Some(base.as_str()))
    {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}{index}");
        if !document
            .iter_nodes()
            .any(|node| node.control_id.as_deref() == Some(candidate.as_str()))
        {
            return candidate;
        }
    }
    unreachable!("loop should always return a unique control id")
}

fn base_node_id(label: &str) -> String {
    let normalized = label
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_ascii_lowercase();
    if normalized.is_empty() {
        "node".to_string()
    } else {
        normalized
    }
}

fn new_child_mount_with_placement(
    node: UiNodeDefinition,
    placement: &super::UiAssetPaletteInsertionPlacement,
) -> UiChildMount {
    UiChildMount {
        mount: placement.mount.clone(),
        slot: placement.slot.clone(),
        node,
    }
}

fn new_child_mount_for_parent(
    document: &UiAssetDocument,
    parent_id: &str,
    node: UiNodeDefinition,
    placement: &super::UiAssetPaletteInsertionPlacement,
) -> UiChildMount {
    let mut mount = new_child_mount_with_placement(node, placement);
    if mount.mount.is_none() {
        mount.mount = document.node(parent_id).and_then(default_native_mount);
    }
    mount
}
