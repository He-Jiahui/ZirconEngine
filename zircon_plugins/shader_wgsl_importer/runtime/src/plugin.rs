use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind, FunctionAssetImporter};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    IMPORTER_CAPABILITY, PLUGIN_ID, RUNTIME_CRATE_NAME, SHADER_WGSL_IMPORTER_DECLARATION,
    import_wgsl,
};

pub const SHADER_WGSL_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_shader_wgsl_importer_dist";
pub const SHADER_WGSL_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_shader_wgsl_importer_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct ShaderWgslImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl ShaderWgslImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for ShaderWgslImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for ShaderWgslImporterRuntimePlugin {
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
        for importer in asset_importer_descriptors() {
            registry.register_asset_importer(FunctionAssetImporter::new(importer, import_wgsl))?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    SHADER_WGSL_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(ShaderWgslImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    SHADER_WGSL_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = SHADER_WGSL_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = SHADER_WGSL_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    SHADER_WGSL_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        AssetImporterDescriptor::new("shader_wgsl_importer.wgsl", PLUGIN_ID, AssetKind::Shader, 1)
            .with_priority(120)
            .with_source_extensions(["wgsl"])
            .with_required_capabilities([IMPORTER_CAPABILITY]),
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
    manifest.supported_platforms = SHADER_WGSL_IMPORTER_DECLARATION
        .supported_platforms()
        .to_vec();
    manifest.default_packaging = SHADER_WGSL_IMPORTER_DECLARATION
        .default_packaging()
        .to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        SHADER_WGSL_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "shader_wgsl_importer.dist",
        SHADER_WGSL_IMPORTER_DIST_CRATE_NAME,
        SHADER_WGSL_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}
