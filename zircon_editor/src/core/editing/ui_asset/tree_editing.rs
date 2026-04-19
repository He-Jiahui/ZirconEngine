use std::collections::{BTreeMap, BTreeSet};

use crate::ui::UiDesignerSelectionModel;
use toml::Value;
use zircon_ui::template::{
    UiChildMount, UiComponentDefinition, UiNodeDefinition, UiNodeDefinitionKind,
    UiStyleDeclarationBlock, UiStyleScope,
};
use zircon_ui::UiAssetDocument;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaletteInsertMode {
    Child,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiTreeMoveDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiTreeReparentDirection {
    IntoPrevious,
    IntoNext,
    Outdent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiAssetPaletteEntryKind {
    Native { widget_type: String },
    Component { component: String },
    Reference { component_ref: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetPaletteEntry {
    pub label: String,
    pub kind: UiAssetPaletteEntryKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiAssetPaletteInsertionPlacement {
    pub mount: Option<String>,
    pub slot: BTreeMap<String, Value>,
}

pub(crate) fn build_palette_entries(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetPaletteEntry> {
    let mut entries = vec![
        native_entry("Container"),
        native_entry("Overlay"),
        native_entry("HorizontalBox"),
        native_entry("VerticalBox"),
        native_entry("FlowBox"),
        native_entry("GridBox"),
        native_entry("ScrollableBox"),
        native_entry("Space"),
        native_entry("Label"),
        native_entry("Image"),
        native_entry("Button"),
        native_entry("TextField"),
    ];
    for component_name in document.components.keys() {
        entries.push(UiAssetPaletteEntry {
            label: format!("Component / {component_name}"),
            kind: UiAssetPaletteEntryKind::Component {
                component: component_name.clone(),
            },
        });
    }
    for reference in widget_imports.keys() {
        let label = reference
            .split_once('#')
            .map(|(_, component)| component.to_string())
            .unwrap_or_else(|| reference.clone());
        entries.push(UiAssetPaletteEntry {
            label: format!("Reference / {label}"),
            kind: UiAssetPaletteEntryKind::Reference {
                component_ref: reference.clone(),
            },
        });
    }
    entries
}

pub(crate) fn insert_palette_item_with_placement(
    document: &mut UiAssetDocument,
    target_node_id: &str,
    entry: &UiAssetPaletteEntry,
    mode: PaletteInsertMode,
    placement: &UiAssetPaletteInsertionPlacement,
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
            let parent_id = if document.nodes.contains_key(target_node_id) {
                target_node_id.to_string()
            } else {
                document.root.as_ref()?.node.clone()
            };
            document
                .nodes
                .get(&parent_id)
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
                let root_id = document.root.as_ref()?.node.clone();
                document
                    .nodes
                    .get(&root_id)
                    .filter(|node| node_accepts_children(node))?;
                InsertLocation::AppendTo(root_id)
            }
        }
    };

    let (node_id, node) = create_node_from_palette_entry(document, entry)?;
    let _ = document.nodes.insert(node_id.clone(), node);
    match location {
        InsertLocation::ChildOf(parent_id) | InsertLocation::AppendTo(parent_id) => {
            document
                .nodes
                .get_mut(&parent_id)?
                .children
                .push(new_child_mount_with_placement(node_id.clone(), placement));
        }
        InsertLocation::AfterChild {
            parent_id,
            child_index,
        } => {
            document.nodes.get_mut(&parent_id)?.children.insert(
                child_index + 1,
                new_child_mount_with_placement(node_id.clone(), placement),
            );
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

pub(crate) fn can_extract_selected_node_to_component(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> bool {
    selected_node_for_component_extraction(document, selection).is_some()
}

pub(crate) fn extract_selected_node_to_component(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    let node_id = selection.primary_node_id.as_deref()?.to_string();
    let original = selected_node_for_component_extraction(document, selection)?.clone();
    let component_name =
        unique_component_name(document, &component_name_label(&node_id, &original));
    let component_root_id =
        unique_node_id(document, &format!("{}_root", base_node_id(&component_name)));
    let _ = document
        .nodes
        .insert(component_root_id.clone(), original.clone());
    let _ = document.components.insert(
        component_name.clone(),
        UiComponentDefinition {
            root: component_root_id,
            style_scope: UiStyleScope::Closed,
            params: BTreeMap::new(),
            slots: BTreeMap::new(),
        },
    );
    let _ = document.nodes.insert(
        node_id.clone(),
        UiNodeDefinition {
            kind: UiNodeDefinitionKind::Component,
            widget_type: None,
            component: Some(component_name),
            component_ref: None,
            slot_name: None,
            control_id: original.control_id.clone(),
            classes: original.classes.clone(),
            params: BTreeMap::new(),
            props: BTreeMap::new(),
            layout: None,
            bindings: Vec::new(),
            style_overrides: original.style_overrides.clone(),
            children: Vec::new(),
        },
    );
    Some(node_id)
}

pub(crate) fn convert_selected_node_to_reference(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    entry: &UiAssetPaletteEntry,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
) -> Option<String> {
    let plan = reference_conversion_plan(document, selection, entry, widget_imports)?;
    let node_id = selection.primary_node_id.as_deref()?.to_string();
    let node = document.nodes.get_mut(&node_id)?;
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

pub(crate) fn move_selected_node(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    direction: UiTreeMoveDirection,
) -> bool {
    let Some(node_id) = selection.primary_node_id.as_deref() else {
        return false;
    };
    let Some((parent_id, child_index)) = child_index_in_parent(document, node_id) else {
        return false;
    };
    let Some(parent) = document.nodes.get_mut(&parent_id) else {
        return false;
    };
    let target_index = match direction {
        UiTreeMoveDirection::Up if child_index > 0 => child_index - 1,
        UiTreeMoveDirection::Down if child_index + 1 < parent.children.len() => child_index + 1,
        _ => return false,
    };
    parent.children.swap(child_index, target_index);
    true
}

pub(crate) fn wrap_selected_node(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    widget_type: &str,
) -> Option<String> {
    let node_id = selection.primary_node_id.as_deref()?;
    let (parent_id, child_index) = child_index_in_parent(document, node_id)?;
    let wrapper_id = unique_node_id(document, &base_node_id(widget_type));
    let wrapper = UiNodeDefinition {
        kind: UiNodeDefinitionKind::Native,
        widget_type: Some(widget_type.to_string()),
        component: None,
        component_ref: None,
        slot_name: None,
        control_id: Some(unique_control_id(document, widget_type)),
        classes: Vec::new(),
        params: BTreeMap::new(),
        props: default_props(widget_type),
        layout: default_layout(widget_type),
        bindings: Vec::new(),
        style_overrides: UiStyleDeclarationBlock::default(),
        children: vec![new_child_mount(node_id.to_string())],
    };
    let _ = document.nodes.insert(wrapper_id.clone(), wrapper);
    let parent = document.nodes.get_mut(&parent_id)?;
    parent.children[child_index].child = wrapper_id.clone();
    Some(wrapper_id)
}

pub(crate) fn unwrap_selected_node(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    let wrapper_id = selection.primary_node_id.as_deref()?;
    let wrapper = document.nodes.get(wrapper_id)?;
    if wrapper.children.len() != 1 {
        return None;
    }
    let child_id = wrapper.children[0].child.clone();
    let (parent_id, child_index) = child_index_in_parent(document, wrapper_id)?;
    let parent = document.nodes.get_mut(&parent_id)?;
    parent.children[child_index].child = child_id.clone();
    let _ = document.nodes.remove(wrapper_id);
    Some(child_id)
}

pub(crate) fn reparent_selected_node(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    direction: UiTreeReparentDirection,
) -> Option<String> {
    let node_id = selection.primary_node_id.as_deref()?.to_string();
    match direction {
        UiTreeReparentDirection::IntoPrevious => {
            reparent_selected_node_into_sibling(document, &node_id, true)
        }
        UiTreeReparentDirection::IntoNext => {
            reparent_selected_node_into_sibling(document, &node_id, false)
        }
        UiTreeReparentDirection::Outdent => reparent_selected_node_outdent(document, &node_id),
    }
}

fn native_entry(widget_type: &str) -> UiAssetPaletteEntry {
    UiAssetPaletteEntry {
        label: format!("Native / {widget_type}"),
        kind: UiAssetPaletteEntryKind::Native {
            widget_type: widget_type.to_string(),
        },
    }
}

fn create_node_from_palette_entry(
    document: &UiAssetDocument,
    entry: &UiAssetPaletteEntry,
) -> Option<(String, UiNodeDefinition)> {
    match &entry.kind {
        UiAssetPaletteEntryKind::Native { widget_type } => {
            let node_id = unique_node_id(document, &base_node_id(widget_type));
            Some((
                node_id,
                UiNodeDefinition {
                    kind: UiNodeDefinitionKind::Native,
                    widget_type: Some(widget_type.clone()),
                    component: None,
                    component_ref: None,
                    slot_name: None,
                    control_id: Some(unique_control_id(document, widget_type)),
                    classes: Vec::new(),
                    params: BTreeMap::new(),
                    props: default_props(widget_type),
                    layout: default_layout(widget_type),
                    bindings: Vec::new(),
                    style_overrides: UiStyleDeclarationBlock::default(),
                    children: Vec::new(),
                },
            ))
        }
        UiAssetPaletteEntryKind::Component { component } => {
            let node_id = unique_node_id(document, &base_node_id(component));
            Some((
                node_id,
                UiNodeDefinition {
                    kind: UiNodeDefinitionKind::Component,
                    widget_type: None,
                    component: Some(component.clone()),
                    component_ref: None,
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
                node_id,
                UiNodeDefinition {
                    kind: UiNodeDefinitionKind::Reference,
                    widget_type: None,
                    component: None,
                    component_ref: Some(component_ref.clone()),
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

#[derive(Clone, Debug)]
struct ReferenceConversionPlan {
    component_ref: String,
    params: BTreeMap<String, Value>,
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
    let node = document.nodes.get(node_id)?;
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

fn selected_node_for_component_extraction<'a>(
    document: &'a UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a UiNodeDefinition> {
    let node_id = selection.primary_node_id.as_deref()?;
    let node = document.nodes.get(node_id)?;
    (!matches!(node.kind, UiNodeDefinitionKind::Slot)).then_some(node)
}

fn component_name_label(node_id: &str, node: &UiNodeDefinition) -> String {
    node.control_id
        .clone()
        .or_else(|| node.component.clone())
        .or_else(|| {
            node.component_ref.as_deref().and_then(|value| {
                value
                    .split_once('#')
                    .map(|(_, component)| component.to_string())
            })
        })
        .or_else(|| node.widget_type.clone())
        .unwrap_or_else(|| node_id.to_string())
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
    document.nodes.iter().find_map(|(parent_id, node)| {
        node.children
            .iter()
            .position(|child| child.child == child_id)
            .map(|index| (parent_id.clone(), index))
    })
}

fn reparent_selected_node_into_sibling(
    document: &mut UiAssetDocument,
    node_id: &str,
    into_previous: bool,
) -> Option<String> {
    let (parent_id, child_index) = child_index_in_parent(document, node_id)?;
    let target_id = {
        let parent = document.nodes.get(&parent_id)?;
        let target_index = if into_previous {
            child_index.checked_sub(1)?
        } else {
            let next_index = child_index + 1;
            (next_index < parent.children.len()).then_some(next_index)?
        };
        parent.children.get(target_index)?.child.clone()
    };
    if !document
        .nodes
        .get(&target_id)
        .is_some_and(node_accepts_children)
    {
        return None;
    }

    let mount = {
        let parent = document.nodes.get_mut(&parent_id)?;
        parent.children.remove(child_index)
    };
    let mount = reset_mount_for_new_parent(mount);
    let target = document.nodes.get_mut(&target_id)?;
    if into_previous {
        target.children.push(mount);
    } else {
        target.children.insert(0, mount);
    }
    Some(node_id.to_string())
}

fn reparent_selected_node_outdent(document: &mut UiAssetDocument, node_id: &str) -> Option<String> {
    let (parent_id, child_index) = child_index_in_parent(document, node_id)?;
    let (grandparent_id, parent_index) = child_index_in_parent(document, &parent_id)?;
    let mount = {
        let parent = document.nodes.get_mut(&parent_id)?;
        parent.children.remove(child_index)
    };
    let mount = reset_mount_for_new_parent(mount);
    let grandparent = document.nodes.get_mut(&grandparent_id)?;
    grandparent.children.insert(parent_index + 1, mount);
    Some(node_id.to_string())
}

fn node_accepts_children(node: &UiNodeDefinition) -> bool {
    matches!(node.kind, UiNodeDefinitionKind::Native)
        && matches!(
            node.widget_type.as_deref(),
            Some(
                "Container"
                    | "Overlay"
                    | "HorizontalBox"
                    | "VerticalBox"
                    | "FlowBox"
                    | "GridBox"
                    | "ScrollableBox"
            )
        )
}

fn child_insert_allowed(
    node: &UiNodeDefinition,
    placement: &UiAssetPaletteInsertionPlacement,
) -> bool {
    placement.mount.is_some() || node_accepts_children(node)
}

fn reset_mount_for_new_parent(mut mount: UiChildMount) -> UiChildMount {
    mount.mount = None;
    mount.slot = BTreeMap::new();
    mount
}

fn default_props(widget_type: &str) -> BTreeMap<String, Value> {
    let mut props = BTreeMap::new();
    match widget_type {
        "Button" => {
            let _ = props.insert("text".to_string(), Value::String("Button".to_string()));
        }
        "Label" => {
            let _ = props.insert("text".to_string(), Value::String("Label".to_string()));
        }
        "TextField" => {
            let _ = props.insert("text".to_string(), Value::String(String::new()));
        }
        _ => {}
    }
    props
}

fn default_layout(widget_type: &str) -> Option<BTreeMap<String, Value>> {
    let mut layout = BTreeMap::new();
    match widget_type {
        "Container" => {
            let _ = layout.insert(
                "container".to_string(),
                table_value(&[("kind", Value::String("Container".to_string()))]),
            );
        }
        "Overlay" => {
            let _ = layout.insert(
                "container".to_string(),
                table_value(&[("kind", Value::String("Overlay".to_string()))]),
            );
        }
        "HorizontalBox" => {
            let _ = layout.insert(
                "container".to_string(),
                table_value(&[
                    ("kind", Value::String("HorizontalBox".to_string())),
                    ("gap", Value::Integer(0)),
                ]),
            );
        }
        "VerticalBox" => {
            let _ = layout.insert(
                "container".to_string(),
                table_value(&[
                    ("kind", Value::String("VerticalBox".to_string())),
                    ("gap", Value::Integer(0)),
                ]),
            );
        }
        "ScrollableBox" => {
            let _ = layout.insert(
                "container".to_string(),
                table_value(&[
                    ("kind", Value::String("ScrollableBox".to_string())),
                    ("axis", Value::String("Vertical".to_string())),
                    ("gap", Value::Integer(0)),
                    ("scrollbar_visibility", Value::String("Auto".to_string())),
                ]),
            );
        }
        _ => {}
    }
    (!layout.is_empty()).then_some(layout)
}

fn unique_node_id(document: &UiAssetDocument, base: &str) -> String {
    if !document.nodes.contains_key(base) {
        return base.to_string();
    }
    for index in 2.. {
        let candidate = format!("{base}_{index}");
        if !document.nodes.contains_key(&candidate) {
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
        .nodes
        .values()
        .any(|node| node.control_id.as_deref() == Some(base.as_str()))
    {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}{index}");
        if !document
            .nodes
            .values()
            .any(|node| node.control_id.as_deref() == Some(candidate.as_str()))
        {
            return candidate;
        }
    }
    unreachable!("loop should always return a unique control id")
}

fn unique_component_name(document: &UiAssetDocument, label: &str) -> String {
    let base = label
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    let base = if base.is_empty() {
        "Component".to_string()
    } else {
        base
    };
    if !document.components.contains_key(&base) {
        return base;
    }
    for index in 2.. {
        let candidate = format!("{base}{index}");
        if !document.components.contains_key(&candidate) {
            return candidate;
        }
    }
    unreachable!("loop should always return a unique component name")
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

fn table_value(entries: &[(&str, Value)]) -> Value {
    Value::Table(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect(),
    )
}

fn new_child_mount(child: String) -> UiChildMount {
    UiChildMount {
        child,
        mount: None,
        slot: BTreeMap::new(),
    }
}

fn new_child_mount_with_placement(
    child: String,
    placement: &UiAssetPaletteInsertionPlacement,
) -> UiChildMount {
    UiChildMount {
        child,
        mount: placement.mount.clone(),
        slot: placement.slot.clone(),
    }
}
