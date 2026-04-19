use std::collections::BTreeMap;
use std::str::FromStr;

use crate::core::editing::ui_asset::preview::preview_mock::UiAssetPreviewMockState;
use crate::core::editing::ui_asset::value_path::{
    get_value_at_path, parse_value_path, set_value_at_path, UiAssetTomlPathSegment,
};
use crate::ui::UiDesignerSelectionModel;
use toml::Value;
use zircon_runtime::ui::template::{UiActionRef, UiBindingRef};
use zircon_runtime::ui::template::{UiNodeDefinition, UiNodeDefinitionKind};
use zircon_runtime::ui::{binding::UiEventKind, template::UiAssetDocument};

use super::style_rule_declarations::parse_declaration_literal;

#[path = "binding/payload_suggestions.rs"]
mod payload_suggestions;
#[path = "binding/schema_projection.rs"]
mod schema_projection;

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

    let payload_key = selected_payload_key_for_binding(binding, selected_payload_key);
    let payload_entries = binding_payload_item_entries(binding);
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
    fields.binding_schema_items = binding_schema_items(binding);
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
    let removed = action.payload != table.clone().into_iter().collect();
    action.payload = table.clone().into_iter().collect();
    if removed {
        compact_binding_action(binding);
    }
    removed
}

pub(crate) fn apply_selected_binding_payload_suggestion(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    selected_payload_key: Option<&str>,
    suggestion_index: usize,
) -> Option<String> {
    let binding = selected_node(document, selection)
        .and_then(|node| selected_binding_index_for_node(node, selected_index))
        .and_then(|index| selected_node(document, selection)?.bindings.get(index))
        .cloned();
    let Some((payload_key, payload_value)) = binding
        .as_ref()
        .map(|binding| binding_payload_suggestions(binding, selected_payload_key))
        .and_then(|suggestions| suggestions.get(suggestion_index).cloned())
    else {
        return None;
    };

    upsert_selected_binding_payload(
        document,
        selection,
        selected_index,
        selected_payload_key,
        &payload_key,
        &payload_value.to_string(),
    )
}

pub(crate) fn apply_selected_binding_route_suggestion(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    suggestion_index: usize,
) -> bool {
    let Some(node) = selected_node(document, selection) else {
        return false;
    };
    let Some(index) = selected_binding_index_for_node(node, selected_index) else {
        return false;
    };
    let Some(binding) = node.bindings.get(index) else {
        return false;
    };
    let Some(target) = binding_route_suggestions(node, binding)
        .get(suggestion_index)
        .cloned()
    else {
        return false;
    };
    set_selected_binding_route_target(document, selection, selected_index, &target)
}

