use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, UiV2ComponentAsset,
};

mod capability;
mod plugin;

pub use capability::{
    IMPORTER_CAPABILITY, MODULE_NAME, PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use plugin::{
    asset_importer_descriptors, module_descriptor, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, UiDocumentImporterRuntimePlugin,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::AssetKind;

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
