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

use crate::cad::import_dxf_model;
use crate::{
    import_mesh_model, CAD_IMPORTER_CAPABILITY, MESH_IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};

pub const MODEL_ASSET_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_asset_importer_model_dist";
pub const MODEL_ASSET_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_asset_importer_model_runtime_entry_v3";

const MODEL_ASSET_IMPORTER_DIST_ENGINE_COMPAT: &str = ">=0.1, <0.2";
const NATIVE_DESCRIPTOR_SYMBOL_V3: &str = "zircon_native_plugin_descriptor_v3";
const NATIVE_ABI_VERSION_V3: u32 = 3;

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
        registry.register_module(module_descriptor())?;
        register_asset_importers(registry)
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Model Asset Importers",
        RuntimePluginId::AssetImporterModel,
        RUNTIME_CRATE_NAME,
    )
    .with_category("asset_importer")
    .with_target_modes(supported_targets())
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(MESH_IMPORTER_CAPABILITY)
    .with_capability(CAD_IMPORTER_CAPABILITY)
    .build()
}

pub fn runtime_plugin() -> ModelAssetImporterRuntimePlugin {
    ModelAssetImporterRuntimePlugin::new()
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        MESH_IMPORTER_CAPABILITY,
        CAD_IMPORTER_CAPABILITY,
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
    ModuleDescriptor::new(MODULE_NAME, "Model asset importer family plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.model.gltf", 100, ["gltf", "glb"])
            .with_required_capabilities(["runtime.asset.importer.model.gltf"]),
        descriptor("asset_importer.model.obj", 100, ["obj"])
            .with_required_capabilities(["runtime.asset.importer.model.obj"]),
        descriptor("asset_importer.model.mesh", 110, ["ply", "stl"])
            .with_required_capabilities([MESH_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.model.cad", 110, ["dxf"])
            .with_required_capabilities([CAD_IMPORTER_CAPABILITY]),
        descriptor(
            "asset_importer.model.optional_native_backend",
            80,
            ["fbx", "dae", "3ds", "usd", "usda", "usdc", "usdz"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("asset_importer.model.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native(
        "asset_importer.model.dist",
        MODEL_ASSET_IMPORTER_DIST_CRATE_NAME,
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
        engine_compat: MODEL_ASSET_IMPORTER_DIST_ENGINE_COMPAT.to_string(),
        dist_crate: MODEL_ASSET_IMPORTER_DIST_CRATE_NAME.to_string(),
        descriptor_symbol: NATIVE_DESCRIPTOR_SYMBOL_V3.to_string(),
        runtime_entry: MODEL_ASSET_IMPORTER_DIST_RUNTIME_ENTRY.to_string(),
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
            "asset_importer.model.mesh" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_mesh_model))?,
            "asset_importer.model.cad" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_dxf_model))?,
            "asset_importer.model.gltf" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "gltf/glb import is provided by the split gltf_importer package",
                ))?;
            }
            "asset_importer.model.obj" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "obj import is provided by the split obj_importer package",
                ))?;
            }
            "asset_importer.model.optional_native_backend" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "fbx/dae/3ds/usd import requires a NativeDynamic model backend",
                ))?;
            }
            _ => unreachable!("asset_importer_descriptors returns only known model importer ids"),
        }
    }
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
