use zircon_runtime::asset::AssetImporterDescriptor;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};
use zircon_runtime::plugin::{
    PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest,
    RuntimePluginDescriptor,
};

pub const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
pub const NATIVE_ABI_VERSION_V3: u32 = 3;

pub fn importer_runtime_supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn importer_runtime_supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

#[derive(Clone, Debug)]
pub struct ImporterRuntimeManifestBuilder {
    runtime_module_name: String,
    runtime_crate_name: String,
    dist_module_name: String,
    dist_crate_name: String,
    dist_runtime_entry: String,
    engine_compat: String,
    capabilities: Vec<String>,
    importers: Vec<AssetImporterDescriptor>,
}

impl ImporterRuntimeManifestBuilder {
    pub fn new(
        runtime_module_name: impl Into<String>,
        runtime_crate_name: impl Into<String>,
        dist_module_name: impl Into<String>,
        dist_crate_name: impl Into<String>,
        dist_runtime_entry: impl Into<String>,
    ) -> Self {
        Self {
            runtime_module_name: runtime_module_name.into(),
            runtime_crate_name: runtime_crate_name.into(),
            dist_module_name: dist_module_name.into(),
            dist_crate_name: dist_crate_name.into(),
            dist_runtime_entry: dist_runtime_entry.into(),
            engine_compat: ">=0.1, <0.2".to_string(),
            capabilities: Vec::new(),
            importers: Vec::new(),
        }
    }

    pub fn with_engine_compat(mut self, engine_compat: impl Into<String>) -> Self {
        self.engine_compat = engine_compat.into();
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_asset_importers(
        mut self,
        importers: impl IntoIterator<Item = AssetImporterDescriptor>,
    ) -> Self {
        self.importers = importers.into_iter().collect();
        self
    }

    pub fn runtime_module_manifest(&self) -> PluginModuleManifest {
        PluginModuleManifest::runtime(
            self.runtime_module_name.clone(),
            self.runtime_crate_name.clone(),
        )
        .with_target_modes(importer_runtime_supported_targets())
        .with_capabilities(self.capabilities.iter().cloned())
    }

    pub fn dist_module_manifest(&self) -> PluginModuleManifest {
        PluginModuleManifest::native(self.dist_module_name.clone(), self.dist_crate_name.clone())
            .with_target_modes(importer_runtime_supported_targets())
            .with_capabilities(self.capabilities.iter().cloned())
    }

    pub fn distribution_manifest(&self) -> PluginDistributionManifest {
        PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat: self.engine_compat.clone(),
            dist_crate: self.dist_crate_name.clone(),
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: self.dist_runtime_entry.clone(),
            ..PluginDistributionManifest::default()
        }
    }

