use toml::Value;
use zircon_ui::template::UiNodeDefinition;

use super::{
    preview_mock_inline_literal, preview_mock_kind_for_nested_value, preview_mock_nested_entries,
    UiAssetPreviewMockEntry, UiAssetPreviewMockNestedEntry,
};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct UiAssetPreviewMockSuggestion {
    pub display_key: String,
    pub resolved_key: String,
    pub value: Value,
}

pub(super) fn build_preview_mock_schema_items(
    node: &UiNodeDefinition,
    node_id: &str,
    entry: &UiAssetPreviewMockEntry,
) -> Vec<String> {
    let base = super::preview_mock_display_key(node, node_id, &entry.key, true);
    let mut schema_items = Vec::new();
    collect_preview_mock_schema_items(&entry.effective_value, &base, &mut schema_items);
    if schema_items.is_empty() {
        schema_items.push(format!("{base} [{}]", entry.kind.label()));
    }
    schema_items.sort();
    schema_items.dedup();
    schema_items
}

pub(super) fn preview_mock_suggestion_items(
    entry: &UiAssetPreviewMockEntry,
    selected_nested_key: Option<&str>,
) -> Vec<String> {
    preview_mock_suggestions(entry, selected_nested_key)
        .into_iter()
        .map(|suggestion| {
            format!(
                "{} = {}",
                suggestion.display_key,
                preview_mock_inline_literal(&suggestion.value)
            )
        })
        .collect()
}

pub(super) fn preview_mock_suggestions(
    entry: &UiAssetPreviewMockEntry,
    selected_nested_key: Option<&str>,
) -> Vec<UiAssetPreviewMockSuggestion> {
    let Some((root_prefix, root_value)) = suggestion_root(entry, selected_nested_key) else {
        return Vec::new();
    };
    immediate_preview_mock_suggestions(root_prefix.as_deref(), &root_value)
}

fn suggestion_root(
    entry: &UiAssetPreviewMockEntry,
    selected_nested_key: Option<&str>,
) -> Option<(Option<String>, Value)> {
    let nested_entries = preview_mock_nested_entries(&entry.effective_value);
    if let Some(selected_nested_key) = selected_nested_key.and_then(|key| {
        matching_nested_container(key, &nested_entries).map(|entry| entry.key.clone())
    }) {
        let nested_entry = nested_entries
            .iter()
            .find(|entry| entry.key == selected_nested_key)?;
        return Some((Some(nested_entry.key.clone()), nested_entry.value.clone()));
    }

    entry
        .kind
        .supports_nested_entries()
        .then_some((None, entry.effective_value.clone()))
}

fn matching_nested_container<'a>(
    selected_nested_key: &str,
    nested_entries: &'a [UiAssetPreviewMockNestedEntry],
) -> Option<&'a UiAssetPreviewMockNestedEntry> {
    nested_entries
        .iter()
        .filter(|entry| {
            entry.kind.supports_nested_entries()
                && selected_or_descendant_path(selected_nested_key, &entry.key)
        })
        .max_by_key(|entry| entry.key.len())
}

fn selected_or_descendant_path(selected: &str, candidate: &str) -> bool {
    if selected == candidate {
        return true;
    }
    selected
        .strip_prefix(candidate)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|ch| matches!(ch, '.' | '['))
}

fn immediate_preview_mock_suggestions(
    root_prefix: Option<&str>,
    value: &Value,
) -> Vec<UiAssetPreviewMockSuggestion> {
    match value {
        Value::Array(entries) => {
            let mut suggestions = entries
                .iter()
                .enumerate()
                .map(|(index, entry)| UiAssetPreviewMockSuggestion {
                    display_key: format!("[{index}]"),
                    resolved_key: resolved_collection_key(root_prefix, index),
                    value: entry.clone(),
                })
                .collect::<Vec<_>>();
            if let Some(template) = entries.first().cloned() {
                suggestions.push(UiAssetPreviewMockSuggestion {
                    display_key: "[n]".to_string(),
                    resolved_key: resolved_collection_key(root_prefix, entries.len()),
                    value: template,
                });
            }
            suggestions
        }
        Value::Table(entries) => {
            let mut keys = entries.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
                .filter_map(|key| {
                    entries
                        .get(&key)
                        .cloned()
                        .map(|value| UiAssetPreviewMockSuggestion {
                            display_key: key.clone(),
                            resolved_key: resolved_object_key(root_prefix, &key),
                            value,
                        })
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn resolved_object_key(root_prefix: Option<&str>, key: &str) -> String {
    match root_prefix {
        Some(prefix) if !prefix.is_empty() => format!("{prefix}.{key}"),
        _ => key.to_string(),
    }
}

fn resolved_collection_key(root_prefix: Option<&str>, index: usize) -> String {
    match root_prefix {
        Some(prefix) if !prefix.is_empty() => format!("{prefix}[{index}]"),
        _ => index.to_string(),
    }
}

fn collect_preview_mock_schema_items(value: &Value, base: &str, items: &mut Vec<String>) {
    match value {
        Value::Array(entries) => {
            for (index, entry) in entries.iter().enumerate() {
                let Some(kind) = preview_mock_kind_for_nested_value(entry) else {
                    continue;
                };
                let path = format!("{base}[{index}]");
                items.push(format!("{path} [{}]", kind.label()));
                if matches!(entry, Value::Array(_) | Value::Table(_)) {
                    collect_preview_mock_schema_items(entry, &path, items);
                }
            }
            let fallback_kind = entries
                .first()
                .and_then(preview_mock_kind_for_nested_value)
                .map(|kind| kind.label().to_string())
                .unwrap_or_else(|| "Value".to_string());
            items.push(format!("{base}[n] [{fallback_kind}]"));
        }
        Value::Table(entries) => {
            let mut keys = entries.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                let Some(entry) = entries.get(&key) else {
                    continue;
                };
                let Some(kind) = preview_mock_kind_for_nested_value(entry) else {
                    continue;
                };
                let path = format!("{base}.{key}");
                items.push(format!("{path} [{}]", kind.label()));
                if matches!(entry, Value::Array(_) | Value::Table(_)) {
                    collect_preview_mock_schema_items(entry, &path, items);
                }
            }
        }
        _ => {}
    }
}
