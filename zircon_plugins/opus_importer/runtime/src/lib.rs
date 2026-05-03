use zircon_runtime::asset::{AssetImporterDescriptor, AssetKind, DiagnosticOnlyAssetImporter};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "opus_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_opus_importer_runtime";
pub const MODULE_NAME: &str = "OpusImporterModule";
pub const OPUS_IMPORTER_ID: &str = "opus_importer.opus";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.opus_importer";
pub const OPUS_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.audio.opus";
pub const NATIVE_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.native";
pub const OPUS_IMPORTER_PRIORITY: i32 = 130;
const MISSING_BACKEND_DIAGNOSTIC: &str = "opus import requires a NativeDynamic libopus backend";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, OPUS_IMPORTER_CAPABILITY]
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
    ModuleDescriptor::new(MODULE_NAME, "Opus audio importer plugin")
}

pub fn asset_importer_descriptor() -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(OPUS_IMPORTER_ID, PLUGIN_ID, AssetKind::Sound, 1)
        .with_priority(OPUS_IMPORTER_PRIORITY)
        .with_source_extensions(["opus"])
        .with_required_capabilities([OPUS_IMPORTER_CAPABILITY, NATIVE_IMPORTER_CAPABILITY])
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![asset_importer_descriptor()]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Opus Audio Importer")
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
    PluginModuleManifest::runtime("opus_importer.runtime", RUNTIME_CRATE_NAME)
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
    registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
        asset_importer_descriptor(),
        MISSING_BACKEND_DIAGNOSTIC,
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::asset::{AssetImportContext, AssetImporterRegistry, AssetUri};

    #[test]
    fn package_declares_opus_native_dynamic_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert_eq!(
            runtime_capabilities(),
            &[RUNTIME_CAPABILITY, OPUS_IMPORTER_CAPABILITY]
        );
        assert_eq!(manifest.supported_targets, supported_targets());
        assert_eq!(manifest.supported_platforms, supported_platforms());
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .capabilities
            .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
        assert!(!manifest
            .capabilities
            .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));
        assert_eq!(manifest.modules.len(), 1);
        assert_eq!(manifest.modules[0].name, "opus_importer.runtime");
        assert_eq!(manifest.modules[0].crate_name, RUNTIME_CRATE_NAME);
        assert!(manifest.modules[0]
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest.modules[0]
            .capabilities
            .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
        assert!(!manifest.modules[0]
            .capabilities
            .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));
        assert!(manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::LibraryEmbed));
        assert!(!manifest
            .default_packaging
            .contains(&ExportPackagingStrategy::NativeDynamic));
        assert_eq!(manifest.asset_importers.len(), 1);

        let importer = &manifest.asset_importers[0];
        assert_eq!(importer.id, OPUS_IMPORTER_ID);
        assert_eq!(importer.plugin_id, PLUGIN_ID);
        assert_eq!(importer.output_kind, AssetKind::Sound);
        assert_eq!(importer.importer_version, 1);
        assert_eq!(importer.priority, OPUS_IMPORTER_PRIORITY);
        assert!(importer.source_extensions.contains(&"opus".to_string()));
        assert!(importer
            .required_capabilities
            .contains(&OPUS_IMPORTER_CAPABILITY.to_string()));
        assert!(importer
            .required_capabilities
            .contains(&NATIVE_IMPORTER_CAPABILITY.to_string()));

        let selection = runtime_selection();
        assert_eq!(selection.packaging, ExportPackagingStrategy::LibraryEmbed);
        assert_eq!(selection.runtime_crate.as_deref(), Some(RUNTIME_CRATE_NAME));
    }

    #[test]
    fn registration_contributes_module_and_opus_importer() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 1);
        assert_eq!(
            report.extensions.asset_importers().descriptors()[0].id,
            OPUS_IMPORTER_ID
        );
    }

    #[test]
    fn opus_importer_wins_over_audio_package_diagnostic_row() {
        let audio_report = zircon_plugin_audio_importer_runtime::plugin_registration();
        let opus_report = plugin_registration();
        let mut registry = AssetImporterRegistry::default();

        for importer in audio_report.extensions.asset_importers().importers() {
            registry.register_arc(importer.clone()).unwrap();
        }
        for importer in opus_report.extensions.asset_importers().importers() {
            registry.register_arc(importer.clone()).unwrap();
        }

        let selected = registry.select(std::path::Path::new("voice.opus")).unwrap();

        assert_eq!(selected.descriptor().id, OPUS_IMPORTER_ID);
        assert!(selected.descriptor().priority > 80);
    }

    #[test]
    fn missing_native_backend_reports_stable_opus_diagnostic() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("voice.opus"))
            .unwrap();
        let context = AssetImportContext::new(
            "voice.opus".into(),
            AssetUri::parse("res://audio/voice.opus").unwrap(),
            b"not a real opus stream".to_vec(),
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("NativeDynamic libopus backend"));
    }
}
