use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    FunctionAssetImporter, ImportedAsset, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "shader_wgsl_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_shader_wgsl_importer_runtime";
pub const MODULE_NAME: &str = "ShaderWgslImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.shader_wgsl_importer";
pub const IMPORTER_CAPABILITY: &str = "runtime.asset.importer.shader.wgsl";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, IMPORTER_CAPABILITY]
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
    ModuleDescriptor::new(MODULE_NAME, "WGSL shader importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        AssetImporterDescriptor::new("shader_wgsl_importer.wgsl", PLUGIN_ID, AssetKind::Shader, 1)
            .with_priority(120)
            .with_source_extensions(["wgsl"])
            .with_required_capabilities([IMPORTER_CAPABILITY]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "WGSL Shader Importer")
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
    PluginModuleManifest::runtime("shader_wgsl_importer.runtime", RUNTIME_CRATE_NAME)
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
        registry.register_asset_importer(FunctionAssetImporter::new(importer, import_wgsl))?;
    }
    Ok(())
}

pub fn import_wgsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let module = wgsl::parse_str(&source).map_err(|error| {
        AssetImportError::ShaderValidation(format!(
            "{}: {}",
            context.uri,
            error.emit_to_string(&source)
        ))
    })?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(&module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{}: {error}", context.uri)))?;
    let entry_points = module
        .entry_points
        .iter()
        .map(|entry| ShaderEntryPointAsset {
            name: entry.name.clone(),
            stage: format!("{:?}", entry.stage).to_ascii_lowercase(),
        })
        .collect();
    Ok(AssetImportOutcome::new(ImportedAsset::Shader(
        ShaderAsset {
            uri: context.uri.clone(),
            source_language: ShaderSourceLanguage::Wgsl,
            source: source.clone(),
            wgsl_source: source,
            entry_points,
            validation_diagnostics: Vec::new(),
        },
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_wgsl_importer() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"wgsl".to_string())));
    }

    #[test]
    fn registration_contributes_module_and_importer() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert!(report
            .extensions
            .asset_importers()
            .descriptors()
            .iter()
            .any(|importer| importer.id == "shader_wgsl_importer.wgsl"));
    }

    #[test]
    fn wgsl_importer_validates_shader_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("main.wgsl"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "main.wgsl".into(),
            zircon_runtime::asset::AssetUri::parse("res://shaders/main.wgsl").unwrap(),
            valid_wgsl().as_bytes().to_vec(),
            Default::default(),
        );

        let imported = importer.import(&context).unwrap().imported_asset;

        match imported {
            zircon_runtime::asset::ImportedAsset::Shader(shader) => {
                assert_eq!(shader.source_language, ShaderSourceLanguage::Wgsl);
                assert_eq!(shader.entry_points.len(), 2);
                assert!(shader.wgsl_source.contains("vs_main"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    fn valid_wgsl() -> &'static str {
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#
    }
}
