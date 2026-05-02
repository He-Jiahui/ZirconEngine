use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    DiagnosticOnlyAssetImporter, FunctionAssetImporter, ImportedAsset, UiLayoutAsset, UiStyleAsset,
    UiWidgetAsset,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "ui_document_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_ui_document_importer_runtime";
pub const MODULE_NAME: &str = "UiDocumentImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.ui_document_importer";
pub const IMPORTER_CAPABILITY: &str = "runtime.asset.importer.ui_document";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, IMPORTER_CAPABILITY]
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
    ModuleDescriptor::new(MODULE_NAME, "UI document importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        ui_descriptor("ui_document_importer.typed_toml", 120).with_full_suffixes([".ui.toml"]),
        ui_descriptor("ui_document_importer.serialized_document", 100)
            .with_source_extensions(["zui", "uidoc"])
            .with_full_suffixes([".ui.json"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "UI Document Importer")
        .with_category("asset_importer")
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_capabilities(runtime_capabilities().iter().copied())
        .with_runtime_module(runtime_module_manifest());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("ui_document_importer.runtime", RUNTIME_CRATE_NAME)
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
    if let Err(error) = register_runtime_extensions(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        if importer.id == "ui_document_importer.typed_toml" {
            registry.register_asset_importer(FunctionAssetImporter::new(
                importer,
                import_ui_toml_document,
            ))?;
        } else {
            registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                importer,
                "serialized UI document importers require a UI document codec backend",
            ))?;
        }
    }
    Ok(())
}

pub fn import_ui_toml_document(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    if let Ok(asset) = UiLayoutAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiLayout(asset)));
    }
    if let Ok(asset) = UiWidgetAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiWidget(asset)));
    }
    if let Ok(asset) = UiStyleAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiStyle(asset)));
    }
    Err(AssetImportError::Parse(format!(
        "parse ui asset toml {}: unsupported or mismatched [asset.kind]",
        context.source_path.display()
    )))
}

fn ui_descriptor(id: impl Into<String>, priority: i32) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::UiLayout, 1)
        .with_priority(priority)
        .with_additional_output_kinds([AssetKind::UiWidget, AssetKind::UiStyle])
        .with_required_capabilities([IMPORTER_CAPABILITY])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_ui_document_importers() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest.asset_importers.iter().any(|importer| {
            importer.full_suffixes.contains(&".ui.toml".to_string())
                && importer.allows_output_kind(AssetKind::UiWidget)
        }));
    }

    #[test]
    fn registration_contributes_module_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 2);
    }

    #[test]
    fn typed_toml_importer_decodes_ui_layout_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("layout.ui.toml"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "layout.ui.toml".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/layout.ui.toml").unwrap(),
            br#"
[asset]
kind = "layout"
id = "main"
"#
            .to_vec(),
            Default::default(),
        );

        let imported = importer.import(&context).unwrap().imported_asset;

        assert!(matches!(
            imported,
            zircon_runtime::asset::ImportedAsset::UiLayout(_)
        ));
    }
}
