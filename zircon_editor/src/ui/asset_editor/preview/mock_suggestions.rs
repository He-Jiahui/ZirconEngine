use toml::Value;
use zircon_runtime_interface::ui::template::UiNodeDefinition;

use super::{
    preview_mock_inline_literal, preview_mock_kind_for_nested_value, UiAssetPreviewMockEntry,
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
    immediate_preview_mock_suggestions(root_prefix.as_deref(), root_value)
}

fn suggestion_root<'a>(
    entry: &'a UiAssetPreviewMockEntry,
    selected_nested_key: Option<&str>,
) -> Option<(Option<String>, &'a Value)> {
    if let Some((key, value)) =
        selected_nested_key.and_then(|key| matching_nested_container(key, &entry.effective_value))
    {
        return Some((Some(key), value));
    }

    entry
        .kind
        .supports_nested_entries()
        .then_some((None, &entry.effective_value))
}

fn matching_nested_container<'a>(
    selected_nested_key: &str,
    value: &'a Value,
) -> Option<(String, &'a Value)> {
    matching_nested_container_from(selected_nested_key, value, None)
}

fn matching_nested_container_from<'a>(
    selected_nested_key: &str,
    value: &'a Value,
    prefix: Option<&str>,
) -> Option<(String, &'a Value)> {
    let mut best = None;
    match value {
        Value::Array(entries) => {
            for (index, entry) in entries.iter().enumerate() {
                let path = match prefix {
                    Some(prefix) => format!("{prefix}[{index}]"),
                    None => index.to_string(),
                };
                let Some(candidate) = nested_container_candidate(selected_nested_key, entry, path)
                else {
                    continue;
                };
                if candidate.0 == selected_nested_key {
                    return Some(candidate);
                }
                prefer_deeper_container(&mut best, candidate);
            }
        }
        Value::Table(entries) => {
            for (key, entry) in entries {
                let path = match prefix {
                    Some(prefix) => format!("{prefix}.{key}"),
                    None => key.clone(),
                };
                let Some(candidate) = nested_container_candidate(selected_nested_key, entry, path)
                else {
                    continue;
                };
                if candidate.0 == selected_nested_key {
                    return Some(candidate);
                }
                prefer_deeper_container(&mut best, candidate);
            }
        }
        _ => {}
    }
    best
}

fn nested_container_candidate<'a>(
    selected_nested_key: &str,
    value: &'a Value,
    path: String,
) -> Option<(String, &'a Value)> {
    let is_container = preview_mock_kind_for_nested_value(value)
        .is_some_and(|kind| kind.supports_nested_entries());
    if !is_container || !selected_or_descendant_path(selected_nested_key, &path) {
        return None;
    }

    Some(
        matching_nested_container_from(selected_nested_key, value, Some(&path))
            .unwrap_or((path, value)),
    )
}

fn prefer_deeper_container<'a>(
    best: &mut Option<(String, &'a Value)>,
    candidate: (String, &'a Value),
) {
    if best
        .as_ref()
        .is_none_or(|(best_path, _)| candidate.0.len() > best_path.len())
    {
        *best = Some(candidate);
    }
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
            let mut sorted_entries = entries.iter().collect::<Vec<_>>();
            sorted_entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            sorted_entries
                .into_iter()
                .map(|(key, value)| UiAssetPreviewMockSuggestion {
                    display_key: key.clone(),
                    resolved_key: resolved_object_key(root_prefix, key),
                    value: value.clone(),
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
            let mut sorted_entries = entries.iter().collect::<Vec<_>>();
            sorted_entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (key, entry) in sorted_entries {
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

#[cfg(test)]
#[path = "mock_suggestions/borrowed_root_tests.rs"]
mod borrowed_root_tests;
