use std::collections::{BTreeMap, HashSet};

use crate::ui::asset_editor::preview::preview_mock::{
    format_preview_mock_inline_value, resolve_preview_mock_value_preview, UiAssetPreviewMockState,
};
use crate::ui::template::EditorTemplateRuntimeService;
use toml::Value;
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

    let mut projected_payload_keys = HashSet::new();
    for (key, value) in super::binding_payload_entries(binding) {
        append_binding_value_projection(
            &mut items,
            document,
            preview_mock_state,
            current_node_id,
            &format!("payload.{key}"),
            value,
            None,
        );
        let _ = projected_payload_keys.insert(key.clone());
    }

    for (key, value) in super::binding_schema_payload_entries(binding) {
        if projected_payload_keys.contains(&key) {
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
        let _ = projected_payload_keys.insert(key);
    }

    items
}

fn binding_diagnostics_by_path(
    document: &UiAssetDocument,
    current_node_id: &str,
    binding_id: &str,
) -> BTreeMap<String, Vec<UiBindingDiagnostic>> {
    EditorTemplateRuntimeService
        .collect_binding_report(document)
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
    let nested_target_prefix = format!("{target_suffix}.");
    for (path, diagnostics) in diagnostics_by_path {
        if path.ends_with(&target_suffix) || path.contains(&nested_target_prefix) {
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
            let mut sorted_entries = entries.iter().collect::<Vec<_>>();
            sorted_entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (key, entry) in sorted_entries {
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
            let mut sorted_entries = entries.iter().collect::<Vec<_>>();
            sorted_entries.sort_by(|(left, _), (right, _)| left.cmp(right));
            for (key, entry) in sorted_entries {
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

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const PAYLOAD_KEY_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_PAYLOAD_KEY_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn payload_keys() -> Vec<String> {
        (0..PAYLOAD_KEY_ADMISSION_COUNT)
            .map(|index| {
                format!(
                    "payload.key.{:04}",
                    (index * 4_099) % UNIQUE_PAYLOAD_KEY_COUNT
                )
            })
            .collect()
    }

    fn ordered_payload_key_count(payload_keys: &[String]) -> usize {
        let mut projected_payload_keys = BTreeSet::new();
        payload_keys
            .iter()
            .filter(|key| projected_payload_keys.insert((*key).clone()))
            .count()
    }

    fn hash_payload_key_count(payload_keys: &[String]) -> usize {
        let mut projected_payload_keys = HashSet::new();
        payload_keys
            .iter()
            .filter(|key| projected_payload_keys.insert((*key).clone()))
            .count()
    }

    #[test]
    fn optimization_batch_20260826u_editor23_hash_payload_membership_matches_ordered_membership() {
        let payload_keys = payload_keys();

        assert_eq!(
            ordered_payload_key_count(&payload_keys),
            hash_payload_key_count(&payload_keys)
        );
        assert_eq!(
            hash_payload_key_count(&payload_keys),
            UNIQUE_PAYLOAD_KEY_COUNT
        );
    }

    #[test]
    fn optimization_batch_20260826u_editor23_payload_projection_uses_hash_membership() {
        let source = include_str!("schema_projection.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
        assert!(production.contains("let mut projected_payload_keys = HashSet::new();"));
        assert!(production.contains("BTreeMap<String, Vec<UiBindingDiagnostic>>"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826u_editor23_payload_key_hash_membership_performance_evidence() {
        let payload_keys = payload_keys();
        assert_eq!(
            ordered_payload_key_count(&payload_keys),
            hash_payload_key_count(&payload_keys)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_payload_key_count(black_box(&payload_keys)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_payload_key_count(black_box(&payload_keys)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_payload_key_count(black_box(&payload_keys)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_payload_key_count(black_box(&payload_keys)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "EDITOR23_PAYLOAD_KEY_HASH_MEMBERSHIP_BENCH_V1 admissions={PAYLOAD_KEY_ADMISSION_COUNT} \
             unique_payload_keys={UNIQUE_PAYLOAD_KEY_COUNT} ordered_lookup_class=log_n \
             hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-membership P95 {:?} exceeded 60% of ordered-membership P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
