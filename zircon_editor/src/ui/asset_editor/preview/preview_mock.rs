use std::collections::BTreeMap;

use crate::ui::asset_editor::value_path::{
    parse_value_path, set_value_at_path, UiAssetTomlPathSegment,
};
use crate::ui::asset_editor::UiDesignerSelectionModel;
use toml::Value;
use zircon_runtime::ui::template::{UiAssetDocument, UiNodeDefinition};

#[path = "mock_expression.rs"]
mod mock_expression;
#[path = "mock_suggestions.rs"]
mod mock_suggestions;
#[path = "mock_value_resolution.rs"]
mod mock_value_resolution;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiAssetPreviewMockState {
    overrides: BTreeMap<String, BTreeMap<String, Value>>,
    selected_property: Option<String>,
    selected_subject_node_id: Option<String>,
    selected_nested_key: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetPreviewMockFields {
    pub subject_items: Vec<String>,
    pub subject_selected_index: i32,
    pub subject_node_id: String,
    pub items: Vec<String>,
    pub selected_index: i32,
    pub property: String,
    pub kind: String,
    pub value: String,
    pub expression_result: String,
    pub nested_items: Vec<String>,
    pub nested_selected_index: i32,
    pub nested_key: String,
    pub nested_kind: String,
    pub nested_value: String,
    pub suggestion_items: Vec<String>,
    pub schema_items: Vec<String>,
    pub state_graph_items: Vec<String>,
    pub can_edit: bool,
    pub can_clear: bool,
    pub nested_can_edit: bool,
    pub nested_can_add: bool,
    pub nested_can_delete: bool,
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

    fn supports_nested_entries(self) -> bool {
        matches!(self, Self::Collection | Self::Object)
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

#[derive(Clone, Debug, PartialEq)]
struct UiAssetPreviewMockNestedEntry {
    key: String,
    display_key: String,
    kind: UiAssetPreviewMockKind,
    value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiAssetPreviewMockSubjectEntry {
    node_id: String,
    label: String,
}

pub(crate) fn build_preview_mock_fields(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &UiAssetPreviewMockState,
) -> UiAssetPreviewMockFields {
    let subject_entries = preview_mock_subject_entries(document);
    let selected_subject_node_id =
        resolved_preview_mock_subject_node_id(document, selection, state).map(str::to_string);
    let entries = preview_mock_entries(document, selection, state);
    let Some(selected_index) = selected_entry_index(&entries, state.selected_property.as_deref())
    else {
        return UiAssetPreviewMockFields {
            subject_items: subject_entries
                .iter()
                .map(|entry| entry.label.clone())
                .collect(),
            subject_selected_index: selected_subject_node_id
                .as_deref()
                .and_then(|node_id| {
                    subject_entries
                        .iter()
                        .position(|entry| entry.node_id.as_str() == node_id)
                })
                .map(|index| index as i32)
                .unwrap_or(-1),
            subject_node_id: selected_subject_node_id.unwrap_or_default(),
            ..UiAssetPreviewMockFields::default()
        };
    };
    let Some(selected) = entries.get(selected_index) else {
        return UiAssetPreviewMockFields::default();
    };
    let nested_entries = preview_mock_nested_entries(&selected.effective_value);
    let selected_nested_index =
        selected_nested_entry_index(&nested_entries, state.selected_nested_key.as_deref());
    let selected_nested = selected_nested_index.and_then(|index| nested_entries.get(index));
    UiAssetPreviewMockFields {
        subject_items: subject_entries
            .iter()
            .map(|entry| entry.label.clone())
            .collect(),
        subject_selected_index: selected_subject_node_id
            .as_deref()
            .and_then(|node_id| {
                subject_entries
                    .iter()
                    .position(|entry| entry.node_id.as_str() == node_id)
            })
            .map(|index| index as i32)
            .unwrap_or(-1),
        subject_node_id: selected_subject_node_id.clone().unwrap_or_default(),
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
        expression_result: selected_subject_node_id
            .as_deref()
            .and_then(|node_id| {
                evaluate_preview_mock_expression(
                    document,
                    state,
                    node_id,
                    &selected.effective_value,
                )
            })
            .unwrap_or_default(),
        nested_items: nested_entries
            .iter()
            .map(|entry| {
                let display_key = if selected.display_key == selected.key {
                    entry.display_key.clone()
                } else {
                    qualified_preview_mock_nested_display_key(&selected.display_key, &entry.key)
                };
                format!(
                    "{} [{}] = {}",
                    display_key,
                    entry.kind.label(),
                    preview_mock_literal(&entry.value)
                )
            })
            .collect(),
        nested_selected_index: selected_nested_index
            .map(|index| index as i32)
            .unwrap_or(-1),
        nested_key: selected_nested
            .map(|entry| entry.key.clone())
            .unwrap_or_default(),
        nested_kind: selected_nested
            .map(|entry| entry.kind.label().to_string())
            .unwrap_or_default(),
        nested_value: selected_nested
            .map(|entry| preview_mock_literal(&entry.value))
            .unwrap_or_default(),
        suggestion_items: mock_suggestions::preview_mock_suggestion_items(
            selected,
            state.selected_nested_key.as_deref(),
        ),
        schema_items: selected_subject_node_id
            .as_deref()
            .and_then(|node_id| {
                document.node(node_id).map(|node| {
                    mock_suggestions::build_preview_mock_schema_items(node, node_id, selected)
                })
            })
            .unwrap_or_default(),
        state_graph_items: build_preview_state_graph_items(document, state),
        can_edit: true,
        can_clear: selected.overridden,
        nested_can_edit: selected_nested.is_some(),
        nested_can_add: selected.kind.supports_nested_entries(),
        nested_can_delete: selected_nested.is_some(),
    }
}

pub(crate) fn build_preview_state_graph_items(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
) -> Vec<String> {
    let mut items = state
        .overrides
        .iter()
        .filter_map(|(node_id, props)| document.node(node_id).map(|node| (node_id, node, props)))
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
    items.extend(preview_mock_expression_graph_items(document, state));
    items.sort();
    items
}

pub(crate) fn resolve_preview_mock_value_preview(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    value: &Value,
) -> Option<Value> {
    mock_value_resolution::resolve_preview_mock_value_preview(
        document,
        state,
        current_node_id,
        value,
    )
}

pub(crate) fn format_preview_mock_inline_value(value: &Value) -> String {
    preview_mock_inline_literal(value)
}

fn preview_mock_expression_graph_items(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
) -> Vec<String> {
    let mut items = Vec::new();
    for node in document.iter_nodes() {
        let node_id = node.node_id.as_str();
        for (key, value) in &node.props {
            let source_key = preview_mock_display_key(node, node_id, key, true);
            for (target_node_id, target_path, target_value) in
                mock_value_resolution::collect_preview_mock_expression_dependencies(
                    document, state, node_id, value,
                )
            {
                let Some(target_node) = document.node(&target_node_id) else {
                    continue;
                };
                let target_key =
                    preview_mock_display_key(target_node, &target_node_id, &target_path, true);
                items.push(format!(
                    "{source_key} -> {target_key} = {}",
                    preview_mock_inline_literal(&target_value)
                ));
            }
        }
    }
    items.extend(mock_value_resolution::build_preview_binding_graph_items(
        document, state,
    ));
    items
}

pub(crate) fn reconcile_preview_mock_state(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
) {
    state.overrides.retain(|node_id, values| {
        let keep_node = document.node(node_id).is_some();
        if !keep_node {
            return false;
        }
        values.retain(|key, _| property_kind(document, node_id, key).is_some());
        !values.is_empty()
    });
    state.selected_subject_node_id = state.selected_subject_node_id.take().filter(|node_id| {
        document.contains_node(node_id) && preview_mock_node_has_entries(document, node_id)
    });

    let entries = preview_mock_entries(document, selection, state);
    state.selected_property = selected_entry_index(&entries, state.selected_property.as_deref())
        .and_then(|index| entries.get(index).map(|entry| entry.key.clone()));
    let nested_entries = selected_preview_mock_entry(document, selection, state)
        .map(|(_, entry)| preview_mock_nested_entries(&entry.effective_value))
        .unwrap_or_default();
    state.selected_nested_key =
        selected_nested_entry_index(&nested_entries, state.selected_nested_key.as_deref())
            .and_then(|index| nested_entries.get(index).map(|entry| entry.key.clone()));
}

pub(crate) fn select_preview_mock_subject_node(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    node_id: &str,
) -> bool {
    if !document.contains_node(node_id) || !preview_mock_node_has_entries(document, node_id) {
        return false;
    }
    state.overrides.retain(|override_node_id, values| {
        let keep_node = document.node(override_node_id).is_some();
        if !keep_node {
            return false;
        }
        values.retain(|key, _| property_kind(document, override_node_id, key).is_some());
        !values.is_empty()
    });
    let next_subject = Some(node_id.to_string());
    let changed = state.selected_subject_node_id != next_subject;
    state.selected_subject_node_id = next_subject;
    let selected_property = state.selected_property.clone();
    let selected_nested_key = state.selected_nested_key.clone();
    let _ = selection;
    state.selected_property = None;
    state.selected_nested_key = None;
    changed || selected_property.is_some() || selected_nested_key.is_some()
}

pub(crate) fn select_preview_mock_subject(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    index: usize,
) -> Option<bool> {
    let subject = preview_mock_subject_entries(document)
        .get(index)?
        .node_id
        .clone();
    Some(select_preview_mock_subject_node(
        document, selection, state, &subject,
    ))
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
    let nested_entries = preview_mock_nested_entries(&selected.effective_value);
    state.selected_nested_key =
        selected_nested_entry_index(&nested_entries, None).and_then(|nested_index| {
            nested_entries
                .get(nested_index)
                .map(|entry| entry.key.clone())
        });
    Some(changed)
}

pub(crate) fn select_preview_mock_nested_entry(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    index: usize,
) -> Option<bool> {
    let (_, entry) = selected_preview_mock_entry(document, selection, state)?;
    let nested_entries = preview_mock_nested_entries(&entry.effective_value);
    let selected = nested_entries.get(index)?;
    let changed = state.selected_nested_key.as_deref() != Some(selected.key.as_str());
    state.selected_nested_key = Some(selected.key.clone());
    Some(changed)
}

pub(crate) fn set_selected_preview_mock_value(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    value: &str,
) -> Result<bool, String> {
    let Some((node_id, entry)) = selected_preview_mock_entry(document, selection, state) else {
        return Ok(false);
    };
    let next_value = parse_preview_mock_value(entry.kind, value).ok_or_else(|| {
        format!(
            "preview mock property {} expects {}",
            entry.display_key,
            entry.kind.label()
        )
    })?;
    Ok(set_preview_mock_override_value(
        document, selection, state, &node_id, &entry.key, next_value,
    ))
}

pub(crate) fn set_selected_preview_mock_nested_value(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    value: &str,
) -> Result<bool, String> {
    let Some((node_id, entry)) = selected_preview_mock_entry(document, selection, state) else {
        return Ok(false);
    };
    let Some(nested_entry) = selected_preview_mock_nested_entry_state(document, selection, state)
    else {
        return Ok(false);
    };
    let next_nested = parse_preview_mock_value(nested_entry.kind, value).ok_or_else(|| {
        format!(
            "preview mock nested property {} expects {}",
            nested_entry.display_key,
            nested_entry.kind.label()
        )
    })?;
    let mut next_value = entry.effective_value.clone();
    mutate_preview_mock_nested_value(&mut next_value, &nested_entry.key, Some(next_nested))?;
    Ok(set_preview_mock_override_value(
        document, selection, state, &node_id, &entry.key, next_value,
    ))
}

pub(crate) fn upsert_selected_preview_mock_nested_entry(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    key: &str,
    value_literal: &str,
) -> Result<bool, String> {
    let Some((node_id, entry)) = selected_preview_mock_entry(document, selection, state) else {
        return Ok(false);
    };
    if !entry.kind.supports_nested_entries() {
        return Ok(false);
    }
    let mut next_value = entry.effective_value.clone();
    let normalized_key = normalize_nested_entry_key(&entry.effective_value, key)?;
    let next_nested_value = preview_mock_nested_entries(&entry.effective_value)
        .into_iter()
        .find(|existing| existing.key == normalized_key)
        .and_then(|existing| parse_preview_mock_value(existing.kind, value_literal))
        .unwrap_or_else(|| parse_preview_mock_loose_value(value_literal));
    mutate_preview_mock_nested_value(&mut next_value, &normalized_key, Some(next_nested_value))?;
    Ok(set_preview_mock_override_value(
        document, selection, state, &node_id, &entry.key, next_value,
    ))
}

pub(crate) fn apply_selected_preview_mock_suggestion(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    suggestion_index: usize,
) -> Result<Option<String>, String> {
    let Some((node_id, entry)) = selected_preview_mock_entry(document, selection, state) else {
        return Ok(None);
    };
    let suggestions =
        mock_suggestions::preview_mock_suggestions(&entry, state.selected_nested_key.as_deref());
    let Some(suggestion) = suggestions.get(suggestion_index).cloned() else {
        return Ok(None);
    };

    let mut next_value = entry.effective_value.clone();
    mutate_preview_mock_nested_value(
        &mut next_value,
        &suggestion.resolved_key,
        Some(suggestion.value),
    )?;
    let _ = set_preview_mock_override_value(
        document, selection, state, &node_id, &entry.key, next_value,
    );
    state.selected_nested_key = Some(suggestion.resolved_key.clone());
    reconcile_preview_mock_state(document, selection, state);
    Ok(Some(suggestion.resolved_key))
}

pub(crate) fn delete_selected_preview_mock_nested_entry(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
) -> Result<bool, String> {
    let Some((node_id, entry)) = selected_preview_mock_entry(document, selection, state) else {
        return Ok(false);
    };
    let Some(nested_entry) = selected_preview_mock_nested_entry_state(document, selection, state)
    else {
        return Ok(false);
    };
    let mut next_value = entry.effective_value.clone();
    mutate_preview_mock_nested_value(&mut next_value, &nested_entry.key, None)?;
    Ok(set_preview_mock_override_value(
        document, selection, state, &node_id, &entry.key, next_value,
    ))
}

pub(crate) fn clear_selected_preview_mock_value(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
) -> bool {
    let Some(node_id) =
        preview_mock_subject_node_id(document, selection, state).map(str::to_string)
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
    if removed {
        reconcile_preview_mock_state(document, selection, state);
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
        let Some(node) = preview_document.node_mut(node_id) else {
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
    let Some(node_id) = resolved_preview_mock_subject_node_id(document, selection, state) else {
        return Vec::new();
    };
    let Some(node) = document.node(node_id) else {
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
                display_key: preview_mock_display_key(node, node_id, key, qualify_display),
                kind,
                effective_value,
                overridden: overrides.and_then(|props| props.get(key)).is_some(),
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| preview_mock_sort_key(&entry.key, entry.kind));
    entries
}

fn preview_mock_nested_entries(value: &Value) -> Vec<UiAssetPreviewMockNestedEntry> {
    let mut entries = Vec::new();
    collect_preview_mock_nested_entries(value, None, &mut entries);
    entries.sort_by_key(|entry| preview_mock_sort_key(&entry.key, entry.kind));
    entries
}

fn preview_mock_subject_node_id<'a>(
    document: &'a UiAssetDocument,
    selection: &'a UiDesignerSelectionModel,
    state: &'a UiAssetPreviewMockState,
) -> Option<&'a str> {
    resolved_preview_mock_subject_node_id(document, selection, state)
}

fn resolved_preview_mock_subject_node_id<'a>(
    document: &'a UiAssetDocument,
    selection: &'a UiDesignerSelectionModel,
    state: &'a UiAssetPreviewMockState,
) -> Option<&'a str> {
    state
        .selected_subject_node_id
        .as_deref()
        .filter(|node_id| preview_mock_node_has_entries(document, node_id))
        .or_else(|| {
            selection
                .primary_node_id
                .as_deref()
                .filter(|node_id| preview_mock_node_has_entries(document, node_id))
        })
        .or_else(|| {
            document
                .iter_nodes()
                .filter(|node| preview_mock_node_has_entries(document, &node.node_id))
                .min_by(|left, right| {
                    preview_mock_subject_sort_key(left).cmp(&preview_mock_subject_sort_key(right))
                })
                .map(|node| node.node_id.as_str())
        })
}

fn selected_preview_mock_entry(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &UiAssetPreviewMockState,
) -> Option<(String, UiAssetPreviewMockEntry)> {
    let node_id = preview_mock_subject_node_id(document, selection, state)?.to_string();
    let entries = preview_mock_entries(document, selection, state);
    let selected_index = selected_entry_index(&entries, state.selected_property.as_deref())?;
    Some((node_id, entries.get(selected_index)?.clone()))
}

fn selected_preview_mock_nested_entry_state(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &UiAssetPreviewMockState,
) -> Option<UiAssetPreviewMockNestedEntry> {
    let (_, entry) = selected_preview_mock_entry(document, selection, state)?;
    let nested_entries = preview_mock_nested_entries(&entry.effective_value);
    let selected_index =
        selected_nested_entry_index(&nested_entries, state.selected_nested_key.as_deref())?;
    nested_entries.get(selected_index).cloned()
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

fn selected_nested_entry_index(
    entries: &[UiAssetPreviewMockNestedEntry],
    selected_nested_key: Option<&str>,
) -> Option<usize> {
    if entries.is_empty() {
        return None;
    }
    selected_nested_key
        .and_then(|selected| entries.iter().position(|entry| entry.key == selected))
        .or(Some(0))
}

fn property_kind(
    document: &UiAssetDocument,
    node_id: &str,
    key: &str,
) -> Option<UiAssetPreviewMockKind> {
    let value = document.node(node_id)?.props.get(key)?;
    preview_mock_kind_for_property(key, value)
}

fn preview_mock_subject_entries(document: &UiAssetDocument) -> Vec<UiAssetPreviewMockSubjectEntry> {
    let mut entries = document
        .iter_nodes()
        .filter(|node| preview_mock_node_has_entries(document, &node.node_id))
        .map(|node| UiAssetPreviewMockSubjectEntry {
            node_id: node.node_id.clone(),
            label: preview_mock_subject_label(node),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.label.cmp(&right.label));
    entries
}

fn preview_mock_subject_label(node: &UiNodeDefinition) -> String {
    format!(
        "{} • {}",
        node.control_id.as_deref().unwrap_or(node.node_id.as_str()),
        node.node_id
    )
}

fn preview_mock_subject_sort_key(node: &UiNodeDefinition) -> (&str, &str) {
    (
        node.control_id.as_deref().unwrap_or(node.node_id.as_str()),
        node.node_id.as_str(),
    )
}

fn preview_mock_node_has_entries(document: &UiAssetDocument, node_id: &str) -> bool {
    document
        .node(node_id)
        .map(|node| {
            node.props
                .iter()
                .any(|(key, value)| preview_mock_kind_for_property(key, value).is_some())
        })
        .unwrap_or(false)
}

fn evaluate_preview_mock_expression(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    value: &Value,
) -> Option<String> {
    let expression = match value {
        Value::String(text) if text.trim_start().starts_with('=') => text.trim(),
        _ => return None,
    };
    if expression.trim_start_matches('=').trim().is_empty() {
        return Some(String::new());
    }
    resolve_preview_mock_value_preview(document, state, current_node_id, value)
        .map(|value| preview_mock_literal(&value))
}

fn resolve_preview_mock_expression<'a>(
    document: &'a UiAssetDocument,
    state: &'a UiAssetPreviewMockState,
    current_node_id: &'a str,
    value: &'a Value,
) -> Option<(&'a str, String, &'a Value)> {
    let parsed = mock_expression::parse_preview_mock_expression(value)?;
    let target_node_id =
        resolve_preview_mock_expression_node(document, current_node_id, &parsed.node_reference)?;
    let mut current_value =
        preview_mock_property_value(document, state, target_node_id, &parsed.property)?;
    let mut target_path = parsed.property.clone();
    for segment in &parsed.nested_segments {
        current_value = preview_mock_nested_value(current_value, segment)?;
        mock_expression::append_expression_path_segment(&mut target_path, segment);
    }
    Some((target_node_id, target_path, current_value))
}

fn resolve_preview_mock_expression_node<'a>(
    document: &'a UiAssetDocument,
    current_node_id: &str,
    reference: &str,
) -> Option<&'a str> {
    if reference == "self" {
        return document
            .node(current_node_id)
            .map(|node| node.node_id.as_str());
    }
    if let Some(node) = document.node(reference) {
        return Some(node.node_id.as_str());
    }
    document
        .iter_nodes()
        .find(|node| node.control_id.as_deref() == Some(reference))
        .map(|node| node.node_id.as_str())
}

fn preview_mock_property_value<'a>(
    document: &'a UiAssetDocument,
    state: &'a UiAssetPreviewMockState,
    node_id: &str,
    key: &str,
) -> Option<&'a Value> {
    state
        .overrides
        .get(node_id)
        .and_then(|props| props.get(key))
        .or_else(|| document.node(node_id)?.props.get(key))
}

