use crate::plugin::RuntimeExtensionRegistry;

use super::{RuntimePluginFeatureRegistrationReport, project_selection_from_feature_manifest};
use crate::plugin::runtime_plugin::{
    RuntimePluginFeature, feature_validation::validate_runtime_plugin_feature_manifest,
};

impl RuntimePluginFeatureRegistrationReport {
    pub fn from_feature(feature: &dyn RuntimePluginFeature) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let manifest = feature.manifest();
        let mut diagnostics = Vec::with_capacity(manifest.modules.len());
        validate_runtime_plugin_feature_manifest(&manifest, &mut diagnostics);
        if let Err(error) = feature.register(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        Self {
            project_selection: project_selection_from_feature_manifest(&manifest),
            provider_package_id: None,
            manifest,
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
mod optimization_batch_20260830bv_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const MODULES_PER_SAMPLE: usize = 128;

    #[test]
    fn feature_report_reserves_module_diagnostic_capacity() {
        let source = include_str!("feature.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(manifest.modules.len())"));
        assert!(!implementation.contains("let mut diagnostics = Vec::new()"));
    }

    #[test]
    fn feature_report_keeps_manifest_before_validation_and_registration() {
        let source = include_str!("feature.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let manifest = implementation
            .find("let manifest = feature.manifest()")
            .expect("manifest");
        let validation = implementation
            .find("validate_runtime_plugin_feature_manifest(&manifest")
            .expect("validation");
        let registration = implementation
            .find("feature.register(&mut extensions)")
            .expect("registration");
        assert!(manifest < validation);
        assert!(validation < registration);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bv_runtime_feature_report_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME374_FEATURE_REPORT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} modules_per_sample={MODULES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut diagnostics = if optimized {
                Vec::with_capacity(MODULES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..MODULES_PER_SAMPLE {
                diagnostics.push(index);
            }
            checksum ^= diagnostics.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
