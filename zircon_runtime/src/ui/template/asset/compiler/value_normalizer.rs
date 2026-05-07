use std::collections::BTreeMap;

use toml::{map::Map, Value};

use zircon_runtime_interface::ui::template::UiNodeDefinition;

const MAX_TOKEN_RESOLUTION_DEPTH: usize = 32;

pub(super) fn build_attribute_map(
    node: &UiNodeDefinition,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut attributes = resolve_value_map(&node.props, tokens, params);
    if let Some(layout) = &node.layout {
        let mut layout = resolve_value_map(layout, tokens, params);
        normalize_layout_table(&mut layout);
        let _ = attributes.insert("layout".to_string(), table_value(layout));
    }
    attributes
}

pub(super) fn compose_tokens(
    inherited: &BTreeMap<String, Value>,
    local: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut tokens = inherited.clone();
    merge_value_maps(&mut tokens, local);
    tokens
}

pub(super) fn resolve_value_map(
    values: &BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    values
        .iter()
        .map(|(key, value)| (key.clone(), resolve_value(value, tokens, params)))
        .collect()
}

pub(super) fn resolve_value(
    value: &Value,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) -> Value {
    resolve_value_at_depth(value, tokens, params, 0)
}

fn resolve_value_at_depth(
    value: &Value,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
    depth: usize,
) -> Value {
    if depth >= MAX_TOKEN_RESOLUTION_DEPTH {
        return value.clone();
    }

    match value {
        Value::String(value) => {
            if let Some(param_name) = value.strip_prefix("$param.") {
                params
                    .get(param_name)
                    .map(|value| resolve_value_at_depth(value, tokens, params, depth + 1))
                    .unwrap_or_else(|| Value::String(value.clone()))
            } else if let Some(token_name) = value.strip_prefix('$') {
                tokens
                    .get(token_name)
                    .map(|value| resolve_value_at_depth(value, tokens, params, depth + 1))
                    .unwrap_or_else(|| Value::String(value.clone()))
            } else {
                Value::String(value.clone())
            }
        }
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| resolve_value_at_depth(value, tokens, params, depth + 1))
                .collect(),
        ),
        Value::Table(values) => Value::Table(
            values
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        resolve_value_at_depth(value, tokens, params, depth + 1),
                    )
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

pub(super) fn merge_value_maps_resolved(
    target: &mut BTreeMap<String, Value>,
    overlay: &BTreeMap<String, Value>,
    tokens: &BTreeMap<String, Value>,
    params: &BTreeMap<String, Value>,
) {
    let resolved = resolve_value_map(overlay, tokens, params);
    merge_value_maps(target, &resolved);
}

pub(super) fn merge_value_maps(
    target: &mut BTreeMap<String, Value>,
    overlay: &BTreeMap<String, Value>,
) {
    for (key, value) in overlay {
        if let Some(current) = target.get_mut(key) {
            merge_value(current, value);
        } else {
            let _ = target.insert(key.clone(), value.clone());
        }
    }
    normalize_layout(target);
}

fn merge_value(current: &mut Value, overlay: &Value) {
    match (current, overlay) {
        (Value::Table(current_table), Value::Table(overlay_table)) => {
            for (key, value) in overlay_table {
                if let Some(existing) = current_table.get_mut(key) {
                    merge_value(existing, value);
                } else {
                    let _ = current_table.insert(key.clone(), value.clone());
                }
            }
        }
        (current, overlay) => *current = overlay.clone(),
    }
}

pub(super) fn normalize_layout(values: &mut BTreeMap<String, Value>) {
    let Some(Value::Table(layout)) = values.get_mut("layout") else {
        return;
    };
    normalize_layout_table_map(layout);
}

fn normalize_layout_table(values: &mut BTreeMap<String, Value>) {
    normalize_axis_entry(values.get_mut("width"));
    normalize_axis_entry(values.get_mut("height"));
}

fn normalize_layout_table_map(values: &mut Map<String, Value>) {
    normalize_axis_entry(values.get_mut("width"));
    normalize_axis_entry(values.get_mut("height"));
}

fn normalize_axis_entry(value: Option<&mut Value>) {
    normalize_axis_table(value);
}

fn normalize_axis_table(value: Option<&mut Value>) {
    let Some(Value::Table(table)) = value else {
        return;
    };
    if table.get("stretch").and_then(Value::as_str) == Some("Fixed") {
        if let Some(preferred) = table.get("preferred").cloned() {
            let _ = table.insert("min".to_string(), preferred.clone());
            let _ = table.insert("max".to_string(), preferred);
        }
    }
}

fn table_value(values: BTreeMap<String, Value>) -> Value {
    Value::Table(values.into_iter().collect::<Map<String, Value>>())
}

pub(super) fn append_classes(target: &mut Vec<String>, extra: &[String]) {
    for class_name in extra {
        if !target.iter().any(|value| value == class_name) {
            target.push(class_name.clone());
        }
    }
}
