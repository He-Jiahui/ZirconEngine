use zircon_runtime::asset::{
    AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter, FunctionAssetImporter,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

use crate::cad::import_dxf_model;
use crate::{
    import_mesh_model, CAD_IMPORTER_CAPABILITY, MESH_IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};

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
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Model Asset Importers")
        .with_category("asset_importer")
        .with_runtime_crate(RUNTIME_CRATE_NAME)
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_runtime_module(runtime_module_manifest())
        .with_capabilities(runtime_capabilities().iter().copied());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("asset_importer.model.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: PLUGIN_ID.to_string(),
        enabled: true,
        required: false,
        target_modes: supported_targets().to_vec(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
        editor_crate: None,
        features: Vec::new(),
    }
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    if let Err(error) = register(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
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
