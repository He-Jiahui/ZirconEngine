use toml::Value;

use crate::ui::asset_editor::value_path::{
    get_value_at_path, parse_value_path, UiAssetTomlPathSegment,
};

#[cfg(test)]
#[path = "payload_suggestions/borrowed_root_tests.rs"]
mod borrowed_root_tests;

pub(super) fn contextual_binding_payload_suggestions(
    root_suggestions: &[(String, Value)],
    current_payload_root: &Value,
    selected_payload_key: Option<&str>,
) -> Option<Vec<(String, Value)>> {
    let selected_payload_key = selected_payload_key?.trim();
    if selected_payload_key.is_empty() {
        return None;
    }

    let selected_path = parse_value_path(selected_payload_key)?;
    let selected_value = borrowed_suggestion_value(root_suggestions, &selected_path)?;
    let current_selected_value = get_value_at_path(current_payload_root, &selected_path);
    let suggestions = immediate_nested_suggestions(selected_value, current_selected_value);
    (!suggestions.is_empty()).then_some(suggestions)
}

fn borrowed_suggestion_value<'a>(
    root_suggestions: &'a [(String, Value)],
    path: &[UiAssetTomlPathSegment],
) -> Option<&'a Value> {
    let (head, tail) = path.split_first()?;
    let UiAssetTomlPathSegment::Key(root_key) = head else {
        return None;
    };
    let root_value = root_suggestions
        .iter()
        .rev()
        .find_map(|(key, value)| (key == root_key).then_some(value))?;
    get_value_at_path(root_value, tail)
}

fn immediate_nested_suggestions(
    value: &Value,
    current_selected_value: Option<&Value>,
) -> Vec<(String, Value)> {
    match value {
        Value::Array(entries) => {
            let mut suggestions = entries
                .iter()
                .enumerate()
                .map(|(index, entry)| (format!("[{index}]"), entry.clone()))
                .collect::<Vec<_>>();
            let append_index = current_selected_value
                .and_then(Value::as_array)
                .map(|current_entries| current_entries.len().max(entries.len()))
                .unwrap_or(entries.len());
            if let Some(template) = entries.first().cloned() {
                suggestions.push((format!("[{append_index}]"), template));
            }
            suggestions
        }
        Value::Table(entries) => {
            let mut keys = entries.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            keys.into_iter()
                .filter_map(|key| entries.get(&key).cloned().map(|value| (key, value)))
                .collect()
        }
        _ => Vec::new(),
    }
}
