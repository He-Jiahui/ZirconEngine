use std::str::FromStr;

use zircon_editor_ui::UiDesignerSelectionModel;
use zircon_ui::{
    UiAssetDocument, UiBindingRef, UiEventKind, UiNodeDefinition, UiNodeDefinitionKind,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UiAssetBindingInspectorFields {
    pub items: Vec<String>,
    pub selected_index: i32,
    pub binding_id: String,
    pub binding_event: String,
    pub binding_route: String,
    pub can_edit: bool,
    pub can_delete: bool,
}

pub(super) fn build_binding_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
) -> UiAssetBindingInspectorFields {
    let Some(node) = selected_node(document, selection) else {
        return UiAssetBindingInspectorFields::default();
    };
    let items = node
        .bindings
        .iter()
        .map(format_binding_item)
        .collect::<Vec<_>>();
    let selected_index = selected_binding_index_for_node(node, selected_index);
    let selected_binding = selected_index.and_then(|index| node.bindings.get(index));
    let editable = !matches!(node.kind, UiNodeDefinitionKind::Slot) && selected_binding.is_some();
    UiAssetBindingInspectorFields {
        items,
        selected_index: selected_index.map(|index| index as i32).unwrap_or(-1),
        binding_id: selected_binding
            .map(|binding| binding.id.clone())
            .unwrap_or_default(),
        binding_event: selected_binding
            .map(|binding| binding.event.to_string())
            .unwrap_or_default(),
        binding_route: selected_binding
            .and_then(|binding| binding.route.clone())
            .unwrap_or_default(),
        can_edit: editable,
        can_delete: editable,
    }
}

pub(super) fn reconcile_selected_binding_index(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    current: Option<usize>,
) -> Option<usize> {
    selected_node(document, selection)
        .and_then(|node| selected_binding_index_for_node(node, current))
}

pub(super) fn add_default_binding(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<usize> {
    let default_id = default_binding_id(document, selection, UiEventKind::Click)?;
    let node = editable_selected_node_mut(document, selection)?;
    let next_index = node.bindings.len();
    node.bindings.push(UiBindingRef {
        id: default_id,
        event: UiEventKind::Click,
        route: None,
    });
    Some(next_index)
}

pub(super) fn delete_selected_binding(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
) -> bool {
    let Some(index) = selected_index else {
        return false;
    };
    let Some(node) = editable_selected_node_mut(document, selection) else {
        return false;
    };
    if index >= node.bindings.len() {
        return false;
    }
    node.bindings.remove(index);
    true
}

pub(super) fn set_selected_binding_id(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    value: &str,
) -> bool {
    let Some(index) = selected_index else {
        return false;
    };
    let Some(default_id) = default_binding_id_for_existing_binding(document, selection, index)
    else {
        return false;
    };
    let next = normalized_binding_id(value, &default_id);
    let Some(binding) = selected_binding_mut(document, selection, index) else {
        return false;
    };
    if binding.id == next {
        return false;
    }
    binding.id = next;
    true
}

pub(super) fn set_selected_binding_event(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    value: &str,
) -> Result<bool, &'static str> {
    let Some(index) = selected_index else {
        return Ok(false);
    };
    let next = UiEventKind::from_str(value.trim()).map_err(|_| "binding.event")?;
    let Some(binding) = selected_binding_mut(document, selection, index) else {
        return Ok(false);
    };
    if binding.event == next {
        return Ok(false);
    }
    binding.event = next;
    Ok(true)
}

pub(super) fn set_selected_binding_route(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    value: &str,
) -> bool {
    let Some(index) = selected_index else {
        return false;
    };
    let next = normalized_binding_route(value);
    let Some(binding) = selected_binding_mut(document, selection, index) else {
        return false;
    };
    if binding.route == next {
        return false;
    }
    binding.route = next;
    true
}

fn selected_node<'a>(
    document: &'a UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a UiNodeDefinition> {
    selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get(node_id))
}

fn editable_selected_node_mut<'a>(
    document: &'a mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a mut UiNodeDefinition> {
    let node = selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.nodes.get_mut(node_id))?;
    (!matches!(node.kind, UiNodeDefinitionKind::Slot)).then_some(node)
}

fn selected_binding_mut<'a>(
    document: &'a mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    index: usize,
) -> Option<&'a mut UiBindingRef> {
    editable_selected_node_mut(document, selection).and_then(|node| node.bindings.get_mut(index))
}

fn default_binding_id_for_existing_binding(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    index: usize,
) -> Option<String> {
    let event = selected_node(document, selection)?
        .bindings
        .get(index)?
        .event;
    default_binding_id(document, selection, event)
}

fn default_binding_id(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    event: UiEventKind,
) -> Option<String> {
    let node = selected_node(document, selection)?;
    let node_label = node
        .control_id
        .clone()
        .or_else(|| selection.primary_node_id.clone())
        .unwrap_or_else(|| "Binding".to_string());
    Some(format!("{node_label}/{}", event.native_name()))
}

fn selected_binding_index_for_node(
    node: &UiNodeDefinition,
    current: Option<usize>,
) -> Option<usize> {
    if matches!(node.kind, UiNodeDefinitionKind::Slot) || node.bindings.is_empty() {
        return None;
    }
    Some(current.unwrap_or(0).min(node.bindings.len() - 1))
}

fn normalized_binding_id(value: &str, default_id: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_id.to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalized_binding_route(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn format_binding_item(binding: &UiBindingRef) -> String {
    match binding.route.as_deref() {
        Some(route) => format!("{} | {} -> {}", binding.event, binding.id, route),
        None => format!("{} | {}", binding.event, binding.id),
    }
}
