use crate::asset::AssetImporterRegistry;
use crate::plugin::RuntimeExtensionRegistry;

pub(super) fn asset_importers_from_extension_registries<'a>(
    registries: impl IntoIterator<Item = &'a RuntimeExtensionRegistry>,
) -> (AssetImporterRegistry, Vec<String>) {
    let mut asset_importers = AssetImporterRegistry::default();
    let mut errors = Vec::new();
    for registry in registries {
        for importer in registry.asset_importers().importers() {
            if let Err(error) = asset_importers.register_arc(importer) {
                errors.push(format!("asset importer registration failed: {error}"));
            }
        }
    }
    (asset_importers, errors)
}
