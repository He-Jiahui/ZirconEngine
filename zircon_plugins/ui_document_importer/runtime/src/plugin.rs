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
    import_ui_zui_document, IMPORTER_CAPABILITY, PLUGIN_ID, RUNTIME_CRATE_NAME,
    UI_DOCUMENT_IMPORTER_DECLARATION,
};

pub const UI_DOCUMENT_IMPORTER_DIST_CRATE_NAME: &str = "zircon_plugin_ui_document_importer_dist";
pub const UI_DOCUMENT_IMPORTER_DIST_RUNTIME_ENTRY: &str =
    "zircon_plugin_ui_document_importer_runtime_entry_v3";

#[derive(Clone, Debug)]
pub struct UiDocumentImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl UiDocumentImporterRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl Default for UiDocumentImporterRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimePlugin for UiDocumentImporterRuntimePlugin {
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
            registry.register_asset_importer(FunctionAssetImporter::new(
                importer,
                import_ui_zui_document,
            ))?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_descriptor() -> RuntimePluginDescriptor {
    UI_DOCUMENT_IMPORTER_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(module_descriptor())
        .into_descriptor()
}

zircon_plugin_sdk::runtime_plugin_exports!(UiDocumentImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    UI_DOCUMENT_IMPORTER_DECLARATION.capabilities()
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    let target_modes = UI_DOCUMENT_IMPORTER_DECLARATION.target_modes();
    [target_modes[0], target_modes[1]]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    let supported_platforms = UI_DOCUMENT_IMPORTER_DECLARATION.supported_platforms();
    [
        supported_platforms[0],
        supported_platforms[1],
        supported_platforms[2],
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    UI_DOCUMENT_IMPORTER_DECLARATION.module_descriptor()
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![ui_zui_descriptor("ui_document_importer.zui_document", 120)
        .with_full_suffixes([".zui"])
        .with_additional_output_kinds([AssetKind::UiLayout, AssetKind::UiStyle])]
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
    manifest.supported_platforms = UI_DOCUMENT_IMPORTER_DECLARATION
        .supported_platforms()
        .to_vec();
    manifest.default_packaging = UI_DOCUMENT_IMPORTER_DECLARATION
        .default_packaging()
        .to_vec();
    manifest
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        UI_DOCUMENT_IMPORTER_DECLARATION.module_name(),
        RUNTIME_CRATE_NAME,
        "ui_document_importer.dist",
        UI_DOCUMENT_IMPORTER_DIST_CRATE_NAME,
        UI_DOCUMENT_IMPORTER_DIST_RUNTIME_ENTRY,
    )
    .with_capabilities(runtime_capabilities().iter().copied())
}

fn ui_zui_descriptor(id: impl Into<String>, priority: i32) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::UiWidget, 2)
        .with_priority(priority)
        .with_required_capabilities([IMPORTER_CAPABILITY])
}
