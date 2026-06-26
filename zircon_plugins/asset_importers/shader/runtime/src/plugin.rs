use zircon_runtime::asset::{
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, ExportTargetPlatform, PluginDistributionManifest,
    PluginModuleManifest, PluginPackageManifest, ProjectPluginSelection, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
    RuntimePluginRegistrationReport,
};

use crate::{
    import_shader, MODULE_NAME, NAGA_IMPORTER_CAPABILITY, PLUGIN_ID, RUNTIME_CAPABILITY,
    RUNTIME_CRATE_NAME, WGSL_IMPORTER_CAPABILITY,
};

pub const SHADER_ASSET_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_asset_importer_shader_dist";
pub const SHADER_ASSET_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_asset_importer_shader_runtime_entry_v3";

const SHADER_ASSET_IMPORTER_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

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
        registry.register_module(module_descriptor())?;
        register_asset_importers(registry)
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Shader Asset Importers",
        RuntimePluginId::AssetImporterShader,
        RUNTIME_CRATE_NAME,
    )
    .with_category("asset_importer")
    .with_target_modes(supported_targets())
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(WGSL_IMPORTER_CAPABILITY)
    .with_capability(NAGA_IMPORTER_CAPABILITY)
    .build()
}

pub fn runtime_plugin() -> ShaderAssetImporterRuntimePlugin {
    ShaderAssetImporterRuntimePlugin::new()
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        WGSL_IMPORTER_CAPABILITY,
        NAGA_IMPORTER_CAPABILITY,
    ]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Shader asset importer plugin")
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

pub fn package_manifest() -> PluginPackageManifest {
    RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("asset_importer.shader.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native(
        "asset_importer.shader.dist",
        SHADER_ASSET_IMPORTER_DIST_CRATE_NAME,
    )
    .with_target_modes(supported_targets())
    .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

fn package_manifest_from_descriptor(descriptor: &RuntimePluginDescriptor) -> PluginPackageManifest {
    let mut manifest = descriptor.package_manifest();
    manifest
        .default_packaging
        .push(ExportPackagingStrategy::NativeDynamic);
    manifest = manifest.with_native_module(dist_module_manifest());
    manifest = manifest.with_distribution(PluginDistributionManifest {
        forms: vec!["dist".to_string()],
        default_packaging: vec![ExportPackagingStrategy::NativeDynamic],
        abi_version: Some(NATIVE_ABI_VERSION_V3),
        engine_compat: SHADER_ASSET_IMPORTER_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: SHADER_ASSET_IMPORTER_DIST_CRATE_NAME.to_string(),
        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
        runtime_entry: SHADER_ASSET_IMPORTER_DIST_RUNTIME_ENTRY.to_string(),
        ..PluginDistributionManifest::default()
    });
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
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
