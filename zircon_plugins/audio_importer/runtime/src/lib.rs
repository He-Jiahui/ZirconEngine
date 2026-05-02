use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    DiagnosticOnlyAssetImporter, FunctionAssetImporter, ImportedAsset, SoundAsset,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "audio_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_audio_importer_runtime";
pub const MODULE_NAME: &str = "AudioImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.audio_importer";
pub const WAV_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.audio.wav";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, WAV_IMPORTER_CAPABILITY]
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

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Audio importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("audio_importer.wav", 120, ["wav"])
            .with_required_capabilities([WAV_IMPORTER_CAPABILITY]),
        descriptor(
            "audio_importer.optional_codec",
            90,
            ["mp3", "ogg", "flac", "aif", "aiff", "opus"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Audio Importer")
        .with_category("asset_importer")
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_capabilities(runtime_capabilities().iter().copied())
        .with_runtime_module(runtime_module_manifest());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("audio_importer.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: PLUGIN_ID.to_string(),
        enabled: true,
        required: false,
        target_modes: supported_targets().to_vec(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
        editor_crate: None,
        features: Vec::new(),
    }
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    if let Err(error) = register_runtime_extensions(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        if importer.id == "audio_importer.wav" {
            registry.register_asset_importer(FunctionAssetImporter::new(importer, import_wav))?;
        } else {
            registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                importer,
                "compressed audio importers require a NativeDynamic or VM codec backend",
            ))?;
        }
    }
    Ok(())
}

pub fn import_wav(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let asset =
        SoundAsset::from_wav_bytes(&context.uri, &context.source_bytes).map_err(|error| {
            AssetImportError::Parse(format!(
                "decode wav {}: {error}",
                context.source_path.display()
            ))
        })?;
    Ok(AssetImportOutcome::new(ImportedAsset::Sound(asset)))
}

fn descriptor(
    id: impl Into<String>,
    priority: i32,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Sound, 1)
        .with_priority(priority)
        .with_source_extensions(extensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_audio_importers() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"flac".to_string())));
    }

    #[test]
    fn registration_contributes_module_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 2);
    }

    #[test]
    fn wav_importer_decodes_sound_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("tone.wav"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "tone.wav".into(),
            zircon_runtime::asset::AssetUri::parse("res://audio/tone.wav").unwrap(),
            tiny_wav_bytes(),
            Default::default(),
        );

        let imported = importer.import(&context).unwrap().imported_asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Sound(sound) => {
                assert_eq!(sound.sample_rate_hz, 8_000);
                assert_eq!(sound.channel_count, 1);
                assert_eq!(sound.frame_count(), 2);
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    fn tiny_wav_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&40_u32.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&8_000_u32.to_le_bytes());
        bytes.extend_from_slice(&16_000_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u16.to_le_bytes());
        bytes.extend_from_slice(&16_u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&16_384_i16.to_le_bytes());
        bytes
    }
}
