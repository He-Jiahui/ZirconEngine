use std::collections::BTreeMap;
use std::str::FromStr;

use toml::Value;
use crate::ui::UiDesignerSelectionModel;
use zircon_ui::{
    UiActionRef, UiAssetDocument, UiBindingRef, UiEventKind, UiNodeDefinition, UiNodeDefinitionKind,
};

use super::style_rule_declarations::parse_declaration_literal;

const BINDING_EVENT_ORDER: &[UiEventKind] = &[
    UiEventKind::Click,
    UiEventKind::DoubleClick,
    UiEventKind::Hover,
    UiEventKind::Press,
    UiEventKind::Release,
    UiEventKind::Change,
    UiEventKind::Submit,
    UiEventKind::Toggle,
    UiEventKind::Focus,
    UiEventKind::Blur,
    UiEventKind::Scroll,
    UiEventKind::Resize,
    UiEventKind::DragBegin,
    UiEventKind::DragUpdate,
    UiEventKind::DragEnd,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum UiBindingActionKind {
    #[default]
    None,
    Route,
    Action,
}

impl UiBindingActionKind {
    fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Route => "Route",
            Self::Action => "Action",
        }
    }

    fn all_labels() -> Vec<String> {
        [Self::None, Self::Route, Self::Action]
            .into_iter()
            .map(|kind| kind.label().to_string())
            .collect()
    }

    fn from_label(value: &str) -> Option<Self> {
        match value.trim() {
            "None" => Some(Self::None),
            "Route" => Some(Self::Route),
            "Action" => Some(Self::Action),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiAssetBindingInspectorFields {
    pub items: Vec<String>,
    pub selected_index: i32,
    pub binding_id: String,
    pub binding_event: String,
    pub binding_event_items: Vec<String>,
    pub binding_event_selected_index: i32,
    pub binding_route: String,
    pub binding_route_target: String,
    pub binding_action_target: String,
    pub binding_action_kind_items: Vec<String>,
    pub binding_action_kind_selected_index: i32,
    pub binding_payload_items: Vec<String>,
    pub binding_payload_selected_index: i32,
    pub binding_payload_key: String,
    pub binding_payload_value: String,
    pub can_edit: bool,
    pub can_delete: bool,
}

impl Default for UiAssetBindingInspectorFields {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: -1,
            binding_id: String::new(),
            binding_event: String::new(),
            binding_event_items: binding_event_items(),
            binding_event_selected_index: -1,
            binding_route: String::new(),
            binding_route_target: String::new(),
            binding_action_target: String::new(),
            binding_action_kind_items: UiBindingActionKind::all_labels(),
            binding_action_kind_selected_index: -1,
            binding_payload_items: Vec::new(),
            binding_payload_selected_index: -1,
            binding_payload_key: String::new(),
            binding_payload_value: String::new(),
            can_edit: false,
            can_delete: false,
        }
    }
}

pub(crate) fn build_binding_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    selected_payload_key: Option<&str>,
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

    let mut fields = UiAssetBindingInspectorFields {
        items,
        selected_index: selected_index.map(|index| index as i32).unwrap_or(-1),
        can_edit: editable,
        can_delete: editable,
        ..UiAssetBindingInspectorFields::default()
    };

    let Some(binding) = selected_binding else {
        return fields;
    };

    let payload_key = selected_payload_key_for_binding(binding, selected_payload_key);
    let payload_entries = binding_payload_entries(binding);
    let selected_payload = payload_key
        .as_deref()
        .and_then(|key| {
            payload_entries
                .iter()
                .position(|(entry_key, _)| entry_key.as_str() == key)
        })
        .and_then(|index| payload_entries.get(index).map(|entry| (index, entry)));

    fields.binding_id = binding.id.clone();
    fields.binding_event = binding.event.to_string();
    fields.binding_event_selected_index = binding_event_index(binding.event)
        .map(|index| index as i32)
        .unwrap_or(-1);
    fields.binding_route = binding_action_target(binding);
    fields.binding_route_target = binding_route_target(binding);
    fields.binding_action_target = binding_action_specific_target(binding);
    fields.binding_action_kind_selected_index = binding_action_kind(binding) as i32;
    fields.binding_payload_items = payload_entries
        .iter()
        .map(|(key, value)| format!("{key} = {}", value.to_string()))
        .collect();
    fields.binding_payload_selected_index = selected_payload
        .map(|(index, _)| index as i32)
        .unwrap_or(-1);
    fields.binding_payload_key = selected_payload
        .map(|(_, (key, _))| key.clone())
        .unwrap_or_default();
    fields.binding_payload_value = selected_payload
        .map(|(_, (_, value))| value.to_string())
        .unwrap_or_default();
    fields
}