fn preview_mock_nested_value<'a>(value: &'a Value, segment: &str) -> Option<&'a Value> {
    match value {
        Value::Array(items) => items.get(segment.parse::<usize>().ok()?),
        Value::Table(table) => table.get(segment),
        _ => None,
    }
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

fn preview_mock_kind_for_nested_value(value: &Value) -> Option<UiAssetPreviewMockKind> {
    match value {
        Value::Boolean(_) => Some(UiAssetPreviewMockKind::Bool),
        Value::Integer(_) | Value::Float(_) => Some(UiAssetPreviewMockKind::Number),
        Value::String(text) if is_resource_reference(text) => {
            Some(UiAssetPreviewMockKind::Resource)
        }
        Value::String(text) if text.trim_start().starts_with('=') => {
            Some(UiAssetPreviewMockKind::Expression)
        }
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
    key.ends_with("_expr") || key.contains("expression") || value.trim_start().starts_with('=')
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
        UiAssetPreviewMockKind::Collection => parse_toml_inline_value(value)
            .and_then(|parsed| matches!(parsed, Value::Array(_)).then_some(parsed)),
        UiAssetPreviewMockKind::Object => parse_toml_inline_value(value)
            .and_then(|parsed| matches!(parsed, Value::Table(_)).then_some(parsed)),
    }
}

