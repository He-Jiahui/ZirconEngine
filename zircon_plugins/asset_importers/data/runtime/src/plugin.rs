use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind, FunctionAssetImporter};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    import_json_data, import_toml_data, import_xml_data, import_yaml_data,
    DATA_ASSET_IMPORTER_DECLARATION, JSON_IMPORTER_CAPABILITY, PLUGIN_ID, RUNTIME_CRATE_NAME,
    TOML_IMPORTER_CAPABILITY, XML_IMPORTER_CAPABILITY, YAML_IMPORTER_CAPABILITY,
};

pub const ASSET_IMPORTER_DATA_DIST_CRATE_NAME: &str = "zircon_plugin_asset_importer_data_dist";
pub const ASSET_IMPORTER_DATA_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_asset_importer_data_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct DataAssetImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl DataAssetImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for DataAssetImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for DataAssetImporterRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        package_manifest_from_descriptor(self.descriptor())
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        register_asset_importers(registry)
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    DATA_ASSET_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(DataAssetImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    DATA_ASSET_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = DATA_ASSET_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = DATA_ASSET_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    DATA_ASSET_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        toml_importer_descriptor(),
        json_importer_descriptor(),
        yaml_importer_descriptor(),
        xml_importer_descriptor(),
    ]
}

fn toml_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.data.toml", ["toml"])
        .with_required_capabilities([TOML_IMPORTER_CAPABILITY])
}

fn json_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.data.json", ["json"])
        .with_required_capabilities([JSON_IMPORTER_CAPABILITY])
}

fn yaml_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.data.yaml", ["yaml", "yml"])
        .with_required_capabilities([YAML_IMPORTER_CAPABILITY])
}

fn xml_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.data.xml", ["xml"])
        .with_required_capabilities([XML_IMPORTER_CAPABILITY])
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    importer_manifest_builder().runtime_module_manifest()
}

pub fn dist_module_manifest() -> PluginModuleManifest {
    importer_manifest_builder().dist_module_manifest()
}

fn package_manifest_from_descriptor(descriptor: &RuntimePluginDescriptor) -> PluginPackageManifest {
    let mut manifest = importer_manifest_builder()
        .with_asset_importers(asset_importer_descriptors())
        .build_package_manifest(descriptor);
    manifest.supported_platforms = DATA_ASSET_IMPORTER_DECLARATION
        .supported_platforms()
        .to_vec();
    manifest.default_packaging = DATA_ASSET_IMPORTER_DECLARATION.default_packaging().to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        DATA_ASSET_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "asset_importer.data.dist",
        ASSET_IMPORTER_DATA_DIST_CRATE_NAME,
        ASSET_IMPORTER_DATA_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn register_asset_importers(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_asset_importer(FunctionAssetImporter::new(
        toml_importer_descriptor(),
        import_toml_data,
    ))?;
    registry.register_asset_importer(FunctionAssetImporter::new(
        json_importer_descriptor(),
        import_json_data,
    ))?;
    registry.register_asset_importer(FunctionAssetImporter::new(
        yaml_importer_descriptor(),
        import_yaml_data,
    ))?;
    registry.register_asset_importer(FunctionAssetImporter::new(
        xml_importer_descriptor(),
        import_xml_data,
    ))?;
    Ok(())
}

fn descriptor(
    id: impl Into<String>,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Data, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const REGISTRATION_PLANS_PER_SAMPLE: usize = 8_192;

    #[test]
    fn canonical_descriptor_builders_preserve_data_contract() {
        let descriptors = asset_importer_descriptors();
        let observed = descriptors
            .iter()
            .map(|descriptor| {
                (
                    descriptor.id.as_str(),
                    descriptor.required_capabilities[0].as_str(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            observed,
            [
                ("asset_importer.data.toml", TOML_IMPORTER_CAPABILITY),
                ("asset_importer.data.json", JSON_IMPORTER_CAPABILITY),
                ("asset_importer.data.yaml", YAML_IMPORTER_CAPABILITY),
                ("asset_importer.data.xml", XML_IMPORTER_CAPABILITY),
            ]
        );
    }

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_direct_data_importer_registration_plan() {
        let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_raw.push(measure_plans(legacy_registration_plan));
                optimized_raw.push(measure_plans(direct_registration_plan));
            } else {
                optimized_raw.push(measure_plans(direct_registration_plan));
                legacy_raw.push(measure_plans(legacy_registration_plan));
            }
        }
        emit_performance_result(legacy_raw, optimized_raw);
    }

    fn legacy_registration_plan() -> usize {
        let mut checksum = 0;
        for importer in black_box(asset_importer_descriptors()) {
            let route = match black_box(importer.id.as_str()) {
                "asset_importer.data.toml" => 1,
                "asset_importer.data.json" => 2,
                "asset_importer.data.yaml" => 3,
                "asset_importer.data.xml" => 4,
                _ => unreachable!(),
            };
            checksum += consume_descriptor(importer, route);
        }
        black_box(checksum)
    }

    fn direct_registration_plan() -> usize {
        let checksum = consume_descriptor(toml_importer_descriptor(), 1)
            + consume_descriptor(json_importer_descriptor(), 2)
            + consume_descriptor(yaml_importer_descriptor(), 3)
            + consume_descriptor(xml_importer_descriptor(), 4);
        black_box(checksum)
    }

    fn consume_descriptor(descriptor: AssetImporterDescriptor, route: usize) -> usize {
        let descriptor = black_box(descriptor);
        black_box(descriptor.source_extensions.len() + route)
    }

    fn measure_plans(plan: fn() -> usize) -> u64 {
        let started = Instant::now();
        let mut checksum = 0;
        for _ in 0..REGISTRATION_PLANS_PER_SAMPLE {
            checksum ^= plan();
        }
        black_box(checksum);
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn emit_performance_result(legacy_raw: Vec<u64>, optimized_raw: Vec<u64>) {
        let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
        let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "direct importer registration plan must improve P95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "PERF_RESULT task=plugins07_direct_data_importer_registration sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank registration_plans_per_sample={REGISTRATION_PLANS_PER_SAMPLE} importers_per_plan=4 legacy_descriptor_vec_allocations_per_sample={REGISTRATION_PLANS_PER_SAMPLE} optimized_descriptor_vec_allocations_per_sample=0 legacy_string_dispatches_per_sample=32768 optimized_string_dispatches_per_sample=0 threshold_percent=10 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
    }

    fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u64]) -> String {
        let values = samples
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        format!("[{values}]")
    }
}
