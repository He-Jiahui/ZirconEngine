use std::collections::HashSet;

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
    if package_manifest.asset_importers.is_empty() {
        return;
    }
    let mut registered_importer_ids = extensions
        .asset_importers()
        .descriptors()
        .into_iter()
        .map(|descriptor| descriptor.id)
        .collect::<HashSet<_>>();
    for importer in package_manifest.asset_importers.iter().cloned() {
        if registered_importer_ids.contains(importer.id.as_str()) {
            validate_duplicate_package_asset_importer(importer, diagnostics);
            continue;
        }
        let importer_id = importer.id.clone();
        match extensions.register_asset_importer_descriptor(importer) {
            Ok(()) => {
                registered_importer_ids.insert(importer_id);
            }
            Err(error) => diagnostics.push(error.to_string()),
        }
    }
}

// 同 ID 时仍用临时 registry 校验整条声明，避免重复冲突遮住非法 importer。
fn validate_duplicate_package_asset_importer(
    importer: crate::asset::AssetImporterDescriptor,
    diagnostics: &mut Vec<String>,
) {
    let mut validation_registry = RuntimeExtensionRegistry::default();
    if let Err(error) = validation_registry.register_asset_importer_descriptor(importer) {
        diagnostics.push(error.to_string());
    }
}

#[cfg(test)]
#[path = "tests/package_contributions.rs"]
mod tests;
