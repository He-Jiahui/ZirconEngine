use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind};
use zircon_runtime::{plugin::PluginPackageManifest, RuntimeTargetMode};

pub const PLUGIN_ID: &str = "asset_importer.audio";
pub const IMPORTER_FAMILY: &str = "audio";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.audio";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY]
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.audio.wav", ["wav"])
            .with_required_capabilities(["runtime.asset.importer.audio.wav"]),
        descriptor(
            "asset_importer.audio.optional_codec",
            ["mp3", "ogg", "flac", "aif", "aiff", "opus"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    asset_importer_descriptors().into_iter().fold(
        PluginPackageManifest::new(PLUGIN_ID, "Audio Asset Importers")
            .with_category("asset_importer")
            .with_runtime_crate("zircon_plugin_asset_importer_audio_runtime")
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
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Sound, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}

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
    }
}
