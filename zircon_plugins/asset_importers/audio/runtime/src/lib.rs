mod capability;
mod plugin;

pub use capability::{
    CODEC_IMPORTER_CAPABILITY, IMPORTER_FAMILY, PLUGIN_ID, RUNTIME_CAPABILITIES,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, AudioAssetImporterRuntimePlugin,
    AUDIO_ASSET_IMPORTER_DIST_CRATE_NAME, AUDIO_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_audio_importer_capabilities() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"flac".to_string())));
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(!manifest
            .capabilities
            .contains(&CODEC_IMPORTER_CAPABILITY.to_string()));
        assert_eq!(manifest.supported_targets, supported_targets());
        assert_eq!(manifest.supported_platforms, supported_platforms());
        let runtime_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "asset_importer.audio.runtime")
            .expect("audio importer package includes runtime module");
        assert_eq!(runtime_module.crate_name, RUNTIME_CRATE_NAME);
        assert_eq!(
            runtime_module.capabilities,
            runtime_capabilities()
                .iter()
                .map(|capability| capability.to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn audio_asset_importer_package_manifest_declares_dist_contract() {
        let manifest = package_manifest();
        let distribution = manifest
            .distribution
            .as_ref()
            .expect("audio importer package exposes dist metadata");

        assert!(manifest.default_packaging.contains(
            &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
        ));
        assert_eq!(distribution.forms, vec!["dist"]);
        assert_eq!(
            distribution.default_packaging,
            vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
        );
        assert_eq!(distribution.abi_version, Some(3));
        assert_eq!(
            distribution.dist_crate,
            AUDIO_ASSET_IMPORTER_DIST_CRATE_NAME
        );
        assert_eq!(
            distribution.runtime_entry,
            AUDIO_ASSET_IMPORTER_DIST_RUNTIME_ENTRY
        );

        let dist_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "asset_importer.audio.dist")
            .expect("audio importer package includes native dist module");
        assert_eq!(
            dist_module.kind,
            zircon_runtime::plugin::PluginModuleKind::Native
        );
        assert_eq!(dist_module.crate_name, AUDIO_ASSET_IMPORTER_DIST_CRATE_NAME);
        assert!(dist_module.target_modes.contains(
            &zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
        ));
        assert!(dist_module
            .target_modes
            .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost));
        assert!(!dist_module
            .capabilities
            .contains(&CODEC_IMPORTER_CAPABILITY.to_string()));
    }
}
