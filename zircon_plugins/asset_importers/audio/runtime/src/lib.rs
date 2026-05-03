use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind};
use zircon_runtime::{plugin::PluginPackageManifest, RuntimeTargetMode};

pub const PLUGIN_ID: &str = "asset_importer.audio";
pub const IMPORTER_FAMILY: &str = "audio";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.audio";
pub const CODEC_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.audio.codec";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, CODEC_IMPORTER_CAPABILITY]
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.audio.wav", ["wav"])
            .with_required_capabilities(["runtime.asset.importer.audio.wav"]),
        descriptor(
            "asset_importer.audio.codec",
            ["mp3", "ogg", "flac", "aif", "aiff"],
        )
        .with_required_capabilities([CODEC_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.audio.opus", ["opus"])
            .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let manifest = PluginPackageManifest::new(PLUGIN_ID, "Audio Asset Importers")
        .with_category("asset_importer")
        .with_runtime_crate("zircon_plugin_asset_importer_audio_runtime")
        .with_supported_targets([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]);
    let manifest = runtime_capabilities()
        .iter()
        .copied()
        .fold(manifest, |manifest, capability| {
            manifest.with_capability(capability)
        });

    asset_importer_descriptors()
        .into_iter()
        .fold(manifest, |manifest, importer| {
            manifest.with_asset_importer(importer)
        })
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
        assert!(manifest
            .capabilities
            .contains(&CODEC_IMPORTER_CAPABILITY.to_string()));
    }
}
