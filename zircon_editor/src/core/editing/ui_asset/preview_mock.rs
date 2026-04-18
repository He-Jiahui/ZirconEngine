use std::collections::BTreeMap;

use toml::Value;
use crate::ui::UiDesignerSelectionModel;
use zircon_ui::UiAssetDocument;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiAssetPreviewMockState {
    overrides: BTreeMap<String, BTreeMap<String, Value>>,
    selected_property: Option<String>,
    selected_subject_node_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetPreviewMockFields {
    pub items: Vec<String>,
    pub selected_index: i32,
    pub property: String,
    pub kind: String,
    pub value: String,
    pub state_graph_items: Vec<String>,
    pub can_edit: bool,
    pub can_clear: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiAssetPreviewMockKind {
    Text,
    Bool,
    Number,
    Enum,
    Resource,
    Collection,
    Object,
    Expression,
}

impl UiAssetPreviewMockKind {
    fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Bool => "Bool",
            Self::Number => "Number",
            Self::Enum => "Enum",
            Self::Resource => "Resource",
            Self::Collection => "Collection",
            Self::Object => "Object",
            Self::Expression => "Expression",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct UiAssetPreviewMockEntry {
    key: String,
    display_key: String,
    kind: UiAssetPreviewMockKind,
    effective_value: Value,
    overridden: bool,
}

pub(crate) fn build_preview_mock_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &UiAssetPreviewMockState,
) -> UiAssetPreviewMockFields {
    let entries = preview_mock_entries(document, selection, state);
    let Some(selected_index) = selected_entry_index(&entries, state.selected_property.as_deref())
    else {
        return UiAssetPreviewMockFields::default();
    };
    let Some(selected) = entries.get(selected_index) else {
        return UiAssetPreviewMockFields::default();
    };
    UiAssetPreviewMockFields {
        items: entries
            .iter()
            .map(|entry| {
                format!(
                    "{} [{}] = {}",
                    entry.display_key,
                    entry.kind.label(),
                    preview_mock_literal(&entry.effective_value)
                )
            })
            .collect(),
        selected_index: selected_index as i32,
        property: selected.display_key.clone(),
        kind: selected.kind.label().to_string(),
        value: preview_mock_literal(&selected.effective_value),
        state_graph_items: build_preview_state_graph_items(document, state),
        can_edit: true,
        can_clear: selected.overridden,
    }
}

pub(crate) fn build_preview_state_graph_items(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
) -> Vec<String> {
    let mut items = state
        .overrides
        .iter()
        .filter_map(|(node_id, props)| document.nodes.get(node_id).map(|node| (node_id, node, props)))
        .flat_map(|(node_id, node, props)| {
            props.iter().map(move |(key, value)| {
                format!(
                    "{} = {}",
                    preview_mock_display_key(node, node_id, key, true),
                    preview_mock_literal(value)
                )
            })
        })
        .collect::<Vec<_>>();
    items.sort();
    items
}

pub(crate) fn reconcile_preview_mock_state(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
) {
    state.overrides.retain(|node_id, values| {
        let keep_node = document.nodes.get(node_id).is_some();
        if !keep_node {
            return false;
        }
        values.retain(|key, _| property_kind(document, node_id, key).is_some());
        !values.is_empty()
    });
    state.selected_subject_node_id = state
        .selected_subject_node_id
        .take()
        .filter(|node_id| {
            document.nodes.contains_key(node_id)
                && selection.primary_node_id.as_deref() != Some(node_id.as_str())
        });

    let entries = preview_mock_entries(document, selection, state);
    state.selected_property = selected_entry_index(&entries, state.selected_property.as_deref())
        .and_then(|index| entries.get(index).map(|entry| entry.key.clone()));
}

pub(crate) fn select_preview_mock_subject_node(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    node_id: &str,
) -> bool {
    if !document.nodes.contains_key(node_id) {
        return false;
    }

    let next_subject = (selection.primary_node_id.as_deref() != Some(node_id))
        .then(|| node_id.to_string());
    let changed = state.selected_subject_node_id != next_subject;
    state.selected_subject_node_id = next_subject;
    let selected_property = state.selected_property.clone();
    reconcile_preview_mock_state(document, selection, state);
    changed || state.selected_property != selected_property
}

pub(crate) fn select_preview_mock_property(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    index: usize,
) -> Option<bool> {
    let entries = preview_mock_entries(document, selection, state);
    let selected = entries.get(index)?;
    let changed = state.selected_property.as_deref() != Some(selected.key.as_str());
    state.selected_property = Some(selected.key.clone());
    Some(changed)
}

pub(crate) fn set_selected_preview_mock_value(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    value: &str,
) -> Result<bool, String> {
    let Some(node_id) = preview_mock_subject_node_id(document, selection, state)
        .map(str::to_string)
    else {
        return Ok(false);
    };
    let entries = preview_mock_entries(document, selection, state);
    let Some(selected_index) = selected_entry_index(&entries, state.selected_property.as_deref())
    else {
        return Ok(false);
    };
    let Some(entry) = entries.get(selected_index) else {
        return Ok(false);
    };
    let next_value = parse_preview_mock_value(entry.kind, value).ok_or_else(|| {
        format!(
            "preview mock property {} expects {}",
            entry.display_key,
            entry.kind.label()
        )
    })?;
    let base_value = document
        .nodes
        .get(&node_id)
        .and_then(|node| node.props.get(&entry.key));
    if base_value == Some(&next_value) {
        return Ok(clear_selected_preview_mock_value(
            document, selection, state,
        ));
    }
    let overrides = state.overrides.entry(node_id).or_default();
    if overrides.get(&entry.key) == Some(&next_value) {
        return Ok(false);
    }
    let _ = overrides.insert(entry.key.clone(), next_value);
    state.selected_property = Some(entry.key.clone());
    Ok(true)
}

pub(crate) fn clear_selected_preview_mock_value(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
) -> bool {
    let Some(node_id) = preview_mock_subject_node_id(document, selection, state)
        .map(str::to_string)
    else {
        return false;
    };
    let entries = preview_mock_entries(document, selection, state);
    let Some(selected_index) = selected_entry_index(&entries, state.selected_property.as_deref())
    else {
        return false;
    };
    let Some(entry) = entries.get(selected_index) else {
        return false;
    };
    let Some(overrides) = state.overrides.get_mut(&node_id) else {
        return false;
    };
    let removed = overrides.remove(&entry.key).is_some();
    if overrides.is_empty() {
        let _ = state.overrides.remove(&node_id);
    }
    removed
}

pub(crate) fn apply_preview_mock_overrides(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
) -> UiAssetDocument {
    if state.overrides.is_empty() {
        return document.clone();
    }
    let mut preview_document = document.clone();
    for (node_id, props) in &state.overrides {
        let Some(node) = preview_document.nodes.get_mut(node_id) else {
            continue;
        };
        for (key, value) in props {
            let _ = node.props.insert(key.clone(), value.clone());
        }
    }
    preview_document
}

fn preview_mock_entries(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &UiAssetPreviewMockState,
) -> Vec<UiAssetPreviewMockEntry> {
    let Some(node_id) = preview_mock_subject_node_id(document, selection, state) else {
        return Vec::new();
    };
    let Some(node) = document.nodes.get(node_id) else {
        return Vec::new();
    };
    let overrides = state.overrides.get(node_id);
    let qualify_display = selection.primary_node_id.as_deref() != Some(node_id);
    let mut entries = node
        .props
        .iter()
        .filter_map(|(key, value)| {
            let kind = preview_mock_kind_for_property(key, value)?;
            let effective_value = overrides
                .and_then(|props| props.get(key))
                .cloned()
                .unwrap_or_else(|| value.clone());
            Some(UiAssetPreviewMockEntry {
                key: key.clone(),
                display_key: preview_mock_display_key(
                    node,
                    node_id,
                    key,
                    qualify_display,
                ),
                kind,
                effective_value,
                overridden: overrides.and_then(|props| props.get(key)).is_some(),
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| preview_mock_sort_key(&entry.key, entry.kind));
    entries
}

fn preview_mock_subject_node_id<'a>(
    document: &'a UiAssetDocument,
    selection: &'a UiDesignerSelectionModel,
    state: &'a UiAssetPreviewMockState,
) -> Option<&'a str> {
    state
        .selected_subject_node_id
        .as_deref()
        .filter(|node_id| document.nodes.contains_key(*node_id))
        .or_else(|| {
            selection
                .primary_node_id
                .as_deref()
                .filter(|node_id| document.nodes.contains_key(*node_id))
        })
}

fn selected_entry_index(
    entries: &[UiAssetPreviewMockEntry],
    selected_property: Option<&str>,
) -> Option<usize> {
    if entries.is_empty() {
        return None;
    }
    selected_property
        .and_then(|selected| entries.iter().position(|entry| entry.key == selected))
        .or(Some(0))
}

fn property_kind(
    document: &UiAssetDocument,
    node_id: &str,
    key: &str,
) -> Option<UiAssetPreviewMockKind> {
    let value = document.nodes.get(node_id)?.props.get(key)?;
    preview_mock_kind_for_property(key, value)
}

fn preview_mock_kind_for_property(key: &str, value: &Value) -> Option<UiAssetPreviewMockKind> {
    match value {
        Value::Boolean(_) => Some(UiAssetPreviewMockKind::Bool),
        Value::Integer(_) | Value::Float(_) => Some(UiAssetPreviewMockKind::Number),
        Value::String(text) if is_resource_reference(text) => {
            Some(UiAssetPreviewMockKind::Resource)
        }
        Value::String(text) if expression_like_property(key, text) => {
            Some(UiAssetPreviewMockKind::Expression)
        }
        Value::String(_) if enum_like_property(key) => Some(UiAssetPreviewMockKind::Enum),
        Value::String(_) => Some(UiAssetPreviewMockKind::Text),
        Value::Array(_) => Some(UiAssetPreviewMockKind::Collection),
        Value::Table(_) => Some(UiAssetPreviewMockKind::Object),
        _ => None,
    }
}

fn enum_like_property(key: &str) -> bool {
    matches!(
        key,
        "kind"
            | "mode"
            | "state"
            | "axis"
            | "direction"
            | "orientation"
            | "alignment"
            | "scrollbar_visibility"
            | "variant"
    )
}

fn is_resource_reference(value: &str) -> bool {
    value.starts_with("asset://") || value.starts_with("res://")
}

fn expression_like_property(key: &str, value: &str) -> bool {
    key.ends_with("_expr")
        || key.contains("expression")
        || value.trim_start().starts_with('=')
}

fn parse_preview_mock_value(kind: UiAssetPreviewMockKind, value: &str) -> Option<Value> {
    match kind {
        UiAssetPreviewMockKind::Bool => parse_bool(value).map(Value::Boolean),
        UiAssetPreviewMockKind::Number => parse_toml_inline_value(value).and_then(|parsed| {
            matches!(parsed, Value::Integer(_) | Value::Float(_)).then_some(parsed)
        }),
        UiAssetPreviewMockKind::Text
        | UiAssetPreviewMockKind::Enum
        | UiAssetPreviewMockKind::Resource
        | UiAssetPreviewMockKind::Expression => Some(Value::String(value.to_string())),
        UiAssetPreviewMockKind::Collection => {
            parse_toml_inline_value(value).and_then(|parsed| matches!(parsed, Value::Array(_)).then_some(parsed))
        }
        UiAssetPreviewMockKind::Object => {
            parse_toml_inline_value(value).and_then(|parsed| matches!(parsed, Value::Table(_)).then_some(parsed))
        }
    }
}

fn parse_toml_inline_value(value: &str) -> Option<Value> {
    let table = format!("value = {value}").parse::<toml::Table>().ok()?;
    table.get("value").cloned()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn preview_mock_literal(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Boolean(value) => value.to_string(),
        _ => value.to_string(),
    }
}

fn preview_mock_display_key(
    node: &zircon_ui::UiNodeDefinition,
    node_id: &str,
    key: &str,
    qualify: bool,
) -> String {
    if !qualify {
        return key.to_string();
    }
    let subject = node.control_id.as_deref().unwrap_or(node_id);
    format!("{subject}.{key}")
}

fn preview_mock_sort_key(key: &str, kind: UiAssetPreviewMockKind) -> (u8, String) {
    if key == "text" {
        return (0, key.to_string());
    }
    let priority = match kind {
        UiAssetPreviewMockKind::Bool => 1,
        UiAssetPreviewMockKind::Number => 2,
        UiAssetPreviewMockKind::Enum => 3,
        UiAssetPreviewMockKind::Resource => 4,
        UiAssetPreviewMockKind::Collection => 5,
        UiAssetPreviewMockKind::Object => 6,
        UiAssetPreviewMockKind::Expression => 7,
        UiAssetPreviewMockKind::Text => 8,
    };
    (priority, key.to_string())
}
