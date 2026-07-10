use zircon_plugin_sdk::{
    importer_runtime_supported_platforms, importer_runtime_supported_targets,
    ImporterRuntimeManifestBuilder,
};
use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind, FunctionAssetImporter};
use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    ExportTargetPlatform, PluginModuleManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginDescriptor,
};

use crate::{
    import_ui_zui_document, IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID, RUNTIME_CAPABILITY,
    RUNTIME_CRATE_NAME,
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
    RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "UI Document Importer",
        RuntimePluginId::UiDocumentImporter,
        RUNTIME_CRATE_NAME,
    )
    .with_module_descriptor(module_descriptor())
    .with_category("asset_importer")
    .with_target_modes(supported_targets())
    .with_capability(RUNTIME_CAPABILITY)
    .with_capability(IMPORTER_CAPABILITY)
    .build()
}

zircon_plugin_sdk::runtime_plugin_exports!(UiDocumentImporterRuntimePlugin);

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, IMPORTER_CAPABILITY]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    importer_runtime_supported_targets()
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    importer_runtime_supported_platforms()
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "UI document importer plugin")
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
    importer_manifest_builder()
        .with_asset_importers(asset_importer_descriptors())
        .build_package_manifest(descriptor)
}

fn importer_manifest_builder() -> ImporterRuntimeManifestBuilder {
    ImporterRuntimeManifestBuilder::new(
        "ui_document_importer.runtime",
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