pub(crate) fn apply_selected_binding_action_suggestion(
    document: &mut UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    selected_index: Option<usize>,
    suggestion_index: usize,
) -> bool {
    let Some(node) = selected_node(document, selection) else {
        return false;
    };
    let Some(index) = selected_binding_index_for_node(node, selected_index) else {
        return false;
    };
    let Some(binding) = node.bindings.get(index) else {
        return false;
    };
    let Some(target) = binding_action_suggestions(node, binding)
        .get(suggestion_index)
        .cloned()
    else {
        return false;
    };
    set_selected_binding_action_target(document, selection, selected_index, &target)
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

fn binding_payload_item_entries(binding: &UiBindingRef) -> Vec<(String, Value)> {
    let payload_root = binding_payload_root_value(binding);
    let mut entries = Vec::new();
    collect_binding_payload_item_entries(&payload_root, None, &mut entries);
    entries
}

fn binding_payload_suggestions(
    binding: &UiBindingRef,
    selected_payload_key: Option<&str>,
) -> Vec<(String, Value)> {
    let root_suggestions = binding_root_payload_suggestions(binding);
    let current_payload_root = binding_payload_root_value(binding);
    payload_suggestions::contextual_binding_payload_suggestions(
        root_suggestions.as_slice(),
        &current_payload_root,
        selected_payload_key,
    )
    .unwrap_or(root_suggestions)
}

fn binding_root_payload_suggestions(binding: &UiBindingRef) -> Vec<(String, Value)> {
    if let Some(target_specific) = binding_target_payload_suggestions(binding) {
        return target_specific;
    }

    match binding.event {
        UiEventKind::Click
        | UiEventKind::DoubleClick
        | UiEventKind::Press
        | UiEventKind::Release
        | UiEventKind::Submit => vec![
            ("confirm".to_string(), Value::Boolean(true)),
            ("channel".to_string(), Value::String("toolbar".to_string())),
            (
                "source".to_string(),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::Change => vec![
            ("value".to_string(), Value::String("preview".to_string())),
            ("committed".to_string(), Value::Boolean(true)),
            (
                "source".to_string(),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::Toggle => vec![
            ("checked".to_string(), Value::Boolean(true)),
            (
                "source".to_string(),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::Scroll => vec![
            ("axis".to_string(), Value::String("vertical".to_string())),
            ("delta".to_string(), Value::Integer(1)),
            (
                "source".to_string(),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::DragBegin | UiEventKind::DragUpdate | UiEventKind::DragEnd => vec![
            ("axis".to_string(), Value::String("x".to_string())),
            ("delta".to_string(), Value::Integer(0)),
            (
                "source".to_string(),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        _ => vec![(
            "source".to_string(),
            Value::String(event_source_tag(binding.event)),
        )],
    }
}

fn binding_schema_items(binding: &UiBindingRef) -> Vec<String> {
    let mut items = vec![format!("event [UiEvent] = {}", binding.event.native_name())];
    match binding_action_kind(binding) {
        UiBindingActionKind::Route => {
            items.push(format!(
                "route.target [Route] = {}",
                binding_route_target(binding)
            ));
        }
        UiBindingActionKind::Action => {
            items.push(format!(
                "action.target [EditorAction] = {}",
                binding_action_specific_target(binding)
            ));
        }
        UiBindingActionKind::None => {
            items.push("action.kind [None]".to_string());
        }
    }
    for (key, value) in binding_schema_payload_entries(binding) {
        items.push(format!(
            "payload.{key} [{}] default = {}",
            binding_value_kind_label(&value),
            binding_schema_value_literal(&value)
        ));
    }
    items
}

fn binding_schema_payload_entries(binding: &UiBindingRef) -> Vec<(String, Value)> {
    if binding_action_kind(binding) == UiBindingActionKind::Action {
        let action_target = binding_action_specific_target(binding);
        if action_target.contains("SaveProject") {
            return vec![
                ("confirm".to_string(), Value::Boolean(true)),
                (
                    "source".to_string(),
                    Value::String(event_source_tag(binding.event)),
                ),
            ];
        }
    }
    binding_root_payload_suggestions(binding)
}

fn binding_target_payload_suggestions(binding: &UiBindingRef) -> Option<Vec<(String, Value)>> {
    match binding_action_kind(binding) {
        UiBindingActionKind::Route => {
            let route_target = binding_route_target(binding);
            if route_target.contains("Selection.Changed") {
                return Some(vec![
                    (
                        "primary".to_string(),
                        Value::String("SelectedNode".to_string()),
                    ),
                    (
                        "selection_ids".to_string(),
                        Value::Array(vec![Value::String("SelectedNode".to_string())]),
                    ),
                    (
                        "context".to_string(),
                        toml::Value::Table(
                            [
                                ("additive".to_string(), Value::Boolean(false)),
                                ("source".to_string(), Value::String("hierarchy".to_string())),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ]);
            }
            if route_target.contains("Form.ValueChanged") {
                return Some(vec![
                    ("value".to_string(), Value::String("preview".to_string())),
                    ("committed".to_string(), Value::Boolean(true)),
                    (
                        "fields".to_string(),
                        Value::Array(vec![Value::String("title".to_string())]),
                    ),
                    (
                        "context".to_string(),
                        toml::Value::Table(
                            [
                                (
                                    "source".to_string(),
                                    Value::String(event_source_tag(binding.event)),
                                ),
                                ("subject".to_string(), Value::String("field".to_string())),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ]);
            }
        }
        UiBindingActionKind::Action => {
            let action_target = binding_action_specific_target(binding);
            if action_target.contains("ToggleVisibility") {
                return Some(vec![
                    ("checked".to_string(), Value::Boolean(true)),
                    (
                        "selection_ids".to_string(),
                        Value::Array(vec![Value::String("SelectedNode".to_string())]),
                    ),
                    (
                        "context".to_string(),
                        toml::Value::Table(
                            [
                                ("scope".to_string(), Value::String("selection".to_string())),
                                (
                                    "source".to_string(),
                                    Value::String(event_source_tag(binding.event)),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ]);
            }
        }
        UiBindingActionKind::None => {}
    }

    None
}

fn binding_value_kind_label(value: &Value) -> &'static str {
    match value {
        Value::String(text) if text.trim_start().starts_with('=') => "Expression",
        Value::Boolean(_) => "Bool",
        Value::Integer(_) | Value::Float(_) => "Number",
        Value::Array(_) => "Collection",
        Value::Table(_) => "Object",
        _ => "Text",
    }
}

fn binding_schema_value_literal(value: &Value) -> String {
    match value {
        Value::String(text) => Value::String(text.clone()).to_string(),
        _ => value.to_string(),
    }
}

fn binding_route_suggestions(node: &UiNodeDefinition, binding: &UiBindingRef) -> Vec<String> {
    let mut suggestions = Vec::new();
    let keywords = binding_keywords(node);
    if binding_event_supports_keyword_shortcuts(binding.event) && is_save_like(&keywords) {
        suggestions.push("MenuAction.SaveProject".to_string());
    }
    match binding.event {
        UiEventKind::Click | UiEventKind::DoubleClick | UiEventKind::Submit => {
            suggestions.push("MenuAction.OpenProject".to_string());
            suggestions.push("MenuAction.SaveLayout".to_string());
            suggestions.push(format!("Route.{}", binding_route_slug(node, binding)));
        }
        UiEventKind::Change => {
            suggestions.push("Route.Selection.Changed".to_string());
            suggestions.push("Route.Form.ValueChanged".to_string());
        }
        UiEventKind::Toggle => {
            suggestions.push("Route.Toggle.Changed".to_string());
            suggestions.push("Route.Panel.VisibilityChanged".to_string());
        }
        _ => {
            suggestions.push(format!("Route.{}", binding_route_slug(node, binding)));
        }
    }
    dedupe_suggestions(suggestions)
}

fn binding_action_suggestions(node: &UiNodeDefinition, binding: &UiBindingRef) -> Vec<String> {
    let mut suggestions = Vec::new();
    let keywords = binding_keywords(node);
    if binding_event_supports_keyword_shortcuts(binding.event) && is_save_like(&keywords) {
        suggestions.push("EditorAction.SaveProject".to_string());
    }
    match binding.event {
        UiEventKind::Click | UiEventKind::DoubleClick | UiEventKind::Submit => {
            suggestions.push("EditorAction.OpenAssetBrowser".to_string());
            suggestions.push("EditorAction.FocusSelection".to_string());
        }
        UiEventKind::Change => {
            suggestions.push("EditorAction.RefreshPreview".to_string());
            suggestions.push("EditorAction.ApplyInspector".to_string());
        }
        UiEventKind::Toggle => {
            suggestions.push("EditorAction.ToggleVisibility".to_string());
            suggestions.push("EditorAction.ToggleSelectionState".to_string());
        }
        _ => {
            suggestions.push(format!(
                "EditorAction.{}",
                binding_route_slug(node, binding)
            ));
        }
    }
    dedupe_suggestions(suggestions)
}

fn binding_keywords(node: &UiNodeDefinition) -> String {
    let control_id = node.control_id.as_deref().unwrap_or_default();
    let text = node
        .props
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or_default();
    format!("{control_id} {text}").to_ascii_lowercase()
}

fn is_save_like(keywords: &str) -> bool {
    keywords.contains("save")
}

fn binding_event_supports_keyword_shortcuts(event: UiEventKind) -> bool {
    matches!(
        event,
        UiEventKind::Click | UiEventKind::DoubleClick | UiEventKind::Submit
    )
}

fn binding_route_slug(node: &UiNodeDefinition, binding: &UiBindingRef) -> String {
    let base = node
        .control_id
        .as_deref()
        .or_else(|| node.component.as_deref())
        .unwrap_or("Binding");
    format!(
        "{}{}",
        sanitize_identifier(base),
        sanitize_identifier(binding.event.native_name())
    )
}

fn sanitize_identifier(value: &str) -> String {
    let mut normalized = String::new();
    let mut capitalize_next = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalize_next {
                normalized.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                normalized.push(ch);
            }
        } else {
            capitalize_next = true;
        }
    }
    if normalized.is_empty() {
        "Binding".to_string()
    } else {
        normalized
    }
}

fn dedupe_suggestions(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

fn event_source_tag(event: UiEventKind) -> String {
    event
        .native_name()
        .strip_prefix("on")
        .map(|name| format!("ui.{}", name.to_ascii_lowercase()))
        .unwrap_or_else(|| "ui.event".to_string())
}

fn selected_payload_key_for_binding(
    binding: &UiBindingRef,
    current: Option<&str>,
) -> Option<String> {
    let payload = binding_payload_item_entries(binding);
    if payload.is_empty() {
        return None;
    }
    current
        .filter(|key| payload.iter().any(|(path, _)| path == key))
        .map(str::to_string)
        .or_else(|| payload.first().map(|(path, _)| path.clone()))
}

fn binding_payload_root_value(binding: &UiBindingRef) -> Value {
    Value::Table(
        binding
            .action
            .as_ref()
            .map(|action| action.payload.clone().into_iter().collect())
            .unwrap_or_default(),
    )
}

fn resolve_binding_payload_upsert_path(
    binding: &UiBindingRef,
    selected_payload_key: Option<&str>,
    payload_key: &str,
) -> Option<(String, Vec<UiAssetTomlPathSegment>)> {
    let trimmed = payload_key.trim();
    let payload_root = binding_payload_root_value(binding);

    if let Some(selected_payload_key) = selected_payload_key.and_then(normalized_payload_key) {
        if let Some(selected_path) = parse_value_path(&selected_payload_key) {
            if let Some(selected_value) = get_value_at_path(&payload_root, &selected_path) {
                if let Some(resolved) = resolve_relative_binding_payload_upsert_path(
                    selected_value,
                    &selected_path,
                    &selected_payload_key,
                    trimmed,
                ) {
                    return Some(resolved);
                }
            }
        }
    }

    let normalized_payload_key = normalized_payload_key(trimmed)?;
    let path = parse_value_path(&normalized_payload_key)?;
    Some((normalized_payload_key, path))
}

fn resolve_relative_binding_payload_upsert_path(
    selected_value: &Value,
    selected_path: &[UiAssetTomlPathSegment],
    selected_payload_key: &str,
    payload_key: &str,
) -> Option<(String, Vec<UiAssetTomlPathSegment>)> {
    if payload_key_anchors_selected_path(selected_payload_key, payload_key) {
        return None;
    }

    match selected_value {
        Value::Table(_) => {
            let relative_key = normalized_payload_key(payload_key)?;
            let relative_path = parse_value_path(&relative_key)?;
            let mut path = selected_path.to_vec();
            path.extend(relative_path);
            Some((join_payload_key(selected_payload_key, &relative_key), path))
        }
        Value::Array(items) if payload_key.trim().is_empty() => {
            let mut path = selected_path.to_vec();
            path.push(UiAssetTomlPathSegment::Index(items.len()));
            Some((format!("{selected_payload_key}[{}]", items.len()), path))
        }
        Value::Array(_) => {
            let (relative_key, relative_path) =
                parse_relative_collection_payload_path(payload_key)?;
            let mut path = selected_path.to_vec();
            path.extend(relative_path);
            Some((join_payload_key(selected_payload_key, &relative_key), path))
        }
        _ => None,
    }
}

fn payload_key_anchors_selected_path(selected_payload_key: &str, payload_key: &str) -> bool {
    let trimmed = payload_key.trim();
    trimmed == selected_payload_key
        || trimmed
            .strip_prefix(selected_payload_key)
            .is_some_and(|rest| rest.starts_with('.') || rest.starts_with('['))
}

fn join_payload_key(selected_payload_key: &str, relative_key: &str) -> String {
    if relative_key.starts_with('[') {
        format!("{selected_payload_key}{relative_key}")
    } else {
        format!("{selected_payload_key}.{relative_key}")
    }
}

fn parse_relative_collection_payload_path(
    payload_key: &str,
) -> Option<(String, Vec<UiAssetTomlPathSegment>)> {
    let trimmed = payload_key.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('[') {
        return Some((trimmed.to_string(), parse_value_path(trimmed)?));
    }

    let digit_end = trimmed
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .last()
        .map(|(index, ch)| index + ch.len_utf8())?;
    let index = trimmed[..digit_end].parse::<usize>().ok()?;
    let remainder = &trimmed[digit_end..];
    let normalized = if remainder.is_empty() {
        format!("[{index}]")
    } else if remainder.starts_with('.') || remainder.starts_with('[') {
        format!("[{index}]{}", remainder)
    } else {
        return None;
    };
    Some((normalized.clone(), parse_value_path(&normalized)?))
}

fn collect_binding_payload_item_entries(
    value: &Value,
    prefix: Option<&str>,
    entries: &mut Vec<(String, Value)>,
) {
    if let Some(prefix) = prefix {
        entries.push((prefix.to_string(), value.clone()));
    }
    match value {
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let path = match prefix {
                    Some(prefix) => format!("{prefix}[{index}]"),
                    None => format!("[{index}]"),
                };
                collect_binding_payload_item_entries(item, Some(path.as_str()), entries);
            }
        }
        Value::Table(table) => {
            let mut keys = table.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                let Some(item) = table.get(&key) else {
                    continue;
                };
                let path = match prefix {
                    Some(prefix) => format!("{prefix}.{key}"),
                    None => key.clone(),
                };
                collect_binding_payload_item_entries(item, Some(path.as_str()), entries);
            }
        }
        _ => {}
    }
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
