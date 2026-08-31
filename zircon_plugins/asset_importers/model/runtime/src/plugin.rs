use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::cad::import_dxf_model;
use crate::{
    import_mesh_model, CAD_IMPORTER_CAPABILITY, MESH_IMPORTER_CAPABILITY,
    MODEL_ASSET_IMPORTER_DECLARATION, PLUGIN_ID, RUNTIME_CRATE_NAME,
};

pub const MODEL_ASSET_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_asset_importer_model_dist";
pub const MODEL_ASSET_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_asset_importer_model_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct ModelAssetImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl ModelAssetImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ModelAssetImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for ModelAssetImporterRuntimePlugin {
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
    MODEL_ASSET_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(ModelAssetImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    MODEL_ASSET_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = MODEL_ASSET_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = MODEL_ASSET_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    MODEL_ASSET_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        gltf_importer_descriptor(),
        obj_importer_descriptor(),
        mesh_importer_descriptor(),
        cad_importer_descriptor(),
        optional_native_importer_descriptor(),
    ]
}

fn gltf_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.model.gltf", 100, ["gltf", "glb"])
        .with_required_capabilities(["runtime.asset.importer.model.gltf"])
}

fn obj_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.model.obj", 100, ["obj"])
        .with_required_capabilities(["runtime.asset.importer.model.obj"])
}

fn mesh_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.model.mesh", 110, ["ply", "stl"])
        .with_required_capabilities([MESH_IMPORTER_CAPABILITY])
}

fn cad_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("asset_importer.model.cad", 110, ["dxf"])
        .with_required_capabilities([CAD_IMPORTER_CAPABILITY])
}

fn optional_native_importer_descriptor() -> AssetImporterDescriptor {
    descriptor(
        "asset_importer.model.optional_native_backend",
        80,
        ["fbx", "dae", "3ds", "usd", "usda", "usdc", "usdz"],
    )
    .with_required_capabilities(["runtime.asset.importer.native"])
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
    manifest.supported_platforms = MODEL_ASSET_IMPORTER_DECLARATION
        .supported_platforms()
        .to_vec();
    manifest.default_packaging = MODEL_ASSET_IMPORTER_DECLARATION
        .default_packaging()
        .to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        MODEL_ASSET_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "asset_importer.model.dist",
        MODEL_ASSET_IMPORTER_DIST_CRATE_NAME,
        MODEL_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn register_asset_importers(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
        gltf_importer_descriptor(),
        "gltf/glb import is provided by the split gltf_importer package",
    ))?;
    registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
        obj_importer_descriptor(),
        "obj import is provided by the split obj_importer package",
    ))?;
    registry.register_asset_importer(FunctionAssetImporter::new(
        mesh_importer_descriptor(),
        import_mesh_model,
    ))?;
    registry.register_asset_importer(FunctionAssetImporter::new(
        cad_importer_descriptor(),
        import_dxf_model,
    ))?;
    registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
        optional_native_importer_descriptor(),
        "fbx/dae/3ds/usd import requires a NativeDynamic model backend",
    ))?;
    Ok(())
}

fn descriptor(
    id: impl Into<String>,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Model, 1)
        .with_priority(priority)
        .with_source_extensions(extensions)
        .with_additional_output_kinds([AssetKind::Mesh])
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const REGISTRATION_PLANS_PER_SAMPLE: usize = 4_096;

    #[test]
    fn canonical_descriptor_builders_preserve_model_contract() {
        let descriptors = asset_importer_descriptors();
        let observed = descriptors
            .iter()
            .map(|descriptor| (descriptor.id.as_str(), descriptor.priority))
            .collect::<Vec<_>>();

        assert_eq!(
            observed,
            [
                ("asset_importer.model.gltf", 100),
                ("asset_importer.model.obj", 100),
                ("asset_importer.model.mesh", 110),
                ("asset_importer.model.cad", 110),
                ("asset_importer.model.optional_native_backend", 80),
            ]
        );
        assert!(descriptors
            .iter()
            .all(|descriptor| descriptor.additional_output_kinds == [AssetKind::Mesh]));
    }

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_direct_model_importer_registration_plan() {
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
                "asset_importer.model.gltf" => 1,
                "asset_importer.model.obj" => 2,
                "asset_importer.model.mesh" => 3,
                "asset_importer.model.cad" => 4,
                "asset_importer.model.optional_native_backend" => 5,
                _ => unreachable!(),
            };
            checksum += consume_descriptor(importer, route);
        }
        black_box(checksum)
    }

    fn direct_registration_plan() -> usize {
        let checksum = consume_descriptor(gltf_importer_descriptor(), 1)
            + consume_descriptor(obj_importer_descriptor(), 2)
            + consume_descriptor(mesh_importer_descriptor(), 3)
            + consume_descriptor(cad_importer_descriptor(), 4)
            + consume_descriptor(optional_native_importer_descriptor(), 5);
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
            "PERF_RESULT task=plugins07_direct_model_asset_importer_registration sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank registration_plans_per_sample={REGISTRATION_PLANS_PER_SAMPLE} importers_per_plan=5 legacy_descriptor_vec_allocations_per_sample={REGISTRATION_PLANS_PER_SAMPLE} optimized_descriptor_vec_allocations_per_sample=0 legacy_string_dispatches_per_sample=20480 optimized_string_dispatches_per_sample=0 threshold_percent=10 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
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
