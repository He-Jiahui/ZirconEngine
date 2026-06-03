use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

mod manifest_metadata;

use self::manifest_metadata::register_package_manifest_metadata_contributions;

pub(in crate::plugin::runtime_plugin::registration_report) fn register_package_manifest_contributions(
    package_manifest: &PluginPackageManifest,
    extensions: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
) {
    // Manifest rows may mirror direct runtime registrations; validate them before
    // ignoring duplicate ids so malformed package metadata cannot be shadowed.
    register_package_manifest_metadata_contributions(package_manifest, extensions, diagnostics);
    for importer in package_manifest.asset_importers.iter().cloned() {
        if extensions
            .asset_importers()
            .descriptors()
            .iter()
            .any(|existing| existing.id == importer.id)
        {
            validate_duplicate_package_asset_importer(importer, diagnostics);
            continue;
        }
        if let Err(error) = extensions.register_asset_importer_descriptor(importer) {
            diagnostics.push(error.to_string());
        }
    }
}

fn validate_duplicate_package_asset_importer(
    importer: crate::asset::AssetImporterDescriptor,
    diagnostics: &mut Vec<String>,
) {
    let mut validation_registry = RuntimeExtensionRegistry::default();
    if let Err(error) = validation_registry.register_asset_importer_descriptor(importer) {
        diagnostics.push(error.to_string());
    }
}
