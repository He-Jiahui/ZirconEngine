use std::hint::black_box;
use std::time::Instant;

use super::{module_plugin_feature_action, EditorPluginFeatureStatus};
use crate::ui::host::EditorPluginFeatureDependencyStatus;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

const SAMPLE_PAIRS: usize = 31;
const FEATURE_COUNT: usize = 2_048;
const ACTIONS_PER_SAMPLE: usize = 1_000;

#[test]
fn optimization_batch_20260829ac_editor248_feature_action_preserves_priority_and_order() {
    let disable = feature("disable.first", true, false, true);
    let ready = feature("ready.first", false, false, true);
    let blocked = feature("blocked.first", false, false, false);
    assert_eq!(
        module_plugin_feature_action(&[disable.clone(), ready.clone(), blocked]),
        (
            "Enable Deps".to_string(),
            "workbench.plugin.feature.enable_dependencies.owner.blocked.first".to_string()
        )
    );
    assert_eq!(
        module_plugin_feature_action(&[disable.clone(), ready]),
        (
            "Enable Feature".to_string(),
            "workbench.plugin.feature.enable.owner.ready.first".to_string()
        )
    );
    assert_eq!(
        module_plugin_feature_action(&[disable]),
        (
            "Disable Feature".to_string(),
            "workbench.plugin.feature.disable.owner.disable.first".to_string()
        )
    );
}

#[test]
fn optimization_batch_20260829ac_editor248_feature_action_uses_one_scan() {
    let source = include_str!("../action.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let action = implementation
        .split("fn module_plugin_feature_action")
        .nth(1)
        .and_then(|body| body.split("fn module_plugin_feature_action_id").next())
        .expect("feature action selector");

    assert!(action.contains("for feature in features"));
    assert!(action.contains("let mut enable_feature = None"));
    assert!(action.contains("let mut disable_feature = None"));
    assert!(!action.contains(".iter()"));
    assert!(!action.contains(".find("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ac_editor248_single_pass_feature_action_bench() {
    let features = benchmark_features();
    assert_eq!(
        module_plugin_feature_action(&features),
        legacy_module_plugin_feature_action(&features)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &features));
            optimized_samples.push(measure(true, &features));
        } else {
            optimized_samples.push(measure(true, &features));
            legacy_samples.push(measure(false, &features));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR248_SINGLE_PASS_FEATURE_ACTION_SELECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
feature_count={} actions_per_sample={ACTIONS_PER_SAMPLE} \
legacy_feature_visits_per_action={} optimized_feature_visits_per_action={} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        features.len(),
        features.len().saturating_mul(3),
        features.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn feature(
    id: impl Into<String>,
    enabled: bool,
    required: bool,
    available: bool,
) -> EditorPluginFeatureStatus {
    EditorPluginFeatureStatus {
        id: id.into(),
        display_name: String::new(),
        owner_plugin_id: "owner".to_string(),
        enabled,
        required,
        available,
        target_modes: Vec::new(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: None,
        editor_crate: None,
        provided_capabilities: Vec::new(),
        dependencies: Vec::<EditorPluginFeatureDependencyStatus>::new(),
        diagnostics: Vec::new(),
    }
}

fn benchmark_features() -> Vec<EditorPluginFeatureStatus> {
    let mut features = Vec::with_capacity(FEATURE_COUNT);
    for index in 0..FEATURE_COUNT.saturating_sub(1) {
        features.push(feature(format!("required.{index}"), true, true, true));
    }
    features.push(feature("disable.last", true, false, true));
    features
}

fn legacy_module_plugin_feature_action(features: &[EditorPluginFeatureStatus]) -> (String, String) {
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && !feature.available)
    {
        return legacy_action("Enable Deps", "enable_dependencies", feature);
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && feature.available)
    {
        return legacy_action("Enable Feature", "enable", feature);
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| feature.enabled && !feature.required)
    {
        return legacy_action("Disable Feature", "disable", feature);
    }
    (String::new(), String::new())
}

fn legacy_action(
    label: &str,
    action: &str,
    feature: &EditorPluginFeatureStatus,
) -> (String, String) {
    (
        label.to_string(),
        format!(
            "workbench.plugin.feature.{action}.{}.{}",
            feature.owner_plugin_id, feature.id
        ),
    )
}

fn measure(optimized: bool, features: &[EditorPluginFeatureStatus]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..ACTIONS_PER_SAMPLE {
        let action = if optimized {
            module_plugin_feature_action(black_box(features))
        } else {
            legacy_module_plugin_feature_action(black_box(features))
        };
        checksum = checksum.wrapping_add(black_box(action).1.len());
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
