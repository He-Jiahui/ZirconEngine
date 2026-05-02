use std::collections::BTreeSet;

use toml::Value;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiBindingRef};

use super::UiAssetPreviewMockState;
use super::{
    mock_expression, preview_mock_display_key, preview_mock_inline_literal,
    resolve_preview_mock_expression,
};

pub(super) fn resolve_preview_mock_value_preview(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    value: &Value,
) -> Option<Value> {
    match value {
        Value::String(text) if text.trim_start().starts_with('=') => {
            resolve_expression_value(document, state, current_node_id, text)
        }
        Value::Array(items) => {
            let mut changed = false;
            let mut resolved = Vec::with_capacity(items.len());
            for item in items {
                if let Some(next) =
                    resolve_preview_mock_value_preview(document, state, current_node_id, item)
                {
                    changed = true;
                    resolved.push(next);
                } else {
                    resolved.push(item.clone());
                }
            }
            changed.then_some(Value::Array(resolved))
        }
        Value::Table(table) => {
            let mut changed = false;
            let mut resolved = toml::map::Map::new();
            for (key, nested) in table {
                if let Some(next) =
                    resolve_preview_mock_value_preview(document, state, current_node_id, nested)
                {
                    changed = true;
                    let _ = resolved.insert(key.clone(), next);
                } else {
                    let _ = resolved.insert(key.clone(), nested.clone());
                }
            }
            changed.then_some(Value::Table(resolved))
        }
        _ => None,
    }
}

fn resolve_expression_value(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    expression_text: &str,
) -> Option<Value> {
    let expression = expression_text.trim();
    let expression = expression.strip_prefix('=')?.trim();
    if expression.is_empty() {
        return Some(Value::String(String::new()));
    }

    if let Some(resolved) =
        resolve_reference_expression(document, state, current_node_id, expression)
    {
        return Some(resolved);
    }

    let (function_name, args) = parse_function_expression(expression)?;
    evaluate_function_expression(document, state, current_node_id, &function_name, &args)
}

fn resolve_reference_expression(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    reference: &str,
) -> Option<Value> {
    mock_expression::parse_preview_mock_reference(reference)?;
    let wrapped = Value::String(format!("={reference}"));
    resolve_preview_mock_expression(document, state, current_node_id, &wrapped)
        .map(|(_, _, resolved)| resolved.clone())
}

pub(super) fn collect_preview_mock_expression_dependencies(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    value: &Value,
) -> Vec<(String, String, Value)> {
    let mut dependencies = Vec::new();
    let mut seen = BTreeSet::new();
    collect_expression_dependencies(
        document,
        state,
        current_node_id,
        value,
        &mut dependencies,
        &mut seen,
    );
    dependencies
}

fn collect_expression_dependencies(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    value: &Value,
    dependencies: &mut Vec<(String, String, Value)>,
    seen: &mut BTreeSet<(String, String)>,
) {
    match value {
        Value::String(text) if text.trim_start().starts_with('=') => {
            collect_expression_dependencies_from_text(
                document,
                state,
                current_node_id,
                text.trim_start_matches('=').trim(),
                dependencies,
                seen,
            );
        }
        Value::Array(entries) => {
            for entry in entries {
                collect_expression_dependencies(
                    document,
                    state,
                    current_node_id,
                    entry,
                    dependencies,
                    seen,
                );
            }
        }
        Value::Table(entries) => {
            for entry in entries.values() {
                collect_expression_dependencies(
                    document,
                    state,
                    current_node_id,
                    entry,
                    dependencies,
                    seen,
                );
            }
        }
        _ => {}
    }
}

fn collect_expression_dependencies_from_text(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    expression: &str,
    dependencies: &mut Vec<(String, String, Value)>,
    seen: &mut BTreeSet<(String, String)>,
) {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return;
    }

    if let Some((function_name, args)) = parse_function_expression(trimmed) {
        if let Some((target_node_id, target_path, target_value)) =
            resolve_function_dependency(document, state, current_node_id, &function_name, &args)
        {
            push_dependency(
                dependencies,
                seen,
                target_node_id,
                target_path,
                target_value,
            );
            return;
        }
        for argument in args {
            collect_expression_argument_dependencies(
                document,
                state,
                current_node_id,
                &argument,
                dependencies,
                seen,
            );
        }
        return;
    }

    if let Some((target_node_id, target_path, target_value)) =
        resolve_reference_dependency(document, state, current_node_id, trimmed)
    {
        push_dependency(
            dependencies,
            seen,
            target_node_id,
            target_path,
            target_value,
        );
        return;
    }
}

