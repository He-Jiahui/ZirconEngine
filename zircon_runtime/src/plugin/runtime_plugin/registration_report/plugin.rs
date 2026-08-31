use crate::plugin::RuntimeExtensionRegistry;
use crate::plugin::runtime_plugin::{
    RuntimePlugin, descriptor::validate_runtime_plugin_descriptor,
};

use super::{
    RuntimePluginRegistrationReport,
    package_contributions::register_package_manifest_contributions,
    validation::{
        validate_runtime_plugin_package_manifest, validate_runtime_plugin_registration_interfaces,
        validate_runtime_plugin_registration_system_anchors,
    },
};

impl RuntimePluginRegistrationReport {
    pub fn from_plugin(plugin: &dyn RuntimePlugin) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let package_manifest = plugin.package_manifest();
        let mut diagnostics = Vec::with_capacity(package_manifest.modules.len());
        validate_runtime_plugin_descriptor(plugin, &mut diagnostics);
        if let Err(error) = extensions.register_module(plugin.module_descriptor().clone()) {
            diagnostics.push(error.to_string());
        }
        if let Err(error) = plugin.register(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        for source in plugin.shader_module_sources() {
            if let Err(error) =
                extensions.register_plugin_shader_module_source(&package_manifest.id, source)
            {
                diagnostics.push(error.to_string());
            }
        }
        let projection = validate_runtime_plugin_package_manifest(
            Some(plugin.descriptor()),
            &package_manifest,
            &mut diagnostics,
        );
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        validate_runtime_plugin_registration_interfaces(
            &package_manifest,
            &projection,
            &extensions,
            &mut diagnostics,
        );
        validate_runtime_plugin_registration_system_anchors(
            &package_manifest,
            &projection,
            &extensions,
            &mut diagnostics,
        );
        Self {
            package_manifest,
            project_selection: plugin.project_selection(),
            extensions,
            diagnostics,
        }
    }
}

#[cfg(test)]
mod optimization_batch_20260830bx_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const MODULES_PER_SAMPLE: usize = 128;

    #[test]
    fn plugin_report_reserves_module_diagnostic_capacity() {
        let source = include_str!("plugin.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(package_manifest.modules.len())"));
        assert!(!implementation.contains("let mut diagnostics = Vec::new()"));
    }

    #[test]
    fn plugin_report_fetches_manifest_before_diagnostics() {
        let source = include_str!("plugin.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let manifest = implementation
            .find("let package_manifest = plugin.package_manifest()")
            .expect("manifest");
        let capacity = implementation
            .find("Vec::with_capacity(package_manifest.modules.len())")
            .expect("diagnostic capacity");
        assert!(manifest < capacity);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bx_runtime_plugin_report_capacity_p95() {
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
            "RUNTIME376_PLUGIN_REPORT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} modules_per_sample={MODULES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
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

#[cfg(test)]
mod tests {
    use super::RuntimePluginRegistrationReport;
    use crate::builtin::RuntimePluginId;
    use crate::plugin::{PluginShaderModuleSource, RuntimePlugin, RuntimePluginDescriptor};

    struct LinkedShaderModuleFixture {
        descriptor: RuntimePluginDescriptor,
        source: PluginShaderModuleSource,
    }

    impl RuntimePlugin for LinkedShaderModuleFixture {
        fn descriptor(&self) -> &RuntimePluginDescriptor {
            &self.descriptor
        }

        fn shader_module_sources(&self) -> Vec<PluginShaderModuleSource> {
            vec![self.source.clone()]
        }
    }

    #[test]
    fn linked_plugin_shader_module_source_is_registered_with_the_runtime_owner() {
        let package_id = "zircon.fixture.linked";
        let plugin = LinkedShaderModuleFixture {
            descriptor: RuntimePluginDescriptor::builder(
                package_id,
                "Linked Shader Fixture",
                RuntimePluginId::new("linked_shader_fixture"),
                "zircon_fixture_linked_shader",
            )
            .build(),
            source: PluginShaderModuleSource::new(
                package_id,
                "zircon_fixture::linked_lighting",
                "fn linked_fixture_lighting() -> vec3f { return vec3f(0.3); }",
                "linked shader fixture",
            ),
        };

        let report = RuntimePluginRegistrationReport::from_plugin(&plugin);

        assert!(report.diagnostics.is_empty());
        assert_eq!(report.extensions.shader_module_sources(), &[plugin.source]);
    }
}
