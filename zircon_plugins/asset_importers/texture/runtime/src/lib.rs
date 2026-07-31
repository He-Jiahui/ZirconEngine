mod capability;
mod plugin;

pub use capability::{
    CONTAINER_IMPORTER_CAPABILITY, IMPORTER_FAMILY, PLUGIN_ID, PSD_IMPORTER_CAPABILITY,
    RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME,
    TEXTURE_ASSET_IMPORTER_DECLARATION,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, package_manifest, plugin_registration,
    runtime_capabilities, runtime_module_manifest, runtime_plugin, runtime_plugin_descriptor,
    runtime_selection, supported_platforms, supported_targets, TextureAssetImporterRuntimePlugin,
    TEXTURE_ASSET_IMPORTER_DIST_CRATE_NAME, TEXTURE_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_texture_importer_capabilities() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"ktx2".to_string())));
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(!manifest
            .capabilities
            .contains(&CONTAINER_IMPORTER_CAPABILITY.to_string()));
        assert!(!manifest
            .capabilities
            .contains(&PSD_IMPORTER_CAPABILITY.to_string()));
        assert_eq!(manifest.supported_targets, supported_targets());
        assert_eq!(manifest.supported_platforms, supported_platforms());
        let runtime_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "asset_importer.texture.runtime")
            .expect("texture importer package includes runtime module");
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
    fn texture_asset_importer_package_manifest_declares_dist_contract() {
        let manifest = package_manifest();
        let distribution = manifest
            .distribution
            .as_ref()
            .expect("texture importer package exposes dist metadata");

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
            TEXTURE_ASSET_IMPORTER_DIST_CRATE_NAME
        );
        assert_eq!(
            distribution.runtime_entry,
            TEXTURE_ASSET_IMPORTER_DIST_RUNTIME_ENTRY
        );

        let dist_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "asset_importer.texture.dist")
            .expect("texture importer package includes native dist module");
        assert_eq!(
            dist_module.kind,
            zircon_runtime::plugin::PluginModuleKind::Native
        );
        assert_eq!(
            dist_module.crate_name,
            TEXTURE_ASSET_IMPORTER_DIST_CRATE_NAME
        );
        assert!(dist_module.target_modes.contains(
            &zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
        ));
        assert!(dist_module
            .target_modes
            .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost));
        assert!(!dist_module
            .capabilities
            .contains(&CONTAINER_IMPORTER_CAPABILITY.to_string()));
        assert!(!dist_module
            .capabilities
            .contains(&PSD_IMPORTER_CAPABILITY.to_string()));
    }
}
