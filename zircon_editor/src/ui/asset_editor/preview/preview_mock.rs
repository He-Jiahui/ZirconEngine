use std::collections::BTreeMap;

use crate::ui::asset_editor::value_path::{
    parse_value_path, set_value_at_path, UiAssetTomlPathSegment,
};
use crate::ui::asset_editor::UiDesignerSelectionModel;
use toml::Value;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiNodeDefinition};

mod entries;
#[path = "mock_expression.rs"]
mod mock_expression;
#[path = "mock_suggestions.rs"]
mod mock_suggestions;
#[path = "mock_value_resolution.rs"]
mod mock_value_resolution;

use entries::*;

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
        let Some(node) = document.node(node_id) else {
            return false;
        };
        values.retain(|key, _| {
            node.props
                .get(key)
                .and_then(|value| preview_mock_kind_for_property(key, value))
                .is_some()
        });
        !values.is_empty()
    });
    state.selected_subject_node_id = state.selected_subject_node_id.take().filter(|node_id| {
        document.contains_node(node_id) && preview_mock_node_has_entries(document, node_id)
    });

    let selected_entry = selected_preview_mock_entry(document, selection, state);
    state.selected_property = selected_entry.as_ref().map(|(_, entry)| entry.key.clone());
    let nested_entries = selected_entry
        .as_ref()
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
        let Some(node) = document.node(override_node_id) else {
            return false;
        };
        values.retain(|key, _| {
            node.props
                .get(key)
                .and_then(|value| preview_mock_kind_for_property(key, value))
                .is_some()
        });
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
    let Some(nested_entry) = selected_preview_mock_nested_entry_state(&entry, state) else {
        return Ok(false);
    };
    let next_nested = parse_preview_mock_value(nested_entry.kind, value).ok_or_else(|| {
        format!(
            "preview mock nested property {} expects {}",
            nested_entry.display_key,
            nested_entry.kind.label()
        )
    })?;
    let mut next_value = entry.effective_value;
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
    let normalized_key = normalize_nested_entry_key(&entry.effective_value, key)?;
    let next_nested_value = preview_mock_nested_entries(&entry.effective_value)
        .into_iter()
        .find(|existing| existing.key == normalized_key)
        .and_then(|existing| parse_preview_mock_value(existing.kind, value_literal))
        .unwrap_or_else(|| parse_preview_mock_loose_value(value_literal));
    let mut next_value = entry.effective_value;
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
    let Some(suggestion) =
        mock_suggestions::preview_mock_suggestions(&entry, state.selected_nested_key.as_deref())
            .into_iter()
            .nth(suggestion_index)
    else {
        return Ok(None);
    };

    let mut next_value = entry.effective_value;
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
    let Some(nested_entry) = selected_preview_mock_nested_entry_state(&entry, state) else {
        return Ok(false);
    };
    let mut next_value = entry.effective_value;
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
    let Some((node_id, entry)) = selected_preview_mock_entry(document, selection, state) else {
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
