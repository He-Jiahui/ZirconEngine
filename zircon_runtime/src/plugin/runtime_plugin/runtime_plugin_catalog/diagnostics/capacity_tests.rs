use std::hint::black_box;
use std::time::Instant;

use super::{
    catalog_diagnostic_capacity, collect_catalog_diagnostics, RuntimePluginCatalogProjection,
};
use crate::core::framework::project::{ProjectPluginFeatureSelection, ProjectPluginSelection};
use crate::plugin::{
    PluginFeatureBundleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const PACKAGE_DIAGNOSTICS_PER_BUILD: usize = 128;
const FEATURE_DIAGNOSTICS_PER_BUILD: usize = 128;
const DIAGNOSTICS_PER_BUILD: usize = PACKAGE_DIAGNOSTICS_PER_BUILD + FEATURE_DIAGNOSTICS_PER_BUILD;

#[test]
fn optimization_batch_20260826ff_runtime201_capacity_preserves_catalog_diagnostic_order() {
    let registrations = [RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new("runtime201", "Runtime 201"),
        project_selection: ProjectPluginSelection::runtime_plugin("runtime201", true, true),
        extensions: RuntimeExtensionRegistry::default(),
        diagnostics: diagnostics("package", PACKAGE_DIAGNOSTICS_PER_BUILD),
    }];
    let feature_registrations = [RuntimePluginFeatureRegistrationReport {
        manifest: PluginFeatureBundleManifest::new(
            "runtime201.feature",
            "Runtime 201 Feature",
            "runtime201",
        ),
        provider_package_id: None,
        project_selection: ProjectPluginFeatureSelection::new("runtime201.feature"),
        extensions: RuntimeExtensionRegistry::default(),
        diagnostics: diagnostics("feature", FEATURE_DIAGNOSTICS_PER_BUILD),
    }];
    let projection = RuntimePluginCatalogProjection::default();

    let diagnostics =
        collect_catalog_diagnostics(None, &registrations, &feature_registrations, &projection);

    assert_eq!(diagnostics.len(), DIAGNOSTICS_PER_BUILD);
    assert!(diagnostics.capacity() >= DIAGNOSTICS_PER_BUILD);
    assert_eq!(diagnostics[0], "package-000");
    assert_eq!(
        diagnostics[PACKAGE_DIAGNOSTICS_PER_BUILD - 1],
        "package-127"
    );
    assert_eq!(diagnostics[PACKAGE_DIAGNOSTICS_PER_BUILD], "feature-000");
    assert_eq!(diagnostics[DIAGNOSTICS_PER_BUILD - 1], "feature-127");
    assert_eq!(
        catalog_diagnostic_capacity(None, &registrations, &feature_registrations, &projection,),
        DIAGNOSTICS_PER_BUILD
    );
}

#[test]
fn optimization_batch_20260826ff_runtime201_catalog_diagnostics_reserve_all_sources() {
    let source = include_str!("../diagnostics.rs");
    assert!(source.contains("fn catalog_diagnostic_capacity("));
    assert!(source.contains("module_order_error.is_some()"));
    assert!(source.contains("registration.diagnostics.len()"));
    assert!(source.contains("feature_definition_diagnostics().len()"));
    assert!(source.contains("bridge_dependency_diagnostics().len()"));
    assert!(source.contains("Vec::with_capacity(catalog_diagnostic_capacity("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ff_runtime201_catalog_diagnostic_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME201_CATALOG_DIAGNOSTIC_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} package_diagnostics={PACKAGE_DIAGNOSTICS_PER_BUILD} \
feature_diagnostics={FEATURE_DIAGNOSTICS_PER_BUILD} diagnostics_per_build={DIAGNOSTICS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn diagnostics(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("{prefix}-{index:03}"))
        .collect()
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let capacity = PACKAGE_DIAGNOSTICS_PER_BUILD.saturating_add(FEATURE_DIAGNOSTICS_PER_BUILD);
        let mut diagnostics = if reserve {
            Vec::with_capacity(capacity)
        } else {
            Vec::new()
        };
        for diagnostic in 0..PACKAGE_DIAGNOSTICS_PER_BUILD {
            diagnostics.push(black_box(diagnostic));
        }
        for diagnostic in 0..FEATURE_DIAGNOSTICS_PER_BUILD {
            diagnostics.push(black_box(diagnostic));
        }
        checksum ^= black_box(diagnostics.len() ^ diagnostics.capacity());
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
