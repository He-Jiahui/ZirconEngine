use super::*;

pub(super) fn preview_mock_entries(
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
    entries.sort_by(|left, right| {
        preview_mock_sort_key(&left.key, left.kind)
            .cmp(&preview_mock_sort_key(&right.key, right.kind))
    });
    entries
}

pub(super) fn preview_mock_nested_entries(value: &Value) -> Vec<UiAssetPreviewMockNestedEntry> {
    let mut entries = Vec::new();
    collect_preview_mock_nested_entries(value, None, &mut entries);
    entries.sort_by(|left, right| {
        preview_mock_sort_key(&left.key, left.kind)
            .cmp(&preview_mock_sort_key(&right.key, right.kind))
    });
    entries
}

pub(super) fn preview_mock_subject_node_id<'a>(
    document: &'a UiAssetDocument,
    selection: &'a UiDesignerSelectionModel,
    state: &'a UiAssetPreviewMockState,
) -> Option<&'a str> {
    resolved_preview_mock_subject_node_id(document, selection, state)
}

pub(super) fn resolved_preview_mock_subject_node_id<'a>(
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
                .filter(|node| preview_mock_node_has_entries_for_node(node))
                .min_by(|left, right| {
                    preview_mock_subject_sort_key(left).cmp(&preview_mock_subject_sort_key(right))
                })
                .map(|node| node.node_id.as_str())
        })
}

pub(super) fn selected_preview_mock_entry(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
    state: &UiAssetPreviewMockState,
) -> Option<(String, UiAssetPreviewMockEntry)> {
    let node_id = preview_mock_subject_node_id(document, selection, state)?.to_string();
    let entries = preview_mock_entries(document, selection, state);
    let selected_index = selected_entry_index(&entries, state.selected_property.as_deref())?;
    Some((node_id, entries.get(selected_index)?.clone()))
}

pub(super) fn selected_preview_mock_nested_entry_state(
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

pub(super) fn selected_entry_index(
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

pub(super) fn selected_nested_entry_index(
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

pub(super) fn preview_mock_subject_entries(
    document: &UiAssetDocument,
) -> Vec<UiAssetPreviewMockSubjectEntry> {
    let mut entries = document
        .iter_nodes()
        .filter(|node| preview_mock_node_has_entries_for_node(node))
        .map(|node| UiAssetPreviewMockSubjectEntry {
            node_id: node.node_id.clone(),
            label: preview_mock_subject_label(node),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.label.cmp(&right.label));
    entries
}

pub(super) fn preview_mock_subject_label(node: &UiNodeDefinition) -> String {
    format!(
        "{} • {}",
        node.control_id.as_deref().unwrap_or(node.node_id.as_str()),
        node.node_id
    )
}

pub(super) fn preview_mock_subject_sort_key(node: &UiNodeDefinition) -> (&str, &str) {
    (
        node.control_id.as_deref().unwrap_or(node.node_id.as_str()),
        node.node_id.as_str(),
    )
}

pub(super) fn preview_mock_node_has_entries(document: &UiAssetDocument, node_id: &str) -> bool {
    document
        .node(node_id)
        .map(preview_mock_node_has_entries_for_node)
        .unwrap_or(false)
}

fn preview_mock_node_has_entries_for_node(node: &UiNodeDefinition) -> bool {
    node.props
        .iter()
        .any(|(key, value)| preview_mock_kind_for_property(key, value).is_some())
}

pub(super) fn evaluate_preview_mock_expression(
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

pub(super) fn resolve_preview_mock_expression<'a>(
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

pub(super) fn resolve_preview_mock_expression_node<'a>(
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

pub(super) fn preview_mock_property_value<'a>(
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

pub(super) fn preview_mock_nested_value<'a>(value: &'a Value, segment: &str) -> Option<&'a Value> {
    match value {
        Value::Array(items) => items.get(segment.parse::<usize>().ok()?),
        Value::Table(table) => table.get(segment),
        _ => None,
    }
}

pub(super) fn preview_mock_kind_for_property(
    key: &str,
    value: &Value,
) -> Option<UiAssetPreviewMockKind> {
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

pub(super) fn preview_mock_kind_for_nested_value(value: &Value) -> Option<UiAssetPreviewMockKind> {
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

pub(super) fn enum_like_property(key: &str) -> bool {
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

pub(super) fn is_resource_reference(value: &str) -> bool {
    value.starts_with("asset://") || value.starts_with("res://")
}

pub(super) fn expression_like_property(key: &str, value: &str) -> bool {
    key.ends_with("_expr") || key.contains("expression") || value.trim_start().starts_with('=')
}

pub(super) fn parse_preview_mock_value(kind: UiAssetPreviewMockKind, value: &str) -> Option<Value> {
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

pub(super) fn parse_preview_mock_loose_value(value: &str) -> Value {
    parse_toml_inline_value(value).unwrap_or_else(|| Value::String(value.to_string()))
}

pub(super) fn parse_toml_inline_value(value: &str) -> Option<Value> {
    let table = format!("value = {value}").parse::<toml::Table>().ok()?;
    table.get("value").cloned()
}

pub(super) fn parse_bool(value: &str) -> Option<bool> {
    let value = value.trim();
    if value == "1"
        || ["true", "yes", "on"]
            .iter()
            .any(|candidate| value.eq_ignore_ascii_case(candidate))
    {
        Some(true)
    } else if value == "0"
        || ["false", "no", "off"]
            .iter()
            .any(|candidate| value.eq_ignore_ascii_case(candidate))
    {
        Some(false)
    } else {
        None
    }
}

pub(super) fn preview_mock_literal(value: &Value) -> String {
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

pub(super) fn preview_mock_inline_literal(value: &Value) -> String {
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

pub(super) fn preview_mock_display_key(
    node: &UiNodeDefinition,
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

pub(super) fn qualified_preview_mock_nested_display_key(base: &str, nested_key: &str) -> String {
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

pub(super) fn preview_mock_sort_key(key: &str, kind: UiAssetPreviewMockKind) -> (u8, &str) {
    if key == "text" {
        return (0, key);
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
    (priority, key)
}

pub(super) fn set_preview_mock_override_value(
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

pub(super) fn mutate_preview_mock_nested_value(
    value: &mut Value,
    key: &str,
    next_value: Option<Value>,
) -> Result<(), String> {
    let segments = preview_nested_path_segments(value, key)?;
    set_value_at_path(value, &segments, next_value)
}

pub(super) fn normalize_nested_entry_key(value: &Value, key: &str) -> Result<String, String> {
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

pub(super) fn collect_preview_mock_nested_entries(
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
            let mut sorted_entries = table.iter().collect::<Vec<_>>();
            sorted_entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (key, item) in sorted_entries {
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

pub(super) fn preview_nested_path_segments(
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
