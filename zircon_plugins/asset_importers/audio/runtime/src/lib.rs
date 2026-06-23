mod capability;
mod plugin;

pub use capability::{
    CODEC_IMPORTER_CAPABILITY, IMPORTER_FAMILY, PLUGIN_ID, RUNTIME_CAPABILITIES,
    RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
};
pub use plugin::{
    asset_importer_descriptors, package_manifest, plugin_registration, runtime_capabilities,
    runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor, runtime_selection,
    supported_platforms, supported_targets, AudioAssetImporterRuntimePlugin,
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
        assert!(manifest
            .capabilities
            .contains(&CODEC_IMPORTER_CAPABILITY.to_string()));
        assert_eq!(manifest.supported_targets, supported_targets());
        assert_eq!(manifest.supported_platforms, supported_platforms());
        assert_eq!(manifest.modules.len(), 1);
        assert_eq!(manifest.modules[0].crate_name, RUNTIME_CRATE_NAME);
        assert_eq!(
            manifest.modules[0].capabilities,
            runtime_capabilities()
                .iter()
                .map(|capability| capability.to_string())
                .collect::<Vec<_>>()
        );
    }
}