fn collect_expression_argument_dependencies(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    argument: &str,
    dependencies: &mut Vec<(String, String, Value)>,
    seen: &mut BTreeSet<(String, String)>,
) {
    let trimmed = argument.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Some(expression) = trimmed.strip_prefix('=') {
        collect_expression_dependencies_from_text(
            document,
            state,
            current_node_id,
            expression.trim(),
            dependencies,
            seen,
        );
        return;
    }
    if let Some((target_node_id, target_path, target_value)) =
        resolve_reference_dependency(document, state, current_node_id, trimmed)
    {
        push_dependency(
            dependencies,
            seen,
            target_node_id,
            target_path,
            target_value,
        );
        return;
    }
    if parse_function_expression(trimmed).is_some() {
        collect_expression_dependencies_from_text(
            document,
            state,
            current_node_id,
            trimmed,
            dependencies,
            seen,
        );
        return;
    }
    let Some(literal) = parse_expression_literal(trimmed) else {
        return;
    };
    collect_expression_dependencies(
        document,
        state,
        current_node_id,
        &literal,
        dependencies,
        seen,
    );
}

fn resolve_reference_dependency(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    reference: &str,
) -> Option<(String, String, Value)> {
    mock_expression::parse_preview_mock_reference(reference)?;
    let wrapped = Value::String(format!("={reference}"));
    resolve_preview_mock_expression(document, state, current_node_id, &wrapped).map(
        |(target_node_id, target_path, target_value)| {
            (
                target_node_id.to_string(),
                target_path,
                target_value.clone(),
            )
        },
    )
}

fn push_dependency(
    dependencies: &mut Vec<(String, String, Value)>,
    seen: &mut BTreeSet<(String, String)>,
    target_node_id: String,
    target_path: String,
    target_value: Value,
) {
    let key = (target_node_id.clone(), target_path.clone());
    if !seen.insert(key) {
        return;
    }
    dependencies.push((target_node_id, target_path, target_value));
}

fn evaluate_function_expression(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    function_name: &str,
    args: &[String],
) -> Option<Value> {
    let evaluated_args = args
        .iter()
        .map(|arg| evaluate_expression_argument(document, state, current_node_id, arg))
        .collect::<Option<Vec<_>>>()?;

    match function_name {
        "concat" => Some(Value::String(
            evaluated_args
                .iter()
                .map(expression_value_to_text)
                .collect::<Vec<_>>()
                .join(""),
        )),
        "coalesce" => evaluated_args
            .into_iter()
            .find(is_meaningful_expression_value),
        "first" => evaluated_args
            .into_iter()
            .next()
            .and_then(expression_value_first),
        "last" => evaluated_args
            .into_iter()
            .next()
            .and_then(expression_value_last),
        "join" => {
            let collection = evaluated_args.first()?;
            let separator = evaluated_args
                .get(1)
                .map(expression_value_to_text)
                .unwrap_or_default();
            expression_value_join(collection, &separator).map(Value::String)
        }
        "eq" => {
            let lhs = evaluated_args.first()?;
            let rhs = evaluated_args.get(1)?;
            Some(Value::Boolean(lhs == rhs))
        }
        "if" => {
            let condition = evaluated_args.first()?;
            let when_true = evaluated_args.get(1)?;
            let when_false = evaluated_args.get(2)?;
            Some(if expression_value_truthy(condition) {
                when_true.clone()
            } else {
                when_false.clone()
            })
        }
        "get" | "at" => {
            let target = evaluated_args.first()?;
            expression_value_access(target, &evaluated_args[1..])
        }
        "has" => Some(Value::Boolean(
            evaluated_args
                .first()
                .and_then(|target| expression_value_access(target, &evaluated_args[1..]))
                .is_some(),
        )),
        "count" | "len" => evaluated_args.first().map(expression_value_length),
        _ => None,
    }
}

