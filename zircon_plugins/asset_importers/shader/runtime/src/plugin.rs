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

use crate::{
    import_shader, NAGA_IMPORTER_CAPABILITY, PLUGIN_ID, RUNTIME_CRATE_NAME,
    SHADER_ASSET_IMPORTER_DECLARATION, WGSL_IMPORTER_CAPABILITY,
};

pub const SHADER_ASSET_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_asset_importer_shader_dist";
pub const SHADER_ASSET_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_asset_importer_shader_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct ShaderAssetImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl ShaderAssetImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ShaderAssetImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for ShaderAssetImporterRuntimePlugin {
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
    SHADER_ASSET_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(ShaderAssetImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    SHADER_ASSET_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = SHADER_ASSET_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = SHADER_ASSET_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    SHADER_ASSET_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.shader.wgsl", ["wgsl"])
            .with_required_capabilities([WGSL_IMPORTER_CAPABILITY]),
        descriptor(
            "asset_importer.shader.naga",
            ["glsl", "vert", "frag", "comp", "vs", "fs", "cs", "spv"],
        )
        .with_required_capabilities([NAGA_IMPORTER_CAPABILITY]),
        descriptor(
            "asset_importer.shader.optional_toolchain",
            ["hlsl", "cg", "fx"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
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
    manifest.supported_platforms = SHADER_ASSET_IMPORTER_DECLARATION
        .supported_platforms()
        .to_vec();
    manifest.default_packaging = SHADER_ASSET_IMPORTER_DECLARATION
        .default_packaging()
        .to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        SHADER_ASSET_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "asset_importer.shader.dist",
        SHADER_ASSET_IMPORTER_DIST_CRATE_NAME,
        SHADER_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn register_asset_importers(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    for importer in asset_importer_descriptors() {
        match importer.id.as_str() {
            "asset_importer.shader.wgsl" | "asset_importer.shader.naga" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_shader))?,
            "asset_importer.shader.optional_toolchain" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "hlsl/cg/fx import requires a NativeDynamic shader toolchain backend",
                ))?;
            }
            _ => unreachable!("asset_importer_descriptors returns only known shader importer ids"),
        }
    }
    Ok(())
}

fn descriptor(
    id: impl Into<String>,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Shader, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}
