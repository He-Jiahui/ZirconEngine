use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::InitLevel;
use zircon_runtime::{
    plugin::PluginFeatureBundleManifest, plugin::PluginFeatureDependency, plugin::PluginModuleKind,
    plugin::PluginModuleManifest,
};

use super::{
    feature_is_available_for_status, feature_manifest_supports_target, feature_status_diagnostics,
    owner_dependency_is_valid, EditorPluginFeatureDependencyStatus, ProjectPluginFeatureSelection,
};

fn benchmark_runtime_module(index: usize) -> PluginModuleManifest {
    PluginModuleManifest {
        name: format!("benchmark.runtime.{index}"),
        description: String::new(),
        kind: PluginModuleKind::Runtime,
        crate_name: format!("benchmark_runtime_{index}"),
        init_level: InitLevel::Post,
        module_dependencies: Vec::new(),
        target_modes: vec![RuntimeTargetMode::ServerRuntime],
        capabilities: Vec::new(),
        system_sets: Vec::new(),
        system_anchors: Vec::new(),
        event_consumers: Vec::new(),
    }
}

fn legacy_feature_manifest_supports_target(
    feature: &PluginFeatureBundleManifest,
    target: RuntimeTargetMode,
) -> bool {
    let runtime_modules = feature
        .modules
        .iter()
        .filter(|module| module.kind == PluginModuleKind::Runtime)
        .collect::<Vec<_>>();
    if runtime_modules.is_empty() {
        return true;
    }
    runtime_modules
        .iter()
        .any(|module| module.target_modes.is_empty() || module.target_modes.contains(&target))
}

fn legacy_owner_dependency_is_valid(feature: &PluginFeatureBundleManifest) -> bool {
    let primary_dependencies = feature
        .dependencies
        .iter()
        .filter(|dependency| dependency.primary)
        .collect::<Vec<_>>();
    primary_dependencies.len() == 1 && primary_dependencies[0].plugin_id == feature.owner_plugin_id
}

#[test]
fn feature_status_rejects_secondary_primary_dependency() {
    let feature =
        PluginFeatureBundleManifest::new("sound.invalid_extra_primary", "Invalid", "sound")
            .with_dependency(PluginFeatureDependency::primary(
                "sound",
                "runtime.plugin.sound",
            ))
            .with_dependency(PluginFeatureDependency::primary(
                "animation",
                "runtime.feature.animation.timeline_event_track",
            ));
    let selection = ProjectPluginFeatureSelection::new("sound.invalid_extra_primary");
    let dependencies = vec![
        EditorPluginFeatureDependencyStatus {
            plugin_id: "sound".to_string(),
            capability: "runtime.plugin.sound".to_string(),
            primary: true,
            plugin_enabled: true,
            capability_available: true,
        },
        EditorPluginFeatureDependencyStatus {
            plugin_id: "animation".to_string(),
            capability: "runtime.feature.animation.timeline_event_track".to_string(),
            primary: true,
            plugin_enabled: true,
            capability_available: true,
        },
    ];
    let available_feature_ids = std::collections::HashSet::new();

    assert!(!feature_is_available_for_status(
        &feature,
        &selection,
        &dependencies,
        &available_feature_ids,
        RuntimeTargetMode::EditorHost,
    ));
    assert!(feature_status_diagnostics(
        &feature,
        &selection,
        &dependencies,
        RuntimeTargetMode::EditorHost,
    )
    .iter()
    .any(|diagnostic| diagnostic.contains("not the only primary dependency")));
}

#[test]
fn optimization_batch_en_feature_checks_stream_without_temporary_vectors() {
    let source = include_str!("../builtin.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("builtin plugin status implementation");
    let target_check = production
        .split("fn feature_manifest_supports_target(")
        .nth(1)
        .expect("target support check")
        .split("fn feature_capabilities_for_target(")
        .next()
        .expect("target support function body");
    let owner_check = production
        .split("fn owner_dependency_is_valid(")
        .nth(1)
        .expect("owner dependency check");

    assert!(!target_check.contains("collect::<Vec<_>>()"));
    assert!(!owner_check.contains("collect::<Vec<_>>()"));

    let mut feature =
        PluginFeatureBundleManifest::new("benchmark.feature", "Benchmark Feature", "benchmark")
            .with_dependency(PluginFeatureDependency::primary(
                "benchmark",
                "runtime.plugin.benchmark",
            ));
    feature.modules.push(benchmark_runtime_module(0));
    assert_eq!(
        feature_manifest_supports_target(&feature, RuntimeTargetMode::ClientRuntime),
        legacy_feature_manifest_supports_target(&feature, RuntimeTargetMode::ClientRuntime)
    );
    assert_eq!(
        owner_dependency_is_valid(&feature),
        legacy_owner_dependency_is_valid(&feature)
    );
}

#[test]
#[ignore = "release-only streaming feature checks benchmark"]
fn optimization_batch_en_streaming_feature_checks_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const CHECKS_PER_SAMPLE: usize = 8_192;
    const MODULE_COUNT: usize = 128;

    fn measure_legacy(feature: &PluginFeatureBundleManifest) -> u128 {
        let started = Instant::now();
        let mut matched = 0_usize;
        for _ in 0..CHECKS_PER_SAMPLE {
            matched += usize::from(black_box(
                legacy_feature_manifest_supports_target(
                    black_box(feature),
                    RuntimeTargetMode::ClientRuntime,
                ) && legacy_owner_dependency_is_valid(black_box(feature)),
            ));
        }
        black_box(matched);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(feature: &PluginFeatureBundleManifest) -> u128 {
        let started = Instant::now();
        let mut matched = 0_usize;
        for _ in 0..CHECKS_PER_SAMPLE {
            matched += usize::from(black_box(
                feature_manifest_supports_target(
                    black_box(feature),
                    RuntimeTargetMode::ClientRuntime,
                ) && owner_dependency_is_valid(black_box(feature)),
            ));
        }
        black_box(matched);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let mut feature =
        PluginFeatureBundleManifest::new("benchmark.feature", "Benchmark Feature", "benchmark")
            .with_dependency(PluginFeatureDependency::primary(
                "benchmark",
                "runtime.plugin.benchmark",
            ));
    feature.modules = (0..MODULE_COUNT)
        .map(benchmark_runtime_module)
        .collect::<Vec<_>>();
    feature
        .modules
        .last_mut()
        .expect("benchmark runtime module")
        .target_modes = vec![RuntimeTargetMode::ClientRuntime];

    assert!(feature_manifest_supports_target(
        &feature,
        RuntimeTargetMode::ClientRuntime
    ));
    assert!(owner_dependency_is_valid(&feature));
    for _ in 0..4 {
        black_box(measure_legacy(&feature));
        black_box(measure_optimized(&feature));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&feature));
            optimized_samples.push(measure_optimized(&feature));
        } else {
            optimized_samples.push(measure_optimized(&feature));
            legacy_samples.push(measure_legacy(&feature));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR376_STREAMING_FEATURE_CHECKS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             checks_per_sample={CHECKS_PER_SAMPLE} module_count={MODULE_COUNT} \
             pair_order=alternating_legacy_even legacy_vector_allocations_per_check=2 \
             optimized_vector_allocations_per_check=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "streaming feature checks must reduce P95 by at least 20%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
