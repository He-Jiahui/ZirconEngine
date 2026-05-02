use std::collections::BTreeMap;

use crate::ui::asset_editor::palette::node_accepts_palette_children;
use crate::ui::asset_editor::UiDesignerSelectionModel;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::{
    component::UiDefaultNodeTemplate,
    template::{
        UiAssetDocument, UiChildMount, UiComponentDefinition, UiNodeDefinition,
        UiNodeDefinitionKind, UiStyleScope,
    },
};

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
    let mut component_root = original.clone();
    component_root.node_id = component_root_id.clone();
    let _ = document.components.insert(
        component_name.clone(),
        UiComponentDefinition {
            root: component_root,
            style_scope: UiStyleScope::Closed,
            contract: Default::default(),
            params: BTreeMap::new(),
            slots: BTreeMap::new(),
        },
    );
    document
        .replace_node(
            &node_id,
            UiNodeDefinition {
                node_id: node_id.clone(),
                kind: UiNodeDefinitionKind::Component,
                widget_type: None,
                component: Some(component_name),
                component_ref: None,
                component_api_version: None,
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
        )
        .then_some(node_id)
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
    let target_index = match direction {
        UiTreeMoveDirection::Up if child_index > 0 => child_index - 1,
        UiTreeMoveDirection::Down => {
            let Some(parent) = document.node(&parent_id) else {
                return false;
            };
            if child_index + 1 >= parent.children.len() {
                return false;
            }
            child_index + 1
        }
        _ => return false,
    };
    document.swap_children(&parent_id, child_index, target_index)
}

pub(crate) fn wrap_selected_node(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    widget_type: &str,
) -> Option<String> {
    let node_id = selection.primary_node_id.as_deref()?;
    let (parent_id, child_index) = child_index_in_parent(document, node_id)?;
    let parent_mount = document.child_mount(node_id)?.mount.clone();
    let parent_slot = document.child_mount(node_id)?.slot.clone();
    let wrapped_node = document.remove_node(node_id)?;
    let wrapper_id = unique_node_id(document, &base_node_id(widget_type));
    let mut wrapper = UiDefaultNodeTemplate::native(widget_type).instantiate(
        wrapper_id.clone(),
        Some(unique_control_id(document, widget_type)),
    );
    wrapper.children = vec![new_child_mount(wrapped_node)];
    document
        .insert_child(
            &parent_id,
            child_index,
            UiChildMount {
                mount: parent_mount,
                slot: parent_slot,
                node: wrapper,
            },
        )
        .then_some(wrapper_id)
}

pub(crate) fn unwrap_selected_node(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<String> {
    let wrapper_id = selection.primary_node_id.as_deref()?;
    let wrapper = document.node(wrapper_id)?;
    if wrapper.children.len() != 1 {
        return None;
    }
    let (parent_id, child_index) = child_index_in_parent(document, wrapper_id)?;
    let parent_mount = document.child_mount(wrapper_id)?.mount.clone();
    let parent_slot = document.child_mount(wrapper_id)?.slot.clone();
    let mut wrapper_node = document.remove_node(wrapper_id)?;
    let child_mount = wrapper_node.children.pop()?;
    let child_id = child_mount.node.node_id.clone();
    document
        .insert_child(
            &parent_id,
            child_index,
            UiChildMount {
                mount: parent_mount,
                slot: parent_slot,
                node: child_mount.node,
            },
        )
        .then_some(child_id)
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

fn selected_node_for_component_extraction<'a>(
    document: &'a UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a UiNodeDefinition> {
    let node_id = selection.primary_node_id.as_deref()?;
    let node = document.node(node_id)?;
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

fn child_index_in_parent(document: &UiAssetDocument, child_id: &str) -> Option<(String, usize)> {
    document.child_index_in_parent(child_id)
}

fn reparent_selected_node_into_sibling(
    document: &mut UiAssetDocument,
    node_id: &str,
    into_previous: bool,
) -> Option<String> {
    let (parent_id, child_index) = child_index_in_parent(document, node_id)?;
    let target_id = {
        let parent = document.node(&parent_id)?;
        let target_index = if into_previous {
            child_index.checked_sub(1)?
        } else {
            let next_index = child_index + 1;
            (next_index < parent.children.len()).then_some(next_index)?
        };
        parent.children.get(target_index)?.node.node_id.clone()
    };
    if !document.node(&target_id).is_some_and(node_accepts_children) {
        return None;
    }

    let mount = reset_mount_for_new_parent(UiChildMount {
        mount: None,
        slot: BTreeMap::new(),
        node: document.remove_node(node_id)?,
    });
    if into_previous {
        if !document.push_child(&target_id, mount) {
            return None;
        }
    } else if !document.insert_child(&target_id, 0, mount) {
        return None;
    }
    Some(node_id.to_string())
}

fn reparent_selected_node_outdent(document: &mut UiAssetDocument, node_id: &str) -> Option<String> {
    let (parent_id, _) = child_index_in_parent(document, node_id)?;
    let (grandparent_id, parent_index) = child_index_in_parent(document, &parent_id)?;
    let mount = reset_mount_for_new_parent(UiChildMount {
        mount: None,
        slot: BTreeMap::new(),
        node: document.remove_node(node_id)?,
    });
    if !document.insert_child(&grandparent_id, parent_index + 1, mount) {
        return None;
    }
    Some(node_id.to_string())
}

fn node_accepts_children(node: &UiNodeDefinition) -> bool {
    node_accepts_palette_children(node)
}

fn reset_mount_for_new_parent(mut mount: UiChildMount) -> UiChildMount {
    mount.mount = None;
    mount.slot = BTreeMap::new();
    mount
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

fn new_child_mount(node: UiNodeDefinition) -> UiChildMount {
    UiChildMount {
        mount: None,
        slot: BTreeMap::new(),
        node,
    }
}
