use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind};
use zircon_runtime::{plugin::PluginPackageManifest, RuntimeTargetMode};

pub const PLUGIN_ID: &str = "asset_importer.model";
pub const IMPORTER_FAMILY: &str = "model";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.model";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY]
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.model.gltf", ["gltf", "glb"])
            .with_required_capabilities(["runtime.asset.importer.model.gltf"]),
        descriptor("asset_importer.model.obj", ["obj"])
            .with_required_capabilities(["runtime.asset.importer.model.obj"]),
        descriptor(
            "asset_importer.model.optional_backend",
            [
                "fbx", "dae", "3ds", "dxf", "ply", "stl", "usd", "usda", "usdc", "usdz",
            ],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    asset_importer_descriptors().into_iter().fold(
        PluginPackageManifest::new(PLUGIN_ID, "Model Asset Importers")
            .with_category("asset_importer")
            .with_runtime_crate("zircon_plugin_asset_importer_model_runtime")
            .with_supported_targets([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capability(RUNTIME_CAPABILITY),
        |manifest, importer| manifest.with_asset_importer(importer),
    )
}

fn descriptor(
    id: impl Into<String>,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Model, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_model_importer_capabilities() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"fbx".to_string())));
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
    }
}
