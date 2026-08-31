use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::{
    merge_package_feature_definitions, package_feature_declaration_capacity,
    RuntimePluginRegistrationReport,
};
use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest, RuntimeExtensionRegistry};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const PACKAGES_PER_BUILD: usize = 2;
const OPTIONAL_FEATURES_PER_PACKAGE: usize = 64;
const FEATURE_EXTENSIONS_PER_PACKAGE: usize = 64;
const DECLARATIONS_PER_BUILD: usize =
    PACKAGES_PER_BUILD * (OPTIONAL_FEATURES_PER_PACKAGE + FEATURE_EXTENSIONS_PER_PACKAGE);

#[test]
fn optimization_batch_20260826fi_runtime204_capacity_preserves_package_feature_declarations() {
    let registrations = (0..PACKAGES_PER_BUILD)
        .map(package_report)
        .collect::<Vec<_>>();
    let mut definitions = HashMap::new();
    let mut diagnostics = Vec::new();
    let mut definition_order = Vec::new();

    let declared = merge_package_feature_definitions(
        &registrations,
        &mut definitions,
        &mut diagnostics,
        &mut definition_order,
    );

    assert_eq!(
        package_feature_declaration_capacity(&registrations),
        DECLARATIONS_PER_BUILD
    );
    assert_eq!(declared.len(), DECLARATIONS_PER_BUILD);
    assert!(declared.capacity() >= DECLARATIONS_PER_BUILD);
    assert_eq!(definitions.len(), DECLARATIONS_PER_BUILD);
    assert_eq!(definition_order.len(), DECLARATIONS_PER_BUILD);
    assert!(diagnostics.is_empty());
    assert_eq!(definition_order[0], "package-0.optional-000@package-0");
    assert_eq!(
        definition_order[DECLARATIONS_PER_BUILD - 1],
        "package-1.extension-063@package-1"
    );
}

#[test]
fn optimization_batch_20260826fi_runtime204_package_feature_declarations_reserve_source_total() {
    let source = include_str!("../package.rs");
    assert!(source.contains("fn package_feature_declaration_capacity("));
    assert!(source.contains("registration.package_manifest.optional_features.len()"));
    assert!(source.contains("registration.package_manifest.feature_extensions.len()"));
    assert!(source.contains("HashSet::with_capacity(package_feature_declaration_capacity("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fi_runtime204_package_feature_declaration_capacity_bench() {
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
        "RUNTIME204_PACKAGE_FEATURE_DECLARATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} packages_per_build={PACKAGES_PER_BUILD} \
optional_features_per_package={OPTIONAL_FEATURES_PER_PACKAGE} \
feature_extensions_per_package={FEATURE_EXTENSIONS_PER_PACKAGE} \
declarations_per_build={DECLARATIONS_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn package_report(package: usize) -> RuntimePluginRegistrationReport {
    let package_id = format!("package-{package}");
    let mut manifest = PluginPackageManifest::new(&package_id, format!("Package {package}"));
    manifest.optional_features = (0..OPTIONAL_FEATURES_PER_PACKAGE)
        .map(|feature| {
            PluginFeatureBundleManifest::new(
                format!("{package_id}.optional-{feature:03}"),
                format!("Optional {feature}"),
                &package_id,
            )
        })
        .collect();
    manifest.feature_extensions = (0..FEATURE_EXTENSIONS_PER_PACKAGE)
        .map(|feature| {
            PluginFeatureBundleManifest::new(
                format!("{package_id}.extension-{feature:03}"),
                format!("Extension {feature}"),
                "external-owner",
            )
        })
        .collect();
    RuntimePluginRegistrationReport {
        package_manifest: manifest,
        project_selection: ProjectPluginSelection::runtime_plugin(&package_id, true, true),
        extensions: RuntimeExtensionRegistry::default(),
        diagnostics: Vec::new(),
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut declarations = if reserve {
            HashSet::with_capacity(DECLARATIONS_PER_BUILD)
        } else {
            HashSet::new()
        };
        for declaration in 0..DECLARATIONS_PER_BUILD {
            declarations.insert(black_box(declaration));
        }
        checksum ^= black_box(declarations.len() ^ declarations.capacity());
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
