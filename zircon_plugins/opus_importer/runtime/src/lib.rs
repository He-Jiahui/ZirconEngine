mod capability;
mod plugin;

pub use capability::{
    MODULE_NAME, NATIVE_IMPORTER_CAPABILITY, OPUS_IMPORTER_CAPABILITY, OPUS_IMPORTER_ID,
    OPUS_IMPORTER_PRIORITY, PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use plugin::{
    asset_importer_descriptor, asset_importer_descriptors, dist_module_manifest, module_descriptor,
    package_manifest, plugin_registration, runtime_capabilities, runtime_module_manifest,
    runtime_plugin, runtime_plugin_descriptor, runtime_selection, supported_platforms,
    supported_targets, OpusImporterRuntimePlugin, OPUS_IMPORTER_DIST_CRATE_NAME,
    OPUS_IMPORTER_DIST_RUNTIME_ENTRY,
};

pub(crate) const MISSING_BACKEND_DIAGNOSTIC: &str =
    "opus import requires a NativeDynamic libopus backend";

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::{AssetImportContext, AssetImporterRegistry, AssetKind, AssetUri};
    use zircon_runtime::core::framework::project::ExportPackagingStrategy;

    #[test]
    fn package_declares_opus_native_dynamic_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert_eq!(
            runtime_capabilities(),
            &[RUNTIME_CAPABILITY, OPUS_IMPORTER_CAPABILITY]
        );
        assert_eq!(manifest.supported_targets, supported_targets());
        assert_eq!(manifest.supported_platforms, supported_platforms());
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .capabilities
            .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
        assert!(!manifest
            .capabilities
            .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));
        let runtime_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "opus_importer.runtime")
            .expect("opus importer package includes runtime module");
        assert_eq!(runtime_module.crate_name, RUNTIME_CRATE_NAME);
        assert!(runtime_module
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(runtime_module
            .capabilities
            .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
        assert!(!runtime_module
            .capabilities
            .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));
        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::LibraryEmbed));
        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic));
        assert_eq!(manifest.asset_importers.len(), 1);

        let importer = &manifest.asset_importers[0];
        assert_eq!(importer.id, OPUS_IMPORTER_ID);
        assert_eq!(importer.plugin_id, PLUGIN_ID);
        assert_eq!(importer.output_kind, AssetKind::Sound);
        assert_eq!(importer.importer_version, 1);
        assert_eq!(importer.priority, OPUS_IMPORTER_PRIORITY);
        assert!(importer.source_extensions.contains(&"opus".to_string()));
        assert!(importer
            .required_capabilities
            .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
        assert!(importer
            .required_capabilities
            .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));

        let selection = runtime_selection();
        assert_eq!(selection.packaging, ExportPackagingStrategy::LibraryEmbed);
        assert_eq!(selection.runtime_crate.as_deref(), Some(RUNTIME_CRATE_NAME));
    }

    #[test]
    fn package_manifest_declares_opus_importer_dist_contract() {
        let manifest = package_manifest();
        let distribution = manifest
            .distribution
            .as_ref()
            .expect("opus importer package exposes dist metadata");

        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic));
        assert_eq!(distribution.forms, vec!["dist"]);
        assert_eq!(
            distribution.default_packaging,
            vec![ExportPackagingStrategy::NativeDynamic]
        );
        assert_eq!(distribution.abi_version, Some(3));
        assert_eq!(distribution.dist_crate, OPUS_IMPORTER_DIST_CRATE_NAME);
        assert_eq!(distribution.runtime_entry, OPUS_IMPORTER_DIST_RUNTIME_ENTRY);

        let dist_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "opus_importer.dist")
            .expect("opus importer package includes native dist module");
        assert_eq!(
            dist_module.kind,
            zircon_runtime::plugin::PluginModuleKind::Native
        );
        assert_eq!(dist_module.crate_name, OPUS_IMPORTER_DIST_CRATE_NAME);
        assert!(dist_module.target_modes.contains(
            &zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
        ));
        assert!(dist_module
            .target_modes
            .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost));
        assert!(dist_module
            .capabilities
            .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
        assert!(!dist_module
            .capabilities
            .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn registration_contributes_module_and_opus_importer() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 1);
        assert_eq!(
            report.extensions.asset_importers().descriptors()[0].id,
            OPUS_IMPORTER_ID
        );
    }

    #[test]
    fn opus_importer_wins_over_audio_package_diagnostic_row() {
        let audio_report = zircon_plugin_audio_importer_runtime::plugin_registration();
        let opus_report = plugin_registration();
        let mut registry = AssetImporterRegistry::default();

        for importer in audio_report.extensions.asset_importers().importers() {
            registry.register_arc(importer.clone()).unwrap();
        }
        for importer in opus_report.extensions.asset_importers().importers() {
            registry.register_arc(importer.clone()).unwrap();
        }

        let selected = registry.select(std::path::Path::new("voice.opus")).unwrap();

        assert_eq!(selected.descriptor().id, OPUS_IMPORTER_ID);
        assert!(selected.descriptor().priority > 80);
    }

    #[test]
    fn missing_native_backend_reports_stable_opus_diagnostic() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("voice.opus"))
            .unwrap();
        let context = AssetImportContext::new(
            "voice.opus".into(),
            AssetUri::parse("res://audio/voice.opus").unwrap(),
            b"not a real opus stream".to_vec(),
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("NativeDynamic libopus backend"));
    }
}