    pub fn build_package_manifest(
        self,
        descriptor: &RuntimePluginDescriptor,
    ) -> PluginPackageManifest {
        let Self {
            dist_module_name,
            dist_crate_name,
            dist_runtime_entry,
            engine_compat,
            capabilities,
            importers,
            ..
        } = self;
        let distribution_dist_crate = dist_crate_name.clone();
        let dist_module = PluginModuleManifest::native(dist_module_name, dist_crate_name)
            .with_target_modes(importer_runtime_supported_targets())
            .with_capabilities(capabilities);
        let distribution = PluginDistributionManifest {
            forms: vec!["dist".to_string()],
            default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
            abi_version: Some(NATIVE_ABI_VERSION_V3),
            engine_compat,
            dist_crate: distribution_dist_crate,
            descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
            runtime_entry: dist_runtime_entry,
            ..PluginDistributionManifest::default()
        };
        let mut manifest = descriptor.package_manifest();
        if !manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic)
        {
            manifest
                .default_packaging
                .push(ExportPackagingStrategy::NativeDynamic);
        }
        manifest = manifest.with_native_module(dist_module);
        manifest = manifest.with_distribution(distribution);
        for importer in importers {
            manifest = manifest.with_asset_importer(importer);
        }
        manifest
    }

    #[cfg(test)]
    fn legacy_build_package_manifest(
        self,
        descriptor: &RuntimePluginDescriptor,
    ) -> PluginPackageManifest {
        let mut manifest = descriptor.package_manifest();
        if !manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic)
        {
            manifest
                .default_packaging
                .push(ExportPackagingStrategy::NativeDynamic);
        }
        manifest = manifest.with_native_module(self.dist_module_manifest());
        manifest = manifest.with_distribution(self.distribution_manifest());
        for importer in self.importers {
            manifest = manifest.with_asset_importer(importer);
        }
        manifest
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::asset::AssetKind;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const MANIFEST_BUILDS_PER_SAMPLE: usize = 256;
    const CAPABILITIES_PER_BUILDER: usize = 32;
    const IMPORTERS_PER_BUILDER: usize = 8;

    #[test]
    fn importer_runtime_manifest_builder_move_preserves_complete_manifest() {
        let descriptor = benchmark_descriptor();
        let builder = benchmark_builder();

        let legacy = builder.clone().legacy_build_package_manifest(&descriptor);
        let optimized = builder.build_package_manifest(&descriptor);

        assert_eq!(optimized, legacy);
    }

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_importer_runtime_manifest_builder_move_projection() {
        let descriptor = benchmark_descriptor();
        let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_raw.push(measure_manifest_builds(
                    ImporterRuntimeManifestBuilder::legacy_build_package_manifest,
                    &descriptor,
                ));
                optimized_raw.push(measure_manifest_builds(
                    ImporterRuntimeManifestBuilder::build_package_manifest,
                    &descriptor,
                ));
            } else {
                optimized_raw.push(measure_manifest_builds(
                    ImporterRuntimeManifestBuilder::build_package_manifest,
                    &descriptor,
                ));
                legacy_raw.push(measure_manifest_builds(
                    ImporterRuntimeManifestBuilder::legacy_build_package_manifest,
                    &descriptor,
                ));
            }
        }

        let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
        let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
            "move-backed importer manifest assembly must improve P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "PERF_RESULT task=plugins07_move_importer_manifest_builder sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank manifest_builds_per_sample={MANIFEST_BUILDS_PER_SAMPLE} capabilities_per_builder={CAPABILITIES_PER_BUILDER} importers_per_builder={IMPORTERS_PER_BUILDER} legacy_builder_field_clone_allocations_per_build=37 optimized_builder_field_clone_allocations_per_build=1 legacy_capability_clones_per_build=32 optimized_capability_clones_per_build=0 threshold_percent=20 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
    }

    fn measure_manifest_builds(
        build: fn(
            ImporterRuntimeManifestBuilder,
            &RuntimePluginDescriptor,
        ) -> PluginPackageManifest,
        descriptor: &RuntimePluginDescriptor,
    ) -> u128 {
        let builders = (0..MANIFEST_BUILDS_PER_SAMPLE)
            .map(|_| benchmark_builder())
            .collect::<Vec<_>>();
        let started = Instant::now();
        for builder in builders {
            black_box(build(builder, black_box(descriptor)));
        }
        started.elapsed().as_nanos()
    }

    fn benchmark_builder() -> ImporterRuntimeManifestBuilder {
        ImporterRuntimeManifestBuilder::new(
            "benchmark.runtime",
            "zircon_plugin_benchmark_runtime",
            "benchmark.dist",
            "zircon_plugin_benchmark_dist",
            "zircon_plugin_benchmark_runtime_entry_v3",
        )
        .with_capabilities(
            (0..CAPABILITIES_PER_BUILDER)
                .map(|index| format!("runtime.asset.importer.benchmark.capability_{index}")),
        )
        .with_asset_importers((0..IMPORTERS_PER_BUILDER).map(|index| {
            AssetImporterDescriptor::new(
                format!("benchmark.importer_{index}"),
                "benchmark_importer",
                AssetKind::Data,
                1,
            )
            .with_source_extensions([format!("benchmark_{index}")])
        }))
    }

    fn benchmark_descriptor() -> RuntimePluginDescriptor {
        RuntimePluginDescriptor::builder(
            "benchmark_importer",
            "Benchmark Importer",
            zircon_runtime::builtin::RuntimePluginId::GltfImporter,
            "zircon_plugin_benchmark_runtime",
        )
        .build()
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