fn parse_preview_mock_loose_value(value: &str) -> Value {
    parse_toml_inline_value(value).unwrap_or_else(|| Value::String(value.to_string()))
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
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(preview_mock_inline_literal)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Table(table) => {
            let mut entries = table.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            format!(
                "{{ {} }}",
                entries
                    .into_iter()
                    .map(|(key, value)| format!("{key} = {}", preview_mock_inline_literal(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        _ => value.to_string(),
    }
}

fn preview_mock_inline_literal(value: &Value) -> String {
    match value {
        Value::String(text) => Value::String(text.clone()).to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(preview_mock_inline_literal)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Table(table) => {
            let mut entries = table.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            format!(
                "{{ {} }}",
                entries
                    .into_iter()
                    .map(|(key, value)| format!("{key} = {}", preview_mock_inline_literal(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        _ => value.to_string(),
    }
}

fn preview_mock_display_key(
    node: &zircon_runtime::ui::template::UiNodeDefinition,
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

fn qualified_preview_mock_nested_display_key(base: &str, nested_key: &str) -> String {
    if nested_key.is_empty() {
        return base.to_string();
    }

    let mut relative = nested_key.to_string();
    if relative
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
        let digit_end = relative
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_digit())
            .last()
            .map(|(index, ch)| index + ch.len_utf8())
            .unwrap_or(0);
        let rest = relative.split_off(digit_end);
        relative = format!("[{}]{rest}", &nested_key[..digit_end]);
    } else if !relative.starts_with('[') && !relative.starts_with('.') {
        relative.insert(0, '.');
    }

    format!("{base}{relative}")
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

fn set_preview_mock_override_value(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &mut UiAssetPreviewMockState,
    node_id: &str,
    property_key: &str,
    next_value: Value,
) -> bool {
    let base_value = document
        .node(node_id)
        .and_then(|node| node.props.get(property_key));
    let changed = if base_value == Some(&next_value) {
        let removed = state
            .overrides
            .get_mut(node_id)
            .and_then(|props| props.remove(property_key))
            .is_some();
        if state
            .overrides
            .get(node_id)
            .is_some_and(|props| props.is_empty())
        {
            let _ = state.overrides.remove(node_id);
        }
        removed
    } else {
        let overrides = state.overrides.entry(node_id.to_string()).or_default();
        if overrides.get(property_key) == Some(&next_value) {
            false
        } else {
            let _ = overrides.insert(property_key.to_string(), next_value);
            true
        }
    };
    if changed {
        state.selected_property = Some(property_key.to_string());
        reconcile_preview_mock_state(document, selection, state);
    }
    changed
}

fn mutate_preview_mock_nested_value(
    value: &mut Value,
    key: &str,
    next_value: Option<Value>,
) -> Result<(), String> {
    let segments = preview_nested_path_segments(value, key)?;
    set_value_at_path(value, &segments, next_value)
}

fn normalize_nested_entry_key(value: &Value, key: &str) -> Result<String, String> {
    let trimmed = key.trim();
    match value {
        Value::Array(items) => {
            if trimmed.is_empty() {
                Ok(items.len().to_string())
            } else {
                let _ = preview_nested_path_segments(value, trimmed)?;
                Ok(trimmed.to_string())
            }
        }
        Value::Table(_) => {
            if trimmed.is_empty() {
                Err("preview mock object entry key is required".to_string())
            } else {
                let _ = preview_nested_path_segments(value, trimmed)?;
                Ok(trimmed.to_string())
            }
        }
        _ => Err("preview mock property does not support nested entries".to_string()),
    }
}

fn collect_preview_mock_nested_entries(
    value: &Value,
    prefix: Option<&str>,
    entries: &mut Vec<UiAssetPreviewMockNestedEntry>,
) {
    match value {
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                let Some(kind) = preview_mock_kind_for_nested_value(item) else {
                    continue;
                };
                let key = match prefix {
                    Some(prefix) => format!("{prefix}[{index}]"),
                    None => index.to_string(),
                };
                let display_key = match prefix {
                    Some(prefix) => format!("{prefix}[{index}]"),
                    None => format!("[{index}]"),
                };
                entries.push(UiAssetPreviewMockNestedEntry {
                    key: key.clone(),
                    display_key,
                    kind,
                    value: item.clone(),
                });
                if matches!(item, Value::Array(_) | Value::Table(_)) {
                    collect_preview_mock_nested_entries(item, Some(key.as_str()), entries);
                }
            }
        }
        Value::Table(table) => {
            let mut keys = table.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                let Some(item) = table.get(&key) else {
                    continue;
                };
                let Some(kind) = preview_mock_kind_for_nested_value(item) else {
                    continue;
                };
                let path = match prefix {
                    Some(prefix) => format!("{prefix}.{key}"),
                    None => key.clone(),
                };
                entries.push(UiAssetPreviewMockNestedEntry {
                    key: path.clone(),
                    display_key: path.clone(),
                    kind,
                    value: item.clone(),
                });
                if matches!(item, Value::Array(_) | Value::Table(_)) {
                    collect_preview_mock_nested_entries(item, Some(path.as_str()), entries);
                }
            }
        }
        _ => {}
    }
}

fn preview_nested_path_segments(
    value: &Value,
    key: &str,
) -> Result<Vec<UiAssetTomlPathSegment>, String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err("preview mock nested path is required".to_string());
    }
    if matches!(value, Value::Array(_)) && !trimmed.contains('.') && !trimmed.contains('[') {
        return trimmed
            .parse::<usize>()
            .map(|index| vec![UiAssetTomlPathSegment::Index(index)])
            .map_err(|_| format!("preview mock collection entry index {trimmed} is invalid"));
    }
    if let Some(parsed) = parse_value_path(trimmed) {
        return Ok(parsed);
    }
    match value {
        Value::Array(_) => Err(format!(
            "preview mock collection entry index {trimmed} is invalid"
        )),
        Value::Table(_) => Ok(vec![UiAssetTomlPathSegment::Key(trimmed.to_string())]),
        _ => Err("preview mock property does not support nested entries".to_string()),
    }
}
