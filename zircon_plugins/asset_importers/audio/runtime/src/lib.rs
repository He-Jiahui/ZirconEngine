use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{ExportTargetPlatform, PluginModuleManifest, PluginPackageManifest};

pub const PLUGIN_ID: &str = "asset_importer.audio";
pub const IMPORTER_FAMILY: &str = "audio";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_asset_importer_audio_runtime";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.audio";
pub const CODEC_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.audio.codec";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, CODEC_IMPORTER_CAPABILITY]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
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
        .with_runtime_module(runtime_module_manifest())
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms());
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

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("asset_importer.audio.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
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
