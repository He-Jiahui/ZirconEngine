use zircon_plugin_sdk::ImporterRuntimeManifestBuilder;
use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind, FunctionAssetImporter};
use zircon_runtime::core::framework::project::ExportTargetPlatform;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

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
        .runtime_declaration(RuntimePluginId::AssetImporterData, RUNTIME_CRATE_NAME)
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
        descriptor("asset_importer.data.toml", ["toml"])
            .with_required_capabilities([TOML_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.data.json", ["json"])
            .with_required_capabilities([JSON_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.data.yaml", ["yaml", "yml"])
            .with_required_capabilities([YAML_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.data.xml", ["xml"])
            .with_required_capabilities([XML_IMPORTER_CAPABILITY]),
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
    for importer in asset_importer_descriptors() {
        let import_fn = match importer.id.as_str() {
            "asset_importer.data.toml" => import_toml_data,
            "asset_importer.data.json" => import_json_data,
            "asset_importer.data.yaml" => import_yaml_data,
            "asset_importer.data.xml" => import_xml_data,
            _ => unreachable!("asset_importer_descriptors returns only known data importer ids"),
        };
        registry.register_asset_importer(FunctionAssetImporter::new(importer, import_fn))?;
    }
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
