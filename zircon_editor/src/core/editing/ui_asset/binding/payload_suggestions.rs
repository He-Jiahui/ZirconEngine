use toml::Value;

use crate::core::editing::ui_asset::value_path::{get_value_at_path, parse_value_path};

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
    let suggestion_root = Value::Table(root_suggestions.iter().cloned().collect());
    let selected_value = get_value_at_path(&suggestion_root, &selected_path)?;
    let current_selected_value = get_value_at_path(current_payload_root, &selected_path);
    let suggestions = immediate_nested_suggestions(selected_value, current_selected_value);
    (!suggestions.is_empty()).then_some(suggestions)
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
