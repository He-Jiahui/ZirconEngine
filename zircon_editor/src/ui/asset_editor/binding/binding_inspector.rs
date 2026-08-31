use std::collections::BTreeMap;
use std::str::FromStr;

use crate::ui::asset_editor::preview::preview_mock::UiAssetPreviewMockState;
use crate::ui::asset_editor::value_path::{
    get_value_at_path, parse_value_path, set_value_at_path, UiAssetTomlPathSegment,
};
use crate::ui::asset_editor::UiDesignerSelectionModel;
use toml::Value;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    template::{
        UiActionPayloadFieldName, UiActionRef, UiAssetDocument, UiBindingRef,
        UiBindingSchemaNameKind, UiNodeDefinition, UiNodeDefinitionKind,
    },
};

use crate::ui::asset_editor::style::style_rule_declarations::parse_declaration_literal;

mod payload_editing;
#[path = "payload_suggestions.rs"]
mod payload_suggestions;
#[path = "schema_projection.rs"]
mod schema_projection;

use payload_editing::*;

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
    UiEventKind::Drop,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum UiBindingActionKind {
    #[default]
    None,
    Route,
    Action,
}

impl UiBindingActionKind {
    const ALL: [Self; 3] = [Self::None, Self::Route, Self::Action];

    fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Route => "Route",
            Self::Action => "Action",
        }
    }

    fn all_labels() -> Vec<String> {
        Self::ALL
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

    fn schema_name_kind(self) -> Option<UiBindingSchemaNameKind> {
        match self {
            Self::None => None,
            Self::Route => Some(UiBindingSchemaNameKind::Route),
            Self::Action => Some(UiBindingSchemaNameKind::Action),
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
    pub binding_route_suggestion_items: Vec<String>,
    pub binding_action_suggestion_items: Vec<String>,
    pub binding_action_kind_items: Vec<String>,
    pub binding_action_kind_selected_index: i32,
    pub binding_payload_items: Vec<String>,
    pub binding_payload_selected_index: i32,
    pub binding_payload_key: String,
    pub binding_payload_value: String,
    pub binding_payload_suggestion_items: Vec<String>,
    pub binding_schema_items: Vec<String>,
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
            binding_route_suggestion_items: Vec::new(),
            binding_action_suggestion_items: Vec::new(),
            binding_action_kind_items: UiBindingActionKind::all_labels(),
            binding_action_kind_selected_index: -1,
            binding_payload_items: Vec::new(),
            binding_payload_selected_index: -1,
            binding_payload_key: String::new(),
            binding_payload_value: String::new(),
            binding_payload_suggestion_items: Vec::new(),
            binding_schema_items: Vec::new(),
            can_edit: false,
            can_delete: false,
        }
    }
}

pub(crate) fn build_binding_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    preview_mock_state: &UiAssetPreviewMockState,
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

    let payload_entries = binding_payload_item_entries(binding);
    let payload_key = selected_payload_key_from_entries(&payload_entries, selected_payload_key);
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
    fields.binding_route_suggestion_items = binding_route_suggestions(node, binding);
    fields.binding_action_suggestion_items = binding_action_suggestions(node, binding);
    fields.binding_action_kind_selected_index = binding_action_kind(binding) as i32;
    fields.binding_payload_items = payload_entries
        .iter()
        .map(|(key, value)| format!("{key} = {}", value))
        .collect();
    fields.binding_payload_suggestion_items =
        binding_payload_suggestions(binding, payload_key.as_deref())
            .into_iter()
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
    fields.binding_schema_items = schema_projection::build_binding_schema_items(
        document,
        selection.primary_node_id.as_deref().unwrap_or_default(),
        preview_mock_state,
        binding,
    );
    fields
}

pub(crate) fn selected_binding_count(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> usize {
    selected_node(document, selection).map_or(0, |node| node.bindings.len())
}

pub(crate) fn binding_event_option(index: usize) -> Option<&'static str> {
    BINDING_EVENT_ORDER
        .get(index)
        .map(|event| event.native_name())
}

pub(crate) fn binding_action_kind_option(index: usize) -> Option<&'static str> {
    UiBindingActionKind::ALL.get(index).map(|kind| kind.label())
}

pub(crate) fn selected_binding_route_suggestion(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    suggestion_index: usize,
) -> Option<String> {
    let node = selected_node(document, selection)?;
    let index = selected_binding_index_for_node(node, selected_index)?;
    let binding = node.bindings.get(index)?;
    binding_route_suggestions(node, binding)
        .into_iter()
        .nth(suggestion_index)
}