pub(crate) fn reconcile_selected_binding_index(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    current: Option<usize>,
) -> Option<usize> {
    selected_node(document, selection)
        .and_then(|node| selected_binding_index_for_node(node, current))
}

pub(crate) fn reconcile_selected_binding_payload_key(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    current: Option<&str>,
) -> Option<String> {
    let binding = selected_node(document, selection)
        .and_then(|node| selected_binding_index_for_node(node, selected_index))
        .and_then(|index| selected_node(document, selection)?.bindings.get(index))?;
    selected_payload_key_for_binding(binding, current)
}

pub(crate) fn add_default_binding(
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
        action: None,
    });
    Some(next_index)
}

pub(crate) fn delete_selected_binding(
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

pub(crate) fn set_selected_binding_id(
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

pub(crate) fn set_selected_binding_event(
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

pub(crate) fn set_selected_binding_action_kind(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    value: &str,
) -> bool {
    let Some(next_kind) = UiBindingActionKind::from_label(value) else {
        return false;
    };
    let Some(binding) =
        selected_binding_mut(document, selection, selected_index.unwrap_or(usize::MAX))
    else {
        return false;
    };
    let previous = binding.clone();
    let current_target = binding_action_target(binding);
    let payload = binding_payload_map(binding);
    apply_binding_action_state(
        binding,
        next_kind,
        normalized_binding_target(&current_target),
        payload,
    );
    *binding != previous
}

pub(crate) fn set_selected_binding_route(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    value: &str,
) -> bool {
    let current_kind = selected_node(document, selection)
        .and_then(|node| selected_binding_index_for_node(node, selected_index))
        .and_then(|index| selected_node(document, selection)?.bindings.get(index))
        .map(binding_action_kind);
    if current_kind == Some(UiBindingActionKind::Action) {
        return set_selected_binding_action_target(document, selection, selected_index, value);
    }
    set_selected_binding_route_target(document, selection, selected_index, value)
}

pub(crate) fn set_selected_binding_route_target(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    value: &str,
) -> bool {
    let Some(binding) =
        selected_binding_mut(document, selection, selected_index.unwrap_or(usize::MAX))
    else {
        return false;
    };
    let previous = binding.clone();
    let payload = binding_payload_map(binding);
    apply_binding_action_state(
        binding,
        UiBindingActionKind::Route,
        normalized_binding_target(value),
        payload,
    );
    *binding != previous
}

pub(crate) fn set_selected_binding_action_target(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    value: &str,
) -> bool {
    let Some(binding) =
        selected_binding_mut(document, selection, selected_index.unwrap_or(usize::MAX))
    else {
        return false;
    };
    let previous = binding.clone();
    let payload = binding_payload_map(binding);
    apply_binding_action_state(
        binding,
        UiBindingActionKind::Action,
        normalized_binding_target(value),
        payload,
    );
    *binding != previous
}

pub(crate) fn upsert_selected_binding_payload(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    payload_key: &str,
    value_literal: &str,
) -> bool {
    let Some(binding) =
        selected_binding_mut(document, selection, selected_index.unwrap_or(usize::MAX))
    else {
        return false;
    };
    let Some(payload_key) = normalized_payload_key(payload_key) else {
        return false;
    };
    let Some(value) = parse_declaration_literal(value_literal) else {
        return false;
    };

    let action = ensure_binding_action_for_payload(binding);
    if action.payload.get(&payload_key) == Some(&value) {
        return false;
    }
    let _ = action.payload.insert(payload_key, value);
    compact_binding_action(binding);
    true
}

pub(crate) fn delete_selected_binding_payload(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    selected_payload_key: Option<&str>,
) -> bool {
    let Some(binding) =
        selected_binding_mut(document, selection, selected_index.unwrap_or(usize::MAX))
    else {
        return false;
    };
    let Some(selected_payload_key) = selected_payload_key else {
        return false;
    };
    let Some(action) = binding.action.as_mut() else {
        return false;
    };
    let removed = action.payload.remove(selected_payload_key).is_some();
    if removed {
        compact_binding_action(binding);
    }
    removed
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

fn binding_event_items() -> Vec<String> {
    BINDING_EVENT_ORDER
        .iter()
        .map(|event| event.native_name().to_string())
        .collect()
}

fn binding_event_index(event: UiEventKind) -> Option<usize> {
    BINDING_EVENT_ORDER
        .iter()
        .position(|candidate| *candidate == event)
}

fn binding_action_kind(binding: &UiBindingRef) -> UiBindingActionKind {
    if binding
        .action
        .as_ref()
        .and_then(|action| action.action.as_ref())
        .is_some()
    {
        UiBindingActionKind::Action
    } else if binding.route.is_some()
        || binding
            .action
            .as_ref()
            .and_then(|action| action.route.as_ref())
            .is_some()
    {
        UiBindingActionKind::Route
    } else if binding
        .action
        .as_ref()
        .map(|action| !action.payload.is_empty())
        .unwrap_or(false)
    {
        UiBindingActionKind::Action
    } else {
        UiBindingActionKind::None
    }
}

fn binding_action_target(binding: &UiBindingRef) -> String {
    match binding_action_kind(binding) {
        UiBindingActionKind::Route => binding_route_target(binding),
        UiBindingActionKind::Action => binding_action_specific_target(binding),
        UiBindingActionKind::None => String::new(),
    }
}

fn binding_route_target(binding: &UiBindingRef) -> String {
    binding
        .route
        .clone()
        .or_else(|| {
            binding
                .action
                .as_ref()
                .and_then(|action| action.route.clone())
        })
        .unwrap_or_default()
}

fn binding_action_specific_target(binding: &UiBindingRef) -> String {
    binding
        .action
        .as_ref()
        .and_then(|action| action.action.clone())
        .unwrap_or_default()
}

fn binding_payload_map(binding: &UiBindingRef) -> BTreeMap<String, Value> {
    binding
        .action
        .as_ref()
        .map(|action| action.payload.clone())
        .unwrap_or_default()
}

fn binding_payload_entries(binding: &UiBindingRef) -> Vec<(String, Value)> {
    binding
        .action
        .as_ref()
        .map(|action| {
            action
                .payload
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn selected_payload_key_for_binding(
    binding: &UiBindingRef,
    current: Option<&str>,
) -> Option<String> {
    let payload = binding.action.as_ref().map(|action| &action.payload)?;
    if payload.is_empty() {
        return None;
    }
    current
        .filter(|key| payload.contains_key(*key))
        .map(str::to_string)
        .or_else(|| payload.keys().next().cloned())
}

fn apply_binding_action_state(
    binding: &mut UiBindingRef,
    kind: UiBindingActionKind,
    target: Option<String>,
    payload: BTreeMap<String, Value>,
) {
    match kind {
        UiBindingActionKind::None => {
            binding.route = None;
            binding.action = None;
        }
        UiBindingActionKind::Route => {
            binding.route = target.clone();
            if payload.is_empty() {
                binding.action = None;
            } else {
                binding.action = Some(UiActionRef {
                    route: target,
                    action: None,
                    payload,
                });
            }
        }
        UiBindingActionKind::Action => {
            binding.route = None;
            if target.is_none() && payload.is_empty() {
                binding.action = None;
            } else {
                binding.action = Some(UiActionRef {
                    route: None,
                    action: target,
                    payload,
                });
            }
        }
    }
}

fn ensure_binding_action_for_payload(binding: &mut UiBindingRef) -> &mut UiActionRef {
    if binding.action.is_none() {
        binding.action = Some(UiActionRef {
            route: binding.route.clone(),
            action: None,
            payload: BTreeMap::new(),
        });
    }
    binding
        .action
        .as_mut()
        .expect("binding action should exist after initialization")
}

fn compact_binding_action(binding: &mut UiBindingRef) {
    let Some(action) = binding.action.as_ref() else {
        return;
    };
    if action.action.is_none() && action.payload.is_empty() {
        binding.route = binding.route.clone().or_else(|| action.route.clone());
        binding.action = None;
    } else if action.action.is_none() && action.route.is_none() && action.payload.is_empty() {
        binding.action = None;
    }
}

fn normalized_binding_id(value: &str, default_id: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_id.to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalized_binding_target(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn normalized_payload_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn format_binding_item(binding: &UiBindingRef) -> String {
    let payload_count = binding
        .action
        .as_ref()
        .map(|action| action.payload.len())
        .unwrap_or(0);
    let payload_suffix = (payload_count > 0)
        .then(|| format!(" (+{payload_count} payload)"))
        .unwrap_or_default();

    match binding_action_kind(binding) {
        UiBindingActionKind::Route => match binding_action_target(binding).as_str() {
            "" => format!("{} | {}{}", binding.event, binding.id, payload_suffix),
            target => format!(
                "{} | {} -> {}{}",
                binding.event, binding.id, target, payload_suffix
            ),
        },
        UiBindingActionKind::Action => match binding_action_target(binding).as_str() {
            "" => format!(
                "{} | {} => Action{}",
                binding.event, binding.id, payload_suffix
            ),
            target => format!(
                "{} | {} => {}{}",
                binding.event, binding.id, target, payload_suffix
            ),
        },
        UiBindingActionKind::None => format!("{} | {}", binding.event, binding.id),
    }
}
