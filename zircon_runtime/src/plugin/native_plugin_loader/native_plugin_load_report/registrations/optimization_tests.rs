use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::plugin::{
    PluginFeatureBundleManifest, PluginModuleManifest, PluginPackageKind, PluginPackageManifest,
};

use super::{has_runtime_feature_module, runtime_feature_manifests};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 512;
const PACKAGE_COUNT: usize = 1_024;
const FEATURE_COUNT: usize = 1_024;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn fixture_packages() -> Vec<PluginPackageManifest> {
    (0..PACKAGE_COUNT)
        .map(|index| {
            let mut package = PluginPackageManifest::new(
                format!("runtime.registration.package.{index:04}.{}", "x".repeat(64)),
                format!("Package {index}"),
            );
            if index % 2 == 0 {
                package.modules.push(PluginModuleManifest::runtime(
                    format!("package_{index}.runtime"),
                    format!("runtime_package_{index}"),
                ));
            } else {
                package.package_kind = PluginPackageKind::FeatureExtension;
            }
            package
        })
        .collect()
}

fn fixture_feature_package() -> PluginPackageManifest {
    let mut package = PluginPackageManifest::new("runtime.feature.owner", "Feature Owner");
    for index in 0..FEATURE_COUNT {
        let feature = PluginFeatureBundleManifest::new(
            format!("runtime.feature.{index:04}.{}", "x".repeat(32)),
            format!("Feature {index}"),
            "runtime.feature.owner",
        )
        .with_runtime_module(PluginModuleManifest::runtime(
            format!("feature_{index}.runtime"),
            format!("runtime_feature_{index}"),
        ));
        if index % 2 == 0 {
            package.optional_features.push(feature);
        } else {
            package.feature_extensions.push(feature);
        }
    }
    package
}

fn measure_samples(mut operation: impl FnMut()) -> Vec<Duration> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            operation();
            started.elapsed()
        })
        .collect()
}

fn legacy_registration_candidate_count(packages: &[PluginPackageManifest]) -> usize {
    packages
        .iter()
        .cloned()
        .filter(|manifest| {
            manifest.package_kind != PluginPackageKind::FeatureExtension
                && manifest
                    .modules
                    .iter()
                    .any(|module| module.kind == crate::plugin::PluginModuleKind::Runtime)
        })
        .count()
}

fn optimized_registration_candidate_count(packages: &[PluginPackageManifest]) -> usize {
    packages
        .iter()
        .filter(|manifest| {
            manifest.package_kind != PluginPackageKind::FeatureExtension
                && manifest
                    .modules
                    .iter()
                    .any(|module| module.kind == crate::plugin::PluginModuleKind::Runtime)
        })
        .count()
}

#[test]
fn runtime58_feature_manifest_stream_preserves_optional_then_extension_order() {
    let package = fixture_feature_package();
    let ids = runtime_feature_manifests(&package)
        .map(|feature| feature.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids.len(), FEATURE_COUNT);
    assert_eq!(
        ids.first().copied(),
        Some("runtime.feature.0000.xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
    );
    assert!(ids[..FEATURE_COUNT / 2]
        .iter()
        .all(|id| id.contains("runtime.feature.")));
    assert!(ids.iter().all(|id| {
        package
            .optional_features
            .iter()
            .chain(package.feature_extensions.iter())
            .any(|feature| feature.id == *id)
    }));
}

#[test]
fn runtime58_registration_source_filters_before_cloning_and_streams_features() {
    let source = include_str!("../registrations.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .expect("optimization test module")
        .0;

    assert!(production.contains(".iter()\n            .filter(|manifest|"));
    assert!(!production.contains(".iter()\n            .cloned()\n            .filter(|manifest|"));
    assert!(!production.contains("let mut features = manifest.optional_features.clone()"));
    assert!(production.contains("impl Iterator<Item = &PluginFeatureBundleManifest>"));
}

#[test]
fn runtime58_feature_stream_retains_runtime_module_filter_contract() {
    let feature = PluginFeatureBundleManifest::new("feature", "Feature", "owner");
    assert!(!has_runtime_feature_module(&feature));
    assert!(has_runtime_feature_module(&feature.with_runtime_module(
        PluginModuleManifest::runtime("feature.runtime", "feature")
    )));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_borrowed_registration_manifest_filter_bench() {
    let packages = fixture_packages();
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_registration_candidate_count(&packages));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(optimized_registration_candidate_count(&packages));
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    let retained = optimized_registration_candidate_count(&packages);

    println!(
        "RUNTIME58_BORROWED_REGISTRATION_MANIFEST_FILTER_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} packages={} retained={} manifest_clones_before_filter={}->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        PACKAGE_COUNT,
        retained,
        PACKAGE_COUNT,
        retained,
    );
    assert_eq!(retained, PACKAGE_COUNT / 2);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 80,
        "optimized p95 should be at most 80% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_lazy_runtime_feature_manifest_bench() {
    let package = fixture_feature_package();
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            let mut features = package.optional_features.clone();
            features.extend(package.feature_extensions.iter().cloned());
            black_box(
                features
                    .into_iter()
                    .filter(has_runtime_feature_module)
                    .count(),
            );
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(
                runtime_feature_manifests(&package)
                    .filter(|feature| has_runtime_feature_module(*feature))
                    .count(),
            );
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);

    println!(
        "RUNTIME58_LAZY_RUNTIME_FEATURE_MANIFEST_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} features={} temporary_feature_vectors=1->0 feature_clones={}->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        FEATURE_COUNT,
        FEATURE_COUNT * 2,
        FEATURE_COUNT,
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 80,
        "optimized p95 should be at most 80% of legacy p95"
    );
}