pub(crate) fn selected_binding_action_suggestion(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    suggestion_index: usize,
) -> Option<String> {
    let node = selected_node(document, selection)?;
    let index = selected_binding_index_for_node(node, selected_index)?;
    let binding = node.bindings.get(index)?;
    binding_action_suggestions(node, binding)
        .into_iter()
        .nth(suggestion_index)
}

pub(crate) fn selected_binding_payload_key(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    item_index: usize,
) -> Option<String> {
    let node = selected_node(document, selection)?;
    let index = selected_binding_index_for_node(node, selected_index)?;
    let binding = node.bindings.get(index)?;
    binding_payload_item_entries(binding)
        .into_iter()
        .nth(item_index)
        .map(|(key, _)| key)
}

pub(crate) fn selected_binding_payload_suggestion(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    selected_payload_key: Option<&str>,
    suggestion_index: usize,
) -> Option<(String, Value)> {
    let node = selected_node(document, selection)?;
    let index = selected_binding_index_for_node(node, selected_index)?;
    let binding = node.bindings.get(index)?;
    binding_payload_suggestions(binding, selected_payload_key)
        .into_iter()
        .nth(suggestion_index)
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
    let binding = selected_node(document, selection).and_then(|node| {
        selected_binding_index_for_node(node, selected_index)
            .and_then(|index| node.bindings.get(index))
    })?;
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
        component_event: None,
        id: default_id,
        event: UiEventKind::Click,
        mode: Default::default(),
        route: None,
        action: None,
        targets: Vec::new(),
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
        next_kind
            .schema_name_kind()
            .and_then(|kind| normalized_binding_target(&current_target, kind)),
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
        .and_then(|node| {
            selected_binding_index_for_node(node, selected_index)
                .and_then(|index| node.bindings.get(index))
        })
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
        normalized_binding_target(value, UiBindingSchemaNameKind::Route),
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
        normalized_binding_target(value, UiBindingSchemaNameKind::Action),
        payload,
    );
    *binding != previous
}

pub(crate) fn upsert_selected_binding_payload(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    selected_payload_key: Option<&str>,
    payload_key: &str,
    value_literal: &str,
) -> Option<String> {
    let Some(binding) =
        selected_binding_mut(document, selection, selected_index.unwrap_or(usize::MAX))
    else {
        return None;
    };
    let Some((resolved_payload_key, path)) =
        resolve_binding_payload_upsert_path(binding, selected_payload_key, payload_key)
    else {
        return None;
    };
    let Some(value) = parse_declaration_literal(value_literal) else {
        return None;
    };

    let mut payload_root = binding_payload_root_value(binding);
    if get_value_at_path(&payload_root, &path) == Some(&value) {
        return None;
    }
    if set_value_at_path(&mut payload_root, &path, Some(value)).is_err() {
        return None;
    }
    let Some(table) = payload_root.as_table() else {
        return None;
    };
    let action = ensure_binding_action_for_payload(binding);
    action.payload = table.clone().into_iter().collect();
    compact_binding_action(binding);
    Some(resolved_payload_key)
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
    let Some(path) = parse_value_path(selected_payload_key) else {
        return false;
    };
    let mut payload_root = binding_payload_root_value(binding);
    if set_value_at_path(&mut payload_root, &path, None).is_err() {
        return false;
    }
    let Some(table) = payload_root.as_table() else {
        return false;
    };
    let Some(action) = binding.action.as_mut() else {
        return false;
    };
    let next_payload = table.clone().into_iter().collect();
    let removed = action.payload != next_payload;
    action.payload = next_payload;
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
        .and_then(|node_id| document.node(node_id))
}

fn editable_selected_node_mut<'a>(
    document: &'a mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> Option<&'a mut UiNodeDefinition> {
    let node = selection
        .primary_node_id
        .as_deref()
        .and_then(|node_id| document.node_mut(node_id))?;
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

fn binding_payload_entries<'a>(
    binding: &'a UiBindingRef,
) -> impl Iterator<Item = (&'a String, &'a Value)> {
    binding
        .action
        .iter()
        .flat_map(|action| action.payload.iter())
}

fn binding_payload_item_entries<'a>(binding: &'a UiBindingRef) -> Vec<(String, &'a Value)> {
    let mut entries = Vec::new();
    if let Some(action) = binding.action.as_ref() {
        for (key, value) in &action.payload {
            collect_binding_payload_item_entries(value, Some(key.as_str()), &mut entries);
        }
    }
    entries
}
