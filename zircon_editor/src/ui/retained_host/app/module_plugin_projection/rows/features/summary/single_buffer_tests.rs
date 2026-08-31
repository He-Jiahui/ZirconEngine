use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::project::ExportPackagingStrategy;

use super::{module_plugin_feature_state, module_plugin_optional_feature_summary};
use crate::ui::host::{EditorPluginFeatureDependencyStatus, EditorPluginFeatureStatus};

const SAMPLE_PAIRS: usize = 21;
const SUMMARIES_PER_SAMPLE: usize = 512;
const FEATURE_COUNT: usize = 32;
const DEPENDENCIES_PER_FEATURE: usize = 4;

#[test]
fn optimization_batch_20260826dg_editor96_feature_summary_preserves_text_contract() {
    let features = vec![
        feature(0, true, true, 2),
        feature(1, false, true, 0),
        feature(2, true, false, 1),
    ];

    assert_eq!(
        module_plugin_optional_feature_summary(&features),
        "Feature 0 [enabled] deps: primary plugin.0.0:capability.0 (ok); plugin.0.1:capability.1 (missing plugin)\nFeature 1 [ready]\nFeature 2 [blocked] deps: primary plugin.2.0:capability.0 (ok)"
    );
}

#[test]
fn optimization_batch_20260826dg_editor96_feature_summary_uses_exact_single_buffer() {
    let features = fixture_features();
    let summary = module_plugin_optional_feature_summary(&features);
    assert_eq!(summary.len(), summary.capacity());

    let source = include_str!("../summary.rs");
    assert!(
        source.contains("String::with_capacity(module_plugin_feature_summary_capacity(features))")
    );
    assert!(source.contains("push_module_plugin_feature_dependencies"));
    assert!(!source.contains("collect::<Vec<_>>()"));
    assert!(!source.contains(".join(\"\\n\")"));
    assert!(!source.contains(".join(\"; \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dg_editor96_feature_summary_single_buffer_bench() {
    let features = fixture_features();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&features, legacy_summary));
            optimized_samples.push(measure(&features, module_plugin_optional_feature_summary));
        } else {
            optimized_samples.push(measure(&features, module_plugin_optional_feature_summary));
            legacy_samples.push(measure(&features, legacy_summary));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR96_FEATURE_SUMMARY_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
summaries_per_sample={SUMMARIES_PER_SAMPLE} features_per_summary={FEATURE_COUNT} \
dependencies_per_feature={DEPENDENCIES_PER_FEATURE} legacy_allocations_per_summary=226 \
optimized_allocations_per_summary=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer feature summary P95 {optimized_p95_ns}ns must be at most 70% of nested collectors P95 {legacy_p95_ns}ns"
    );
}

fn fixture_features() -> Vec<EditorPluginFeatureStatus> {
    (0..FEATURE_COUNT)
        .map(|index| {
            feature(
                index,
                index % 2 == 0,
                index % 3 != 0,
                DEPENDENCIES_PER_FEATURE,
            )
        })
        .collect()
}

fn feature(
    index: usize,
    enabled: bool,
    available: bool,
    dependency_count: usize,
) -> EditorPluginFeatureStatus {
    EditorPluginFeatureStatus {
        id: format!("feature-{index}"),
        display_name: format!("Feature {index}"),
        owner_plugin_id: "owner.plugin".to_string(),
        enabled,
        required: false,
        available,
        target_modes: Vec::new(),
        packaging: ExportPackagingStrategy::SourceTemplate,
        runtime_crate: None,
        editor_crate: None,
        provided_capabilities: Vec::new(),
        dependencies: (0..dependency_count)
            .map(|dependency| EditorPluginFeatureDependencyStatus {
                plugin_id: format!("plugin.{index}.{dependency}"),
                capability: format!("capability.{dependency}"),
                primary: dependency == 0,
                plugin_enabled: dependency % 3 != 1,
                capability_available: dependency % 3 != 2,
            })
            .collect(),
        diagnostics: Vec::new(),
    }
}

fn legacy_summary(features: &[EditorPluginFeatureStatus]) -> String {
    features
        .iter()
        .map(|feature| {
            let state = module_plugin_feature_state(feature);
            let dependencies = legacy_dependency_summary(feature);
            if dependencies.is_empty() {
                format!("{} [{state}]", feature.display_name)
            } else {
                format!("{} [{state}] deps: {dependencies}", feature.display_name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn legacy_dependency_summary(feature: &EditorPluginFeatureStatus) -> String {
    feature
        .dependencies
        .iter()
        .map(|dependency| {
            let state = match (dependency.plugin_enabled, dependency.capability_available) {
                (true, true) => "ok",
                (false, _) => "missing plugin",
                (true, false) => "missing capability",
            };
            let role = if dependency.primary { "primary " } else { "" };
            format!(
                "{role}{}:{} ({state})",
                dependency.plugin_id, dependency.capability
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn measure(
    features: &[EditorPluginFeatureStatus],
    summarize: fn(&[EditorPluginFeatureStatus]) -> String,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SUMMARIES_PER_SAMPLE {
        checksum ^= black_box(summarize(black_box(features))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
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
