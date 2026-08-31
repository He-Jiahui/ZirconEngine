use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    ARRAY_IMPORTER_CAPABILITY, CONTAINER_IMPORTER_CAPABILITY, CUBEMAP_IMPORTER_CAPABILITY,
    IMAGE_IMPORTER_CAPABILITY, PLUGIN_ID, PSD_IMPORTER_CAPABILITY, RUNTIME_CRATE_NAME,
    TEXTURE_IMPORTER_DECLARATION, import_cubemap, import_image, import_psd,
    import_texture_array_manifest, import_texture_container,
};

pub const TEXTURE_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_texture_importer_dist";
pub const TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_texture_importer_runtime_entry_v3";
const TEXTURE_IMPORTER_VERSION: u32 = 2;
const CUBEMAP_IMPORTER_VERSION: u32 = 3;

#[derive(Clone, Debug)]
pub struct TextureImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl TextureImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for TextureImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for TextureImporterRuntimePlugin {
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
            image_importer_descriptor(),
            import_image,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            container_importer_descriptor(),
            import_texture_container,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            psd_importer_descriptor(),
            import_psd,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            cubemap_importer_descriptor(),
            import_cubemap,
        ))?;
        registry.register_asset_importer(FunctionAssetImporter::new(
            array_importer_descriptor(),
            import_texture_array_manifest,
        ))?;
        registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
            optional_native_container_importer_descriptor(),
            "cubemap/dxgi texture import requires a NativeDynamic texture backend",
        ))?;
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    TEXTURE_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(TextureImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    TEXTURE_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = TEXTURE_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = TEXTURE_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    TEXTURE_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        image_importer_descriptor(),
        container_importer_descriptor(),
        psd_importer_descriptor(),
        cubemap_importer_descriptor(),
        array_importer_descriptor(),
        optional_native_container_importer_descriptor(),
    ]
}

fn image_importer_descriptor() -> AssetImporterDescriptor {
    descriptor(
        "texture_importer.image",
        120,
        [
            "png", "jpg", "jpeg", "bmp", "tga", "tiff", "tif", "gif", "webp", "hdr", "exr", "qoi",
            "pnm", "pbm", "pgm", "ppm",
        ],
    )
    .with_required_capabilities([IMAGE_IMPORTER_CAPABILITY])
}

fn container_importer_descriptor() -> AssetImporterDescriptor {
    descriptor(
        "texture_importer.container",
        90,
        ["dds", "ktx", "ktx2", "astc"],
    )
    .with_required_capabilities([CONTAINER_IMPORTER_CAPABILITY])
}

fn psd_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("texture_importer.psd", 100, ["psd"])
        .with_required_capabilities([PSD_IMPORTER_CAPABILITY])
}

fn cubemap_importer_descriptor() -> AssetImporterDescriptor {
    descriptor_with_version(
        "texture_importer.cubemap",
        CUBEMAP_IMPORTER_VERSION,
        130,
        ["zcube"],
    )
    .with_required_capabilities([CUBEMAP_IMPORTER_CAPABILITY])
}

fn array_importer_descriptor() -> AssetImporterDescriptor {
    descriptor("texture_importer.array", 130, ["zarray"])
        .with_required_capabilities([ARRAY_IMPORTER_CAPABILITY])
}

fn optional_native_container_importer_descriptor() -> AssetImporterDescriptor {
    descriptor(
        "texture_importer.optional_native_container",
        80,
        ["cubemap", "dxgi"],
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
    manifest.supported_platforms = TEXTURE_IMPORTER_DECLARATION.supported_platforms().to_vec();
    manifest.default_packaging = TEXTURE_IMPORTER_DECLARATION.default_packaging().to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        TEXTURE_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "texture_importer.dist",
        TEXTURE_IMPORTER_DIST_CRATE_NAME,
        TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn descriptor(
    id: impl Into<String>,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    descriptor_with_version(id, TEXTURE_IMPORTER_VERSION, priority, extensions)
}

fn descriptor_with_version(
    id: impl Into<String>,
    importer_version: u32,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Texture, importer_version)
        .with_priority(priority)
        .with_source_extensions(extensions)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const REGISTRATION_PLANS_PER_SAMPLE: usize = 4_096;

    #[test]
    fn canonical_descriptor_builders_preserve_public_order_and_capabilities() {
        let descriptors = asset_importer_descriptors();
        let observed = descriptors
            .iter()
            .map(|descriptor| {
                (
                    descriptor.id.as_str(),
                    descriptor.importer_version,
                    descriptor.priority,
                    descriptor.required_capabilities[0].as_str(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            observed,
            [
                (
                    "texture_importer.image",
                    TEXTURE_IMPORTER_VERSION,
                    120,
                    IMAGE_IMPORTER_CAPABILITY,
                ),
                (
                    "texture_importer.container",
                    TEXTURE_IMPORTER_VERSION,
                    90,
                    CONTAINER_IMPORTER_CAPABILITY,
                ),
                (
                    "texture_importer.psd",
                    TEXTURE_IMPORTER_VERSION,
                    100,
                    PSD_IMPORTER_CAPABILITY,
                ),
                (
                    "texture_importer.cubemap",
                    CUBEMAP_IMPORTER_VERSION,
                    130,
                    CUBEMAP_IMPORTER_CAPABILITY,
                ),
                (
                    "texture_importer.array",
                    TEXTURE_IMPORTER_VERSION,
                    130,
                    ARRAY_IMPORTER_CAPABILITY,
                ),
                (
                    "texture_importer.optional_native_container",
                    TEXTURE_IMPORTER_VERSION,
                    80,
                    "runtime.asset.importer.native",
                ),
            ]
        );
    }

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_direct_texture_importer_registration_plan() {
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
            "PERF_RESULT task=plugins07_direct_texture_importer_registration sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank registration_plans_per_sample={REGISTRATION_PLANS_PER_SAMPLE} importers_per_plan=6 legacy_descriptor_vec_allocations_per_sample={REGISTRATION_PLANS_PER_SAMPLE} optimized_descriptor_vec_allocations_per_sample=0 legacy_string_dispatches_per_sample=24576 optimized_string_dispatches_per_sample=0 threshold_percent=10 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
    }

    fn legacy_registration_plan() -> usize {
        let mut checksum = 0;
        for importer in black_box(asset_importer_descriptors()) {
            let route = match black_box(importer.id.as_str()) {
                "texture_importer.image" => 1,
                "texture_importer.container" => 2,
                "texture_importer.psd" => 3,
                "texture_importer.cubemap" => 4,
                "texture_importer.array" => 5,
                "texture_importer.optional_native_container" => 6,
                _ => unreachable!(),
            };
            checksum += consume_descriptor(importer, route);
        }
        black_box(checksum)
    }

    fn direct_registration_plan() -> usize {
        let checksum = consume_descriptor(image_importer_descriptor(), 1)
            + consume_descriptor(container_importer_descriptor(), 2)
            + consume_descriptor(psd_importer_descriptor(), 3)
            + consume_descriptor(cubemap_importer_descriptor(), 4)
            + consume_descriptor(array_importer_descriptor(), 5)
            + consume_descriptor(optional_native_container_importer_descriptor(), 6);
        black_box(checksum)
    }

    fn consume_descriptor(descriptor: AssetImporterDescriptor, route: usize) -> usize {
        let descriptor = black_box(descriptor);
        black_box(descriptor.source_extensions.len() + route)
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
