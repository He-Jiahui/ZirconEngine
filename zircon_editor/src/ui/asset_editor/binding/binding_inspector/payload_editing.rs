use super::*;

fn payload_field_name(field: UiActionPayloadFieldName) -> String {
    field.schema_name().to_string()
}

pub(super) fn binding_payload_suggestions(
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

pub(super) fn binding_root_payload_suggestions(binding: &UiBindingRef) -> Vec<(String, Value)> {
    if let Some(target_specific) = binding_target_payload_suggestions(binding) {
        return target_specific;
    }

    match binding.event {
        UiEventKind::Click
        | UiEventKind::DoubleClick
        | UiEventKind::Press
        | UiEventKind::Release
        | UiEventKind::Submit => vec![
            (
                payload_field_name(UiActionPayloadFieldName::Confirm),
                Value::Boolean(true),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Channel),
                Value::String("toolbar".to_string()),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Source),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::Change => vec![
            (
                payload_field_name(UiActionPayloadFieldName::Value),
                Value::String("preview".to_string()),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Committed),
                Value::Boolean(true),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Source),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::Toggle => vec![
            (
                payload_field_name(UiActionPayloadFieldName::Checked),
                Value::Boolean(true),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Source),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::Scroll => vec![
            (
                payload_field_name(UiActionPayloadFieldName::Axis),
                Value::String("vertical".to_string()),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Delta),
                Value::Integer(1),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Source),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::DragBegin | UiEventKind::DragUpdate | UiEventKind::DragEnd => vec![
            (
                payload_field_name(UiActionPayloadFieldName::Axis),
                Value::String("x".to_string()),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Delta),
                Value::Integer(0),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Source),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        UiEventKind::Drop => vec![
            (
                payload_field_name(UiActionPayloadFieldName::PayloadKind),
                Value::String("asset".to_string()),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Reference),
                Value::String("res://textures/grid.albedo.png".to_string()),
            ),
            (
                payload_field_name(UiActionPayloadFieldName::Source),
                Value::String(event_source_tag(binding.event)),
            ),
        ],
        _ => vec![(
            payload_field_name(UiActionPayloadFieldName::Source),
            Value::String(event_source_tag(binding.event)),
        )],
    }
}

pub(super) fn binding_schema_payload_entries(binding: &UiBindingRef) -> Vec<(String, Value)> {
    if binding_action_kind(binding) == UiBindingActionKind::Action {
        let action_target = binding_action_specific_target(binding);
        let action_target = action_target.to_ascii_lowercase();
        if action_target.contains("project.save") {
            return vec![
                (
                    payload_field_name(UiActionPayloadFieldName::Confirm),
                    Value::Boolean(true),
                ),
                (
                    payload_field_name(UiActionPayloadFieldName::Source),
                    Value::String(event_source_tag(binding.event)),
                ),
            ];
        }
    }
    binding_root_payload_suggestions(binding)
}

pub(super) fn binding_target_payload_suggestions(
    binding: &UiBindingRef,
) -> Option<Vec<(String, Value)>> {
    match binding_action_kind(binding) {
        UiBindingActionKind::Route => {
            let route_target = binding_route_target(binding);
            let route_key = normalized_route_target_key(&route_target);
            if route_key.contains("selection.changed") {
                return Some(vec![
                    (
                        payload_field_name(UiActionPayloadFieldName::Primary),
                        Value::String("SelectedNode".to_string()),
                    ),
                    (
                        payload_field_name(UiActionPayloadFieldName::SelectionIds),
                        Value::Array(vec![Value::String("SelectedNode".to_string())]),
                    ),
                    (
                        payload_field_name(UiActionPayloadFieldName::Context),
                        toml::Value::Table(
                            [
                                (
                                    payload_field_name(UiActionPayloadFieldName::Additive),
                                    Value::Boolean(false),
                                ),
                                (
                                    payload_field_name(UiActionPayloadFieldName::Source),
                                    Value::String("hierarchy".to_string()),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ]);
            }
            if route_key.contains("form.valuechanged") {
                return Some(vec![
                    (
                        payload_field_name(UiActionPayloadFieldName::Value),
                        Value::String("preview".to_string()),
                    ),
                    (
                        payload_field_name(UiActionPayloadFieldName::Committed),
                        Value::Boolean(true),
                    ),
                    (
                        payload_field_name(UiActionPayloadFieldName::Fields),
                        Value::Array(vec![Value::String("title".to_string())]),
                    ),
                    (
                        payload_field_name(UiActionPayloadFieldName::Context),
                        toml::Value::Table(
                            [
                                (
                                    payload_field_name(UiActionPayloadFieldName::Source),
                                    Value::String(event_source_tag(binding.event)),
                                ),
                                (
                                    payload_field_name(UiActionPayloadFieldName::Subject),
                                    Value::String("field".to_string()),
                                ),
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
            let action_target = action_target.to_ascii_lowercase();
            if action_target.contains("visibility.toggle") {
                return Some(vec![
                    (
                        payload_field_name(UiActionPayloadFieldName::Checked),
                        Value::Boolean(true),
                    ),
                    (
                        payload_field_name(UiActionPayloadFieldName::SelectionIds),
                        Value::Array(vec![Value::String("SelectedNode".to_string())]),
                    ),
                    (
                        payload_field_name(UiActionPayloadFieldName::Context),
                        toml::Value::Table(
                            [
                                (
                                    payload_field_name(UiActionPayloadFieldName::Scope),
                                    Value::String("selection".to_string()),
                                ),
                                (
                                    payload_field_name(UiActionPayloadFieldName::Source),
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

fn normalized_route_target_key(route_target: &str) -> String {
    route_target
        .trim()
        .trim_start_matches("route.")
        .chars()
        .filter_map(|ch| match ch {
            '_' | '-' | ':' | '/' | ' ' => None,
            ch => Some(ch.to_ascii_lowercase()),
        })
        .collect()
}

pub(super) fn binding_value_kind_label(value: &Value) -> &'static str {
    match value {
        Value::String(text) if text.trim_start().starts_with('=') => "Expression",
        Value::Boolean(_) => "Bool",
        Value::Integer(_) | Value::Float(_) => "Number",
        Value::Array(_) => "Collection",
        Value::Table(_) => "Object",
        _ => "Text",
    }
}

pub(super) fn binding_route_suggestions(
    node: &UiNodeDefinition,
    binding: &UiBindingRef,
) -> Vec<String> {
    let mut suggestions = Vec::new();
    let keywords = binding_keywords(node);
    if binding_event_supports_keyword_shortcuts(binding.event) && is_save_like(&keywords) {
        suggestions.push("menu_action.workbench.project.save".to_string());
    }
    match binding.event {
        UiEventKind::Click | UiEventKind::DoubleClick | UiEventKind::Submit => {
            suggestions.push("menu_action.workbench.project.open".to_string());
            suggestions.push("menu_action.workbench.layout.save".to_string());
            suggestions.push(format!("route.{}", binding_route_slug(node, binding)));
        }
        UiEventKind::Change => {
            suggestions.push("route.selection.changed".to_string());
            suggestions.push("route.form.value_changed".to_string());
        }
        UiEventKind::Toggle => {
            suggestions.push("route.toggle.changed".to_string());
            suggestions.push("route.panel.visibility_changed".to_string());
        }
        UiEventKind::Drop => {
            suggestions.push("route.reference.dropped".to_string());
            suggestions.push("route.asset.accept_drop".to_string());
        }
        _ => {
            suggestions.push(format!("route.{}", binding_route_slug(node, binding)));
        }
    }
    dedupe_suggestions(suggestions)
}

pub(super) fn binding_action_suggestions(
    node: &UiNodeDefinition,
    binding: &UiBindingRef,
) -> Vec<String> {
    let mut suggestions = Vec::new();
    let keywords = binding_keywords(node);
    if binding_event_supports_keyword_shortcuts(binding.event) && is_save_like(&keywords) {
        suggestions.push("editor_action.workbench.project.save".to_string());
    }
    match binding.event {
        UiEventKind::Click | UiEventKind::DoubleClick | UiEventKind::Submit => {
            suggestions.push("editor_action.workbench.asset_browser.open".to_string());
            suggestions.push("editor_action.workbench.selection.focus".to_string());
        }
        UiEventKind::Change => {
            suggestions.push("editor_action.workbench.preview.refresh".to_string());
            suggestions.push("editor_action.workbench.inspector.apply".to_string());
        }
        UiEventKind::Toggle => {
            suggestions.push("editor_action.workbench.visibility.toggle".to_string());
            suggestions.push("editor_action.workbench.selection_state.toggle".to_string());
        }
        UiEventKind::Drop => {
            suggestions.push("editor_action.workbench.asset_drop.accept".to_string());
            suggestions.push("editor_action.workbench.reference.assign".to_string());
        }
        _ => {
            suggestions.push(format!(
                "editor_action.workbench.custom.{}",
                binding_action_path_slug(node, binding)
            ));
        }
    }
    dedupe_suggestions(suggestions)
}

pub(super) fn binding_keywords(node: &UiNodeDefinition) -> String {
    let control_id = node.control_id.as_deref().unwrap_or_default();
    let text = node
        .props
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or_default();
    format!("{control_id} {text}").to_ascii_lowercase()
}

pub(super) fn is_save_like(keywords: &str) -> bool {
    keywords.contains("save")
}

pub(super) fn binding_event_supports_keyword_shortcuts(event: UiEventKind) -> bool {
    matches!(
        event,
        UiEventKind::Click | UiEventKind::DoubleClick | UiEventKind::Submit
    )
}

pub(super) fn binding_route_slug(node: &UiNodeDefinition, binding: &UiBindingRef) -> String {
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

pub(super) fn binding_action_path_slug(node: &UiNodeDefinition, binding: &UiBindingRef) -> String {
    let base = node
        .control_id
        .as_deref()
        .or_else(|| node.component.as_deref())
        .unwrap_or("binding");
    format!(
        "{}.{}",
        sanitize_path_segment(base),
        sanitize_path_segment(binding.event.native_name())
    )
}

pub(super) fn sanitize_identifier(value: &str) -> String {
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

pub(super) fn sanitize_path_segment(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_separator = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !normalized.is_empty() {
                normalized.push('_');
            }
            normalized.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !normalized.ends_with('_') && !normalized.is_empty() {
            normalized.push('_');
            previous_was_separator = true;
        }
    }
    let normalized = normalized.trim_matches('_');
    if normalized.is_empty() {
        "binding".to_string()
    } else {
        normalized.to_string()
    }
}

pub(super) fn dedupe_suggestions(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

pub(super) fn event_source_tag(event: UiEventKind) -> String {
    event
        .native_name()
        .strip_prefix("on")
        .map(|name| format!("ui.{}", name.to_ascii_lowercase()))
        .unwrap_or_else(|| "ui.event".to_string())
}

pub(super) fn selected_payload_key_for_binding(
    binding: &UiBindingRef,
    current: Option<&str>,
) -> Option<String> {
    let payload = binding_payload_item_entries(binding);
    selected_payload_key_from_entries(&payload, current)
}

pub(super) fn selected_payload_key_from_entries(
    payload: &[(String, &Value)],
    current: Option<&str>,
) -> Option<String> {
    if payload.is_empty() {
        return None;
    }
    current
        .filter(|key| payload.iter().any(|(path, _)| path == key))
        .map(str::to_string)
        .or_else(|| payload.first().map(|(path, _)| path.clone()))
}

pub(super) fn binding_payload_root_value(binding: &UiBindingRef) -> Value {
    Value::Table(
        binding
            .action
            .as_ref()
            .map(|action| action.payload.clone().into_iter().collect())
            .unwrap_or_default(),
    )
}

pub(super) fn resolve_binding_payload_upsert_path(
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

pub(super) fn resolve_relative_binding_payload_upsert_path(
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

pub(super) fn payload_key_anchors_selected_path(
    selected_payload_key: &str,
    payload_key: &str,
) -> bool {
    let trimmed = payload_key.trim();
    trimmed == selected_payload_key
        || trimmed
            .strip_prefix(selected_payload_key)
            .is_some_and(|rest| rest.starts_with('.') || rest.starts_with('['))
}

pub(super) fn join_payload_key(selected_payload_key: &str, relative_key: &str) -> String {
    if relative_key.starts_with('[') {
        format!("{selected_payload_key}{relative_key}")
    } else {
        format!("{selected_payload_key}.{relative_key}")
    }
}

pub(super) fn parse_relative_collection_payload_path(
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

pub(super) fn collect_binding_payload_item_entries<'a>(
    value: &'a Value,
    prefix: Option<&str>,
    entries: &mut Vec<(String, &'a Value)>,
) {
    if let Some(prefix) = prefix {
        entries.push((prefix.to_string(), value));
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
            let mut sorted_entries = table.iter().collect::<Vec<_>>();
            sorted_entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (key, item) in sorted_entries {
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

pub(super) fn apply_binding_action_state(
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
                    payload_missing_policy: Default::default(),
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
                    payload_missing_policy: Default::default(),
                });
            }
        }
    }
}

pub(super) fn ensure_binding_action_for_payload(binding: &mut UiBindingRef) -> &mut UiActionRef {
    if binding.action.is_none() {
        binding.action = Some(UiActionRef {
            route: binding.route.clone(),
            action: None,
            payload: BTreeMap::new(),
            payload_missing_policy: Default::default(),
        });
    }
    binding
        .action
        .as_mut()
        .expect("binding action should exist after initialization")
}

pub(super) fn compact_binding_action(binding: &mut UiBindingRef) {
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

pub(super) fn normalized_binding_id(value: &str, default_id: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_id.to_string()
    } else {
        trimmed.to_string()
    }
}

pub(super) fn normalized_binding_target(
    value: &str,
    kind: UiBindingSchemaNameKind,
) -> Option<String> {
    let trimmed = value.trim();
    kind.validate(trimmed).ok().map(|()| trimmed.to_string())
}

pub(super) fn normalized_payload_key(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let path = parse_value_path(trimmed)?;
    path.iter()
        .filter_map(|segment| match segment {
            UiAssetTomlPathSegment::Key(key) => Some(key.as_str()),
            UiAssetTomlPathSegment::Index(_) => None,
        })
        .all(|key| UiBindingSchemaNameKind::PayloadField.validate(key).is_ok())
        .then(|| trimmed.to_string())
}

pub(super) fn format_binding_item(binding: &UiBindingRef) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn route_binding(route: &str) -> UiBindingRef {
        UiBindingRef {
            component_event: None,
            id: "Button/onClick".to_string(),
            event: UiEventKind::Click,
            mode: Default::default(),
            route: Some(route.to_string()),
            action: None,
            targets: Vec::new(),
        }
    }

    #[test]
    fn dotted_form_route_uses_form_value_changed_payload_suggestions() {
        let suggestions =
            binding_root_payload_suggestions(&route_binding("route.form.value_changed"));
        assert_eq!(
            suggestions,
            vec![
                ("value".to_string(), Value::String("preview".to_string())),
                ("committed".to_string(), Value::Boolean(true)),
                (
                    "fields".to_string(),
                    Value::Array(vec![Value::String("title".to_string())])
                ),
                (
                    "context".to_string(),
                    Value::Table(
                        [
                            ("source".to_string(), Value::String("ui.click".to_string())),
                            ("subject".to_string(), Value::String("field".to_string())),
                        ]
                        .into_iter()
                        .collect()
                    )
                ),
            ]
        );
    }

    #[test]
    fn dotted_selection_route_uses_selection_payload_suggestions() {
        let suggestions =
            binding_root_payload_suggestions(&route_binding("route.selection.changed"));
        assert_eq!(
            suggestions,
            vec![
                (
                    "primary".to_string(),
                    Value::String("SelectedNode".to_string())
                ),
                (
                    "selection_ids".to_string(),
                    Value::Array(vec![Value::String("SelectedNode".to_string())])
                ),
                (
                    "context".to_string(),
                    Value::Table(
                        [
                            ("additive".to_string(), Value::Boolean(false)),
                            ("source".to_string(), Value::String("hierarchy".to_string())),
                        ]
                        .into_iter()
                        .collect()
                    )
                ),
            ]
        );
    }

    #[test]
    fn editor_normalization_uses_the_shared_binding_name_schema() {
        assert_eq!(
            normalized_binding_target(" workbench.asset.open ", UiBindingSchemaNameKind::Route),
            Some("workbench.asset.open".to_string())
        );
        assert_eq!(
            normalized_binding_target("workbench..open", UiBindingSchemaNameKind::Route),
            None
        );
        assert_eq!(
            normalized_binding_target("view/console", UiBindingSchemaNameKind::Action),
            None
        );
        assert_eq!(
            normalized_payload_key("context.source"),
            Some("context.source".to_string())
        );
        assert_eq!(normalized_payload_key("context.not valid"), None);
    }
}