fn evaluate_expression_argument(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    argument: &str,
) -> Option<Value> {
    let trimmed = argument.trim();
    if trimmed.is_empty() {
        return Some(Value::String(String::new()));
    }
    if trimmed.starts_with('=') {
        return resolve_expression_value(document, state, current_node_id, trimmed);
    }
    if parse_function_expression(trimmed).is_some() {
        return evaluate_function_expression_from_text(document, state, current_node_id, trimmed);
    }
    if let Some(reference) = resolve_reference_expression(document, state, current_node_id, trimmed)
    {
        return Some(reference);
    }
    parse_expression_literal(trimmed)
}

fn evaluate_function_expression_from_text(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    expression: &str,
) -> Option<Value> {
    let (function_name, args) = parse_function_expression(expression)?;
    evaluate_function_expression(document, state, current_node_id, &function_name, &args)
}

fn parse_function_expression(expression: &str) -> Option<(String, Vec<String>)> {
    let trimmed = expression.trim();
    let open = trimmed.find('(')?;
    let close = trimmed.rfind(')')?;
    if close <= open || close != trimmed.len() - 1 {
        return None;
    }
    let function_name = trimmed[..open].trim();
    if function_name.is_empty()
        || !function_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }
    let args = split_function_arguments(&trimmed[(open + 1)..close])?;
    Some((function_name.to_ascii_lowercase(), args))
}

fn split_function_arguments(arguments: &str) -> Option<Vec<String>> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut quote = None::<char>;
    let mut escape = false;

    for ch in arguments.chars() {
        if let Some(active_quote) = quote {
            current.push(ch);
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' && active_quote == '"' {
                escape = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth = paren_depth.checked_sub(1)?;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                current.push(ch);
            }
            '{' => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' => {
                brace_depth = brace_depth.checked_sub(1)?;
                current.push(ch);
            }
            ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                items.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 || brace_depth != 0 {
        return None;
    }

    let tail = current.trim();
    if !tail.is_empty() {
        items.push(tail.to_string());
    } else if !arguments.trim().is_empty() && arguments.trim_end().ends_with(',') {
        return None;
    }

    Some(items)
}

fn parse_expression_literal(value: &str) -> Option<Value> {
    let table = format!("value = {value}").parse::<toml::Table>().ok()?;
    table.get("value").cloned()
}

fn expression_value_to_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        _ => preview_mock_inline_literal(value),
    }
}

fn is_meaningful_expression_value(value: &Value) -> bool {
    match value {
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Table(entries) => !entries.is_empty(),
        _ => true,
    }
}

fn expression_value_length(value: &Value) -> Value {
    let length = match value {
        Value::String(text) => text.chars().count(),
        Value::Array(items) => items.len(),
        Value::Table(entries) => entries.len(),
        _ => 0,
    };
    Value::Integer(length as i64)
}

fn expression_value_first(value: Value) -> Option<Value> {
    match value {
        Value::Array(items) => items.into_iter().next(),
        Value::String(text) => text.chars().next().map(|ch| Value::String(ch.to_string())),
        _ => None,
    }
}

fn expression_value_last(value: Value) -> Option<Value> {
    match value {
        Value::Array(items) => items.into_iter().last(),
        Value::String(text) => text.chars().last().map(|ch| Value::String(ch.to_string())),
        _ => None,
    }
}

