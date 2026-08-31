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
    import_obj, IMPORTER_CAPABILITY, OBJ_IMPORTER_DECLARATION, PLUGIN_ID, RUNTIME_CRATE_NAME,
};

pub const OBJ_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_obj_importer_dist";
pub const OBJ_IMPORTER_DIST_RUNTIME_ENTRY: &str = "zircon_plugin_obj_importer_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct ObjImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl ObjImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ObjImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for ObjImporterRuntimePlugin {
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
        registry.register_asset_importer(FunctionAssetImporter::new(
            obj_importer_descriptor(),
            import_obj,
        ))?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    OBJ_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(ObjImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    OBJ_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = OBJ_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = OBJ_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    OBJ_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![obj_importer_descriptor()]
}

fn obj_importer_descriptor() -> AssetImporterDescriptor {
    AssetImporterDescriptor::new("obj_importer.obj", PLUGIN_ID, AssetKind::Model, 1)
        .with_priority(120)
        .with_source_extensions(["obj"])
        .with_additional_output_kinds([AssetKind::Mesh])
        .with_required_capabilities([IMPORTER_CAPABILITY])
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
    manifest.supported_platforms = OBJ_IMPORTER_DECLARATION.supported_platforms().to_vec();
    manifest.default_packaging = OBJ_IMPORTER_DECLARATION.default_packaging().to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        OBJ_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "obj_importer.dist",
        OBJ_IMPORTER_DIST_CRATE_NAME,
        OBJ_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const REGISTRATION_PLANS_PER_SAMPLE: usize = 16_384;

    #[test]
    fn canonical_descriptor_builder_preserves_obj_contract() {
        let descriptors = asset_importer_descriptors();

        assert_eq!(descriptors.len(), 1);
        let descriptor = &descriptors[0];
        assert_eq!(descriptor.id, "obj_importer.obj");
        assert_eq!(descriptor.priority, 120);
        assert_eq!(descriptor.source_extensions, ["obj"]);
        assert_eq!(descriptor.additional_output_kinds, [AssetKind::Mesh]);
        assert_eq!(descriptor.required_capabilities, [IMPORTER_CAPABILITY]);
    }

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_direct_obj_importer_registration_plan() {
        let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);

        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_raw.push(measure_registration_plans(legacy_registration_plan));
                optimized_raw.push(measure_registration_plans(direct_registration_plan));
            } else {
                optimized_raw.push(measure_registration_plans(direct_registration_plan));
                legacy_raw.push(measure_registration_plans(legacy_registration_plan));
            }
        }

        emit_performance_result(
            "plugins07_direct_obj_importer_registration",
            legacy_raw,
            optimized_raw,
        );
    }

    fn legacy_registration_plan() -> usize {
        let mut descriptors = black_box(asset_importer_descriptors()).into_iter();
        consume_descriptor(descriptors.next().expect("one OBJ importer descriptor"))
    }

    fn direct_registration_plan() -> usize {
        consume_descriptor(obj_importer_descriptor())
    }

    fn consume_descriptor(descriptor: AssetImporterDescriptor) -> usize {
        let descriptor = black_box(descriptor);
        black_box(
            descriptor.source_extensions.len()
                + descriptor.additional_output_kinds.len()
                + descriptor.required_capabilities.len(),
        )
    }

    fn measure_registration_plans(plan: fn() -> usize) -> u64 {
        let started = Instant::now();
        let mut checksum = 0;
        for _ in 0..REGISTRATION_PLANS_PER_SAMPLE {
            checksum ^= plan();
        }
        black_box(checksum);
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn emit_performance_result(task: &str, legacy_raw: Vec<u64>, optimized_raw: Vec<u64>) {
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
            "PERF_RESULT task={task} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank registration_plans_per_sample={REGISTRATION_PLANS_PER_SAMPLE} importers_per_plan=1 legacy_descriptor_vec_allocations_per_sample={REGISTRATION_PLANS_PER_SAMPLE} optimized_descriptor_vec_allocations_per_sample=0 threshold_percent=10 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
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
