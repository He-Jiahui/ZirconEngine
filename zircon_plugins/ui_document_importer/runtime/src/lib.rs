use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    FunctionAssetImporter, ImportedAsset, UiV2ComponentAsset,
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
    vec![ui_zui_descriptor("ui_document_importer.zui_component", 120).with_full_suffixes([".zui"])]
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
        registry.register_asset_importer(FunctionAssetImporter::new(
            importer,
            import_ui_zui_component_document,
        ))?;
    }
    Ok(())
}

pub fn import_ui_zui_component_document(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let asset = UiV2ComponentAsset::from_zui_str(&document).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse .zui component asset {}: {error}",
            context.source_path.display()
        ))
    })?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::UiV2Component(asset),
    ))
}

fn ui_zui_descriptor(id: impl Into<String>, priority: i32) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::UiWidget, 2)
        .with_priority(priority)
        .with_required_capabilities([IMPORTER_CAPABILITY])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_only_zui_component_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert_eq!(manifest.asset_importers.len(), 1);
        assert!(manifest.asset_importers.iter().any(|importer| {
            importer.full_suffixes.contains(&".zui".to_string())
                && importer.importer_version == 2
                && importer.allows_output_kind(AssetKind::UiWidget)
        }));
        assert!(manifest.asset_importers.iter().all(|importer| !importer
            .full_suffixes
            .contains(&".ui.json".to_string())
            && !importer.full_suffixes.contains(&".v2.ui.toml".to_string())
            && !importer.allows_output_kind(AssetKind::UiLayout)
            && !importer.allows_output_kind(AssetKind::UiStyle)
            && !importer.source_extensions.contains(&"uidoc".to_string())));
    }

    #[test]
    fn plugin_toml_declares_only_zui_component_importer() {
        let manifest = include_str!("../../plugin.toml");

        assert_eq!(manifest.matches("[[asset_importers]]").count(), 1);
        assert!(manifest.contains("id = \"ui_document_importer.zui_component\""));
        assert!(manifest.contains("full_suffixes = [\".zui\"]"));
        assert!(manifest.contains("output_kind = \"UiWidget\""));
        assert!(manifest.contains("importer_version = 2"));
        assert!(!manifest.contains("ui_document_importer.serialized_json"));
        assert!(!manifest.contains("ui_document_importer.serialized_binary"));
        assert!(!manifest.contains("full_suffixes = [\".v2.ui.toml\"]"));
        assert!(!manifest.contains("full_suffixes = [\".ui.toml\"]"));
        assert!(!manifest.contains(".ui.json"));
        assert!(!manifest.contains("source_extensions = [\"uidoc\"]"));
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
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 1);
    }

    #[test]
    fn zui_importer_decodes_single_component_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("toolbar.zui"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "toolbar.zui".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/toolbar.zui").unwrap(),
            br#"
[asset]
kind = "component"
id = "toolbar"
version = 2

[components.Toolbar]
root = "root"

[nodes.root]
component = "Container"
"#
            .to_vec(),
            Default::default(),
        );

        let outcome = importer.import(&context).unwrap();
        let imported = &outcome.root_entry().expect("root UI asset entry").asset;

        assert!(matches!(
            imported,
            zircon_runtime::asset::ImportedAsset::UiV2Component(_)
        ));
    }

    #[test]
    fn registration_does_not_select_legacy_ui_document_formats() {
        let report = plugin_registration();
        let importers = report.extensions.asset_importers();

        assert!(importers
            .select(std::path::Path::new("layout.ui.json"))
            .is_err());
        assert!(importers
            .select(std::path::Path::new("layout.v2.ui.toml"))
            .is_err());
        assert!(importers
            .select(std::path::Path::new("layout.uidoc"))
            .is_err());
    }
}