fn expression_value_join(value: &Value, separator: &str) -> Option<String> {
    match value {
        Value::Array(items) => Some(
            items
                .iter()
                .map(expression_value_to_text)
                .collect::<Vec<_>>()
                .join(separator),
        ),
        Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

fn expression_value_truthy(value: &Value) -> bool {
    match value {
        Value::Boolean(value) => *value,
        Value::Integer(value) => *value != 0,
        Value::Float(value) => *value != 0.0,
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Table(entries) => !entries.is_empty(),
        _ => false,
    }
}

fn resolve_function_dependency(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    function_name: &str,
    args: &[String],
) -> Option<(String, String, Value)> {
    if !matches!(function_name, "get" | "at" | "has") {
        return None;
    }
    let reference = args.first()?.trim();
    let (target_node_id, mut target_path, target_value) =
        resolve_reference_dependency(document, state, current_node_id, reference)?;
    let mut current_value = target_value.clone();
    for argument in args.iter().skip(1) {
        let segment = expression_segment_from_argument(argument)?;
        current_value = access_value_segment(&current_value, &segment)?;
        mock_expression::append_expression_path_segment(&mut target_path, &segment);
    }
    Some((target_node_id, target_path, current_value))
}

fn expression_value_access(target: &Value, segments: &[Value]) -> Option<Value> {
    let mut current = target.clone();
    for segment in segments {
        let segment = expression_segment_from_value(segment)?;
        current = access_value_segment(&current, &segment)?;
    }
    Some(current)
}

fn access_value_segment(value: &Value, segment: &str) -> Option<Value> {
    match value {
        Value::Table(entries) => entries.get(segment).cloned(),
        Value::Array(entries) => entries.get(segment.parse::<usize>().ok()?).cloned(),
        Value::String(text) => text
            .chars()
            .nth(segment.parse::<usize>().ok()?)
            .map(|ch| Value::String(ch.to_string())),
        _ => None,
    }
}

fn expression_segment_from_argument(argument: &str) -> Option<String> {
    let literal = parse_expression_literal(argument.trim())?;
    expression_segment_from_value(&literal)
}

fn expression_segment_from_value(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Integer(value) => Some(value.to_string()),
        _ => None,
    }
}

pub(super) fn build_preview_binding_graph_items(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
) -> Vec<String> {
    let mut items = Vec::new();
    for node in document.iter_nodes() {
        let node_id = node.node_id.as_str();
        let source_node_label = node.control_id.as_deref().unwrap_or(node_id);
        for binding in &node.bindings {
            let binding_source = format!("{source_node_label}.{}", binding.event.native_name());
            if let Some(target) = binding_graph_target(binding) {
                items.push(format!("{binding_source} => {target}"));
            }
            if let Some(action) = binding.action.as_ref() {
                for (payload_key, payload_value) in &action.payload {
                    collect_binding_payload_expression_graph_items(
                        document,
                        state,
                        node_id,
                        &format!("{binding_source}.payload.{payload_key}"),
                        payload_value,
                        &mut items,
                    );
                }
            }
        }
    }
    items
}

fn collect_binding_payload_expression_graph_items(
    document: &UiAssetDocument,
    state: &UiAssetPreviewMockState,
    current_node_id: &str,
    source_path: &str,
    value: &Value,
    items: &mut Vec<String>,
) {
    match value {
        Value::String(text) if text.trim_start().starts_with('=') => {
            for (target_node_id, target_path, target_value) in
                collect_preview_mock_expression_dependencies(
                    document,
                    state,
                    current_node_id,
                    value,
                )
            {
                let Some(target_node) = document.node(&target_node_id) else {
                    continue;
                };
                let target_key =
                    preview_mock_display_key(target_node, &target_node_id, &target_path, true);
                items.push(format!(
                    "{source_path} -> {target_key} = {}",
                    preview_mock_inline_literal(&target_value)
                ));
            }
        }
        Value::Array(entries) => {
            for (index, entry) in entries.iter().enumerate() {
                collect_binding_payload_expression_graph_items(
                    document,
                    state,
                    current_node_id,
                    &format!("{source_path}[{index}]"),
                    entry,
                    items,
                );
            }
        }
        Value::Table(entries) => {
            let mut keys = entries.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                let Some(entry) = entries.get(&key) else {
                    continue;
                };
                collect_binding_payload_expression_graph_items(
                    document,
                    state,
                    current_node_id,
                    &format!("{source_path}.{key}"),
                    entry,
                    items,
                );
            }
        }
        _ => {}
    }
}

fn binding_graph_target(binding: &UiBindingRef) -> Option<String> {
    binding
        .action
        .as_ref()
        .and_then(|action| action.action.clone())
        .or_else(|| binding.route.clone())
        .or_else(|| {
            binding
                .action
                .as_ref()
                .and_then(|action| action.route.clone())
        })
        .filter(|target| !target.trim().is_empty())
}
