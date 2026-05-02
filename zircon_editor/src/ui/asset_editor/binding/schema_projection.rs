use std::collections::{BTreeMap, BTreeSet};

use crate::ui::asset_editor::preview::preview_mock::{
    format_preview_mock_inline_value, resolve_preview_mock_value_preview, UiAssetPreviewMockState,
};
use toml::Value;
use zircon_runtime::ui::template::{collect_asset_binding_report, UiDocumentCompiler};
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiBindingDiagnostic, UiBindingRef, UiBindingTarget, UiBindingTargetKind,
};

pub(super) fn build_binding_schema_items(
    document: &UiAssetDocument,
    current_node_id: &str,
    preview_mock_state: &UiAssetPreviewMockState,
    binding: &UiBindingRef,
) -> Vec<String> {
    let mut items = vec![format!("event [UiEvent] = {}", binding.event.native_name())];
    match super::binding_action_kind(binding) {
        super::UiBindingActionKind::Route => {
            items.push(format!(
                "route.target [Route] = {}",
                super::binding_route_target(binding)
            ));
        }
        super::UiBindingActionKind::Action => {
            items.push(format!(
                "action.target [EditorAction] = {}",
                super::binding_action_specific_target(binding)
            ));
        }
        super::UiBindingActionKind::None => {
            items.push("action.kind [None]".to_string());
        }
    }

    let diagnostics_by_path = binding_diagnostics_by_path(document, current_node_id, &binding.id);
    for (index, assignment) in binding.targets.iter().enumerate() {
        items.push(format!(
            "target[{index}] [{}] = {}",
            binding_target_label(&assignment.target),
            assignment.expression
        ));
        append_target_diagnostics(&mut items, &diagnostics_by_path, index);
    }

    let mut projected_payload_keys = BTreeSet::new();
    for (key, value) in super::binding_payload_entries(binding) {
        let _ = projected_payload_keys.insert(key.clone());
        append_binding_value_projection(
            &mut items,
            document,
            preview_mock_state,
            current_node_id,
            &format!("payload.{key}"),
            &value,
            None,
        );
    }

    for (key, value) in super::binding_schema_payload_entries(binding) {
        if !projected_payload_keys.insert(key.clone()) {
            continue;
        }
        append_binding_value_projection(
            &mut items,
            document,
            preview_mock_state,
            current_node_id,
            &format!("payload.{key}"),
            &value,
            Some("default"),
        );
    }

    items
}

fn binding_diagnostics_by_path(
    document: &UiAssetDocument,
    current_node_id: &str,
    binding_id: &str,
) -> BTreeMap<String, Vec<UiBindingDiagnostic>> {
    collect_asset_binding_report(document, UiDocumentCompiler::default().component_registry())
        .diagnostics
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.node_id == current_node_id && diagnostic.binding_id == binding_id
        })
        .fold(BTreeMap::new(), |mut by_path, diagnostic| {
            by_path
                .entry(diagnostic.path.clone())
                .or_default()
                .push(diagnostic);
            by_path
        })
}

fn append_target_diagnostics(
    items: &mut Vec<String>,
    diagnostics_by_path: &BTreeMap<String, Vec<UiBindingDiagnostic>>,
    target_index: usize,
) {
    let target_suffix = format!(".targets[{target_index}]");
    for (path, diagnostics) in diagnostics_by_path {
        if path.ends_with(&target_suffix) || path.contains(&format!("{target_suffix}.")) {
            for diagnostic in diagnostics {
                items.push(format!(
                    "target diagnostic [{}] {}: {}",
                    diagnostic.code.as_str(),
                    diagnostic.path,
                    diagnostic.message
                ));
            }
        }
    }
}

fn binding_target_label(target: &UiBindingTarget) -> String {
    let kind = match target.kind {
        UiBindingTargetKind::Prop => "prop",
        UiBindingTargetKind::Class => "class",
        UiBindingTargetKind::Visibility => "visibility",
        UiBindingTargetKind::Enabled => "enabled",
        UiBindingTargetKind::ActionPayload => "action_payload",
    };
    target
        .name
        .as_deref()
        .map(|name| format!("{kind}.{name}"))
        .unwrap_or_else(|| kind.to_string())
}

fn append_binding_value_projection(
    items: &mut Vec<String>,
    document: &UiAssetDocument,
    preview_mock_state: &UiAssetPreviewMockState,
    current_node_id: &str,
    path: &str,
    value: &Value,
    suffix: Option<&str>,
) {
    let suffix = suffix
        .map(|suffix| format!(" {suffix}"))
        .unwrap_or_default();
    items.push(format!(
        "{path} [{}]{suffix} = {}",
        super::binding_value_kind_label(value),
        binding_schema_default_literal(value)
    ));

    if suffix.is_empty() {
        if let Some(preview_value) =
            resolve_preview_mock_value_preview(document, preview_mock_state, current_node_id, value)
        {
            items.push(format!(
                "{path}.preview [{}] = {}",
                super::binding_value_kind_label(&preview_value),
                format_preview_mock_inline_value(&preview_value)
            ));
        }
    }

    match value {
        Value::Array(entries) => {
            for (index, entry) in entries.iter().enumerate() {
                append_binding_value_projection(
                    items,
                    document,
                    preview_mock_state,
                    current_node_id,
                    &format!("{path}[{index}]"),
                    entry,
                    suffix_label(suffix.as_str()),
                );
            }
            if let Some(template) = entries.first() {
                append_binding_template_projection(
                    items,
                    document,
                    preview_mock_state,
                    current_node_id,
                    &format!("{path}[n]"),
                    template,
                    suffix_label(suffix.as_str()),
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
                append_binding_value_projection(
                    items,
                    document,
                    preview_mock_state,
                    current_node_id,
                    &format!("{path}.{key}"),
                    entry,
                    suffix_label(suffix.as_str()),
                );
            }
        }
        _ => {}
    }
}

fn append_binding_template_projection(
    items: &mut Vec<String>,
    document: &UiAssetDocument,
    preview_mock_state: &UiAssetPreviewMockState,
    current_node_id: &str,
    path: &str,
    value: &Value,
    suffix: Option<&str>,
) {
    let suffix = suffix
        .map(|suffix| format!(" {suffix}"))
        .unwrap_or_default();
    items.push(format!(
        "{path} [{}]{suffix} = {}",
        super::binding_value_kind_label(value),
        binding_schema_default_literal(value)
    ));

    if suffix.is_empty() {
        if let Some(preview_value) =
            resolve_preview_mock_value_preview(document, preview_mock_state, current_node_id, value)
        {
            items.push(format!(
                "{path}.preview [{}] = {}",
                super::binding_value_kind_label(&preview_value),
                format_preview_mock_inline_value(&preview_value)
            ));
        }
    }

    match value {
        Value::Array(entries) => {
            if let Some(template) = entries.first() {
                append_binding_template_projection(
                    items,
                    document,
                    preview_mock_state,
                    current_node_id,
                    &format!("{path}[n]"),
                    template,
                    suffix_label(suffix.as_str()),
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
                append_binding_template_projection(
                    items,
                    document,
                    preview_mock_state,
                    current_node_id,
                    &format!("{path}.{key}"),
                    entry,
                    suffix_label(suffix.as_str()),
                );
            }
        }
        _ => {}
    }
}

fn suffix_label(suffix: &str) -> Option<&str> {
    if suffix.is_empty() {
        None
    } else {
        Some("default")
    }
}

fn binding_schema_default_literal(value: &Value) -> String {
    match value {
        Value::String(text) => Value::String(text.clone()).to_string(),
        _ => value.to_string(),
    }
}
