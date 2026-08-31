use std::{hint::black_box, time::Instant};

use zircon_runtime_interface::ui::template::{
    UiActionHostPolicy, UiActionPolicyDiagnostic, UiActionPolicyDiagnosticSeverity,
    UiActionPolicyReport, UiActionSideEffectClass,
};

use super::{action_binding_label, unsafe_action_guidance_items};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826al_action_policy_guidance_hash_index_preserves_rows() {
    let runtime_policy = UiActionHostPolicy::runtime_default();
    let editor_policy = UiActionHostPolicy::editor_authoring();
    let runtime_report = report([
        diagnostic("node-a", "asset-save", UiActionSideEffectClass::AssetIo),
        diagnostic("node-b", "network-fetch", UiActionSideEffectClass::Network),
        diagnostic(
            "node-c",
            "process-run",
            UiActionSideEffectClass::ExternalProcess,
        ),
    ]);
    let editor_report = report([
        diagnostic("node-b", "network-fetch", UiActionSideEffectClass::Network),
        diagnostic("node-b", "network-fetch", UiActionSideEffectClass::Network),
    ]);

    let expected = legacy_unsafe_action_guidance_items(
        &runtime_policy,
        &runtime_report,
        &editor_policy,
        &editor_report,
    );
    let actual = unsafe_action_guidance_items(
        &runtime_policy,
        &runtime_report,
        &editor_policy,
        &editor_report,
    );

    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 4);
    assert!(actual[2].contains("editor-only AssetIo"));
    assert!(actual[3].contains("process-run"));
}

#[test]
fn optimization_batch_20260826al_action_policy_guidance_uses_borrowed_hash_index() {
    let source = include_str!("../runtime_report_state.rs");

    assert!(source.contains("HashSet<(&str, &str)>") || source.contains("HashSet<_>"));
    assert!(source.contains("editor_diagnostic_keys.contains"));
    assert!(!source.contains("editor_report.diagnostics.iter().any"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826al_action_policy_guidance_hash_index_p95() {
    const RUNTIME_DIAGNOSTICS: usize = 4_096;
    let runtime_policy = UiActionHostPolicy::runtime_default();
    let editor_policy = UiActionHostPolicy::editor_authoring();
    let runtime_report = report((0..RUNTIME_DIAGNOSTICS).map(|index| {
        diagnostic(
            &format!("node-{index}"),
            &format!("binding-{index}"),
            UiActionSideEffectClass::Network,
        )
    }));
    let editor_report = report((0..RUNTIME_DIAGNOSTICS).step_by(2).map(|index| {
        diagnostic(
            &format!("node-{index}"),
            &format!("binding-{index}"),
            UiActionSideEffectClass::Network,
        )
    }));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(|| {
                legacy_unsafe_action_guidance_items(
                    &runtime_policy,
                    &runtime_report,
                    &editor_policy,
                    &editor_report,
                )
            }));
            optimized_ns.push(measure_ns(|| {
                unsafe_action_guidance_items(
                    &runtime_policy,
                    &runtime_report,
                    &editor_policy,
                    &editor_report,
                )
            }));
        } else {
            optimized_ns.push(measure_ns(|| {
                unsafe_action_guidance_items(
                    &runtime_policy,
                    &runtime_report,
                    &editor_policy,
                    &editor_report,
                )
            }));
            legacy_ns.push(measure_ns(|| {
                legacy_unsafe_action_guidance_items(
                    &runtime_policy,
                    &runtime_report,
                    &editor_policy,
                    &editor_report,
                )
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
        "diagnostic hash-index P95 must be at least 80% below nested matching: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_ACTION_POLICY_GUIDANCE_HASH_INDEX_BENCH_V1 runtime_diagnostics={RUNTIME_DIAGNOSTICS} editor_diagnostics={} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_worst_case_pair_checks={} optimized_hash_probes={RUNTIME_DIAGNOSTICS} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        RUNTIME_DIAGNOSTICS / 2,
        RUNTIME_DIAGNOSTICS * (RUNTIME_DIAGNOSTICS / 2),
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn report(diagnostics: impl IntoIterator<Item = UiActionPolicyDiagnostic>) -> UiActionPolicyReport {
    UiActionPolicyReport {
        diagnostics: diagnostics.into_iter().collect(),
    }
}

fn diagnostic(
    node_id: &str,
    binding_id: &str,
    side_effect: UiActionSideEffectClass,
) -> UiActionPolicyDiagnostic {
    UiActionPolicyDiagnostic {
        severity: UiActionPolicyDiagnosticSeverity::Error,
        node_id: node_id.to_string(),
        binding_id: binding_id.to_string(),
        route: None,
        action: None,
        side_effect,
        message: String::new(),
    }
}

fn legacy_unsafe_action_guidance_items(
    runtime_policy: &UiActionHostPolicy,
    runtime_report: &UiActionPolicyReport,
    editor_policy: &UiActionHostPolicy,
    editor_report: &UiActionPolicyReport,
) -> Vec<String> {
    let mut items = Vec::new();
    for diagnostic in &editor_report.diagnostics {
        items.push(format!(
            "editor-authoring binding {} uses {:?}; explicit host capability required before authoring or packaging. Move unsafe work behind an approved host service.",
            action_binding_label(diagnostic),
            diagnostic.side_effect
        ));
    }
    for diagnostic in &runtime_report.diagnostics {
        if editor_policy.allows(diagnostic.side_effect) {
            items.push(format!(
                "runtime-default binding {} is editor-only {:?}; keep it in editor profile or replace it with a LocalUi runtime action.",
                action_binding_label(diagnostic),
                diagnostic.side_effect
            ));
        } else if !runtime_policy.allows(diagnostic.side_effect)
            && !editor_report.diagnostics.iter().any(|editor_diagnostic| {
                editor_diagnostic.node_id == diagnostic.node_id
                    && editor_diagnostic.binding_id == diagnostic.binding_id
            })
        {
            items.push(format!(
                "runtime-default binding {} uses {:?}; explicit host capability required before runtime packaging.",
                action_binding_label(diagnostic),
                diagnostic.side_effect
            ));
        }
    }
    if items.is_empty() {
        items.push("all action bindings are compatible with runtime-default and editor-authoring host policies".to_string());
    }
    items
}

fn measure_ns(operation: impl FnOnce() -> Vec<String>) -> u128 {
    let started = Instant::now();
    let items = black_box(operation)();
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(black_box(items.len()), 4_096);
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
