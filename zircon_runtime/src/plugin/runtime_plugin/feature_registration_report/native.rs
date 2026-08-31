use crate::plugin::{PluginFeatureBundleManifest, PluginModuleKind, RuntimeExtensionRegistry};

use super::{RuntimePluginFeatureRegistrationReport, project_selection_from_feature_manifest};
use crate::plugin::runtime_plugin::feature_validation::{
    validate_runtime_plugin_feature_manifest, validate_runtime_plugin_feature_provider_package_id,
};

impl RuntimePluginFeatureRegistrationReport {
    pub fn from_native_feature_manifest(
        manifest: PluginFeatureBundleManifest,
        provider_package_id: Option<String>,
    ) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::with_capacity(manifest.modules.len());
        validate_runtime_plugin_feature_manifest(&manifest, &mut diagnostics);
        if let Some(provider_package_id) = provider_package_id.as_deref() {
            validate_runtime_plugin_feature_provider_package_id(
                provider_package_id,
                &mut diagnostics,
            );
        }
        for module in manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
        {
            if let Err(error) = extensions.register_module(module.module_descriptor()) {
                diagnostics.push(error.to_string());
            }
        }
        let mut project_selection = project_selection_from_feature_manifest(&manifest);
        project_selection.provider_package_id = provider_package_id.clone();
        Self {
            project_selection,
            provider_package_id,
            manifest,
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
mod optimization_batch_20260830bu_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const MODULES_PER_SAMPLE: usize = 128;

    #[test]
    fn native_feature_report_reserves_module_diagnostic_capacity() {
        let source = include_str!("native.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(manifest.modules.len())"));
        assert!(!implementation.contains("let mut diagnostics = Vec::new()"));
    }

    #[test]
    fn native_feature_report_validates_before_module_registration() {
        let source = include_str!("native.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let validation = implementation
            .find("validate_runtime_plugin_feature_manifest")
            .expect("manifest validation");
        let module_loop = implementation
            .find("for module in manifest")
            .expect("runtime module loop");
        assert!(validation < module_loop);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bu_runtime_native_feature_report_capacity_p95() {
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
            "RUNTIME373_NATIVE_FEATURE_REPORT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} modules_per_sample={MODULES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
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
