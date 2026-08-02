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
