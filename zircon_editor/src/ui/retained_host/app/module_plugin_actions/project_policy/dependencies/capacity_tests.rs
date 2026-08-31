use std::hint::black_box;
use std::time::Instant;

use super::{feature_dependency_enable_message, feature_dependency_message_capacity};
use crate::ui::host::EditorPluginFeatureSelectionUpdateReport;
use zircon_runtime::core::framework::project::ProjectPluginSelection;

const SAMPLE_PAIRS: usize = 21;
const MESSAGES_PER_SAMPLE: usize = 1_024;
const ITEMS_PER_GROUP: usize = 64;

#[test]
fn optimization_batch_20260826fs_editor160_direct_render_preserves_dependency_message() {
    let report = report(2);

    let message = feature_dependency_enable_message(&report);

    assert_eq!(
        message,
        "Feature render.hdr dependencies enabled: plugins render.core.0, render.core.1; \
features render.hdr.0, render.hdr.1; diagnostic 0; diagnostic 1"
    );
    assert_eq!(message.len(), feature_dependency_message_capacity(&report));

    let mut already_enabled = report(0);
    already_enabled.diagnostics = vec!["catalog unchanged".to_string()];
    assert_eq!(
        feature_dependency_enable_message(&already_enabled),
        "Feature render.hdr dependencies already enabled: catalog unchanged"
    );
}

#[test]
fn optimization_batch_20260826fs_editor160_dependency_message_avoids_details_and_join() {
    let source = include_str!("../dependencies.rs");
    assert!(source.contains("String::with_capacity(feature_dependency_message_capacity(report))"));
    assert!(source.contains("push_joined(&mut message"));
    assert!(!source.contains("let mut details = Vec::new();"));
    assert!(!source.contains(".join(\"; \")"));
    assert!(!source.contains(".join(\", \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fs_editor160_feature_dependency_message_direct_render_bench() {
    let report = report(ITEMS_PER_GROUP);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&report, false));
            optimized_samples.push(measure(&report, true));
        } else {
            optimized_samples.push(measure(&report, true));
            legacy_samples.push(measure(&report, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR160_FEATURE_DEPENDENCY_MESSAGE_DIRECT_RENDER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
messages_per_sample={MESSAGES_PER_SAMPLE} items_per_group={ITEMS_PER_GROUP} \
legacy_join_allocations_per_message=4 optimized_join_allocations_per_message=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn report(item_count: usize) -> EditorPluginFeatureSelectionUpdateReport {
    EditorPluginFeatureSelectionUpdateReport {
        plugin_id: "render".to_string(),
        feature_id: "render.hdr".to_string(),
        enabled: true,
        project_selection: ProjectPluginSelection::runtime_plugin("render", true, false),
        enabled_dependency_plugins: (0..item_count)
            .map(|index| format!("render.core.{index}"))
            .collect(),
        enabled_dependency_features: (0..item_count)
            .map(|index| format!("render.hdr.{index}"))
            .collect(),
        diagnostics: (0..item_count)
            .map(|index| format!("diagnostic {index}"))
            .collect(),
    }
}

fn measure(report: &EditorPluginFeatureSelectionUpdateReport, direct: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..MESSAGES_PER_SAMPLE {
        let message = if direct {
            feature_dependency_enable_message(black_box(report))
        } else {
            legacy_message(black_box(report))
        };
        checksum ^= black_box(message.len() ^ message.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_message(report: &EditorPluginFeatureSelectionUpdateReport) -> String {
    let mut details = Vec::new();
    if !report.enabled_dependency_plugins.is_empty() {
        details.push(format!(
            "plugins {}",
            report.enabled_dependency_plugins.join(", ")
        ));
    }
    if !report.enabled_dependency_features.is_empty() {
        details.push(format!(
            "features {}",
            report.enabled_dependency_features.join(", ")
        ));
    }
    let mut message = format!(
        "Feature {} dependencies enabled: {}",
        report.feature_id,
        details.join("; ")
    );
    if !report.diagnostics.is_empty() {
        message.push_str("; ");
        message.push_str(&report.diagnostics.join("; "));
    }
    message
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
