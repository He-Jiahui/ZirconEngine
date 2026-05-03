use naga::valid::{Capabilities, ValidationFlags, Validator};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    DiagnosticOnlyAssetImporter, FunctionAssetImporter, ImportedAsset, ShaderAsset,
    ShaderEntryPointAsset, ShaderSourceLanguage,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "asset_importer.shader";
pub const IMPORTER_FAMILY: &str = "shader";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_asset_importer_shader_runtime";
pub const MODULE_NAME: &str = "ShaderImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.shader";
pub const WGSL_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.shader.wgsl";
pub const NAGA_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.shader.naga";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        WGSL_IMPORTER_CAPABILITY,
        NAGA_IMPORTER_CAPABILITY,
    ]
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
    ModuleDescriptor::new(MODULE_NAME, "Shader asset importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.shader.wgsl", ["wgsl"])
            .with_required_capabilities([WGSL_IMPORTER_CAPABILITY]),
        descriptor(
            "asset_importer.shader.naga",
            ["glsl", "vert", "frag", "comp", "vs", "fs", "cs", "spv"],
        )
        .with_required_capabilities([NAGA_IMPORTER_CAPABILITY]),
        descriptor(
            "asset_importer.shader.optional_toolchain",
            ["hlsl", "cg", "fx"],
        )
        .with_required_capabilities(["runtime.asset.importer.native"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Shader Asset Importers")
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
    PluginModuleManifest::runtime("asset_importer.shader.runtime", RUNTIME_CRATE_NAME)
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
        match importer.id.as_str() {
            "asset_importer.shader.wgsl" | "asset_importer.shader.naga" => registry
                .register_asset_importer(FunctionAssetImporter::new(importer, import_shader))?,
            "asset_importer.shader.optional_toolchain" => {
                registry.register_asset_importer(DiagnosticOnlyAssetImporter::new(
                    importer,
                    "hlsl/cg/fx import requires a NativeDynamic shader toolchain backend",
                ))?;
            }
            _ => unreachable!("asset_importer_descriptors returns only known shader importer ids"),
        }
    }
    Ok(())
}

pub fn import_shader(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "wgsl" => import_wgsl(context),
        "glsl" | "vert" | "frag" | "comp" | "vs" | "fs" | "cs" => import_glsl(context),
        "spv" => import_spirv(context),
        _ => Err(AssetImportError::UnsupportedFormat(format!(
            "shader importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

fn import_wgsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let module = naga::front::wgsl::parse_str(&source).map_err(|error| {
        AssetImportError::ShaderValidation(format!(
            "{}: {}",
            context.uri,
            error.emit_to_string(&source)
        ))
    })?;
    module_to_shader_asset(
        context,
        ShaderSourceLanguage::Wgsl,
        source.clone(),
        module,
        source,
    )
}

fn import_glsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let stage = infer_shader_stage(context)?;
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&naga::front::glsl::Options::from(stage), &source)
        .map_err(|error| {
            AssetImportError::ShaderValidation(format!(
                "{}: {}",
                context.uri,
                error.emit_to_string(&source)
            ))
        })?;
    let info = validate_naga_module(context, &module)?;
    let wgsl_source = module_to_wgsl(context, &module, &info)?;
    shader_outcome(
        context,
        ShaderSourceLanguage::Glsl,
        source,
        wgsl_source,
        shader_entry_points(&module),
    )
}

fn import_spirv(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let module = naga::front::spv::parse_u8_slice(
        &context.source_bytes,
        &naga::front::spv::Options::default(),
    )
    .map_err(|error| {
        AssetImportError::ShaderValidation(format!("{}: spir-v parse failed: {error}", context.uri))
    })?;
    let info = validate_naga_module(context, &module)?;
    let wgsl_source = module_to_wgsl(context, &module, &info)?;
    shader_outcome(
        context,
        ShaderSourceLanguage::SpirV,
        hex_encode(&context.source_bytes),
        wgsl_source,
        shader_entry_points(&module),
    )
}

fn module_to_shader_asset(
    context: &AssetImportContext,
    source_language: ShaderSourceLanguage,
    source: String,
    module: naga::Module,
    wgsl_source: String,
) -> Result<AssetImportOutcome, AssetImportError> {
    validate_naga_module(context, &module)?;
    shader_outcome(
        context,
        source_language,
        source,
        wgsl_source,
        shader_entry_points(&module),
    )
}

fn shader_outcome(
    context: &AssetImportContext,
    source_language: ShaderSourceLanguage,
    source: String,
    wgsl_source: String,
    entry_points: Vec<ShaderEntryPointAsset>,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(ImportedAsset::Shader(
        ShaderAsset {
            uri: context.uri.clone(),
            source_language,
            source,
            wgsl_source,
            entry_points,
            validation_diagnostics: Vec::new(),
        },
    )))
}

fn validate_naga_module(
    context: &AssetImportContext,
    module: &naga::Module,
) -> Result<naga::valid::ModuleInfo, AssetImportError> {
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{}: {error}", context.uri)))
}

fn module_to_wgsl(
    context: &AssetImportContext,
    module: &naga::Module,
    info: &naga::valid::ModuleInfo,
) -> Result<String, AssetImportError> {
    naga::back::wgsl::write_string(module, info, naga::back::wgsl::WriterFlags::empty()).map_err(
        |error| {
            AssetImportError::ShaderValidation(format!(
                "{}: wgsl emission failed: {error}",
                context.uri
            ))
        },
    )
}

fn infer_shader_stage(context: &AssetImportContext) -> Result<naga::ShaderStage, AssetImportError> {
    if let Some(stage) = context
        .import_settings
        .get("shader_stage")
        .and_then(|value| value.as_str())
    {
        return parse_shader_stage(stage);
    }

    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    let extension_stage = match extension.to_ascii_lowercase().as_str() {
        "vert" | "vs" => Some(naga::ShaderStage::Vertex),
        "frag" | "fs" => Some(naga::ShaderStage::Fragment),
        "comp" | "cs" => Some(naga::ShaderStage::Compute),
        _ => None,
    };
    if let Some(stage) = extension_stage {
        return Ok(stage);
    }

    let stem_hint = context
        .source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.rsplit('.').next())
        .unwrap_or_default();
    if stem_hint.is_empty() {
        Ok(naga::ShaderStage::Vertex)
    } else {
        parse_shader_stage(stem_hint).or(Ok(naga::ShaderStage::Vertex))
    }
}

fn parse_shader_stage(stage: &str) -> Result<naga::ShaderStage, AssetImportError> {
    match stage.trim().to_ascii_lowercase().as_str() {
        "vertex" | "vert" | "vs" => Ok(naga::ShaderStage::Vertex),
        "fragment" | "frag" | "fs" => Ok(naga::ShaderStage::Fragment),
        "compute" | "comp" | "cs" => Ok(naga::ShaderStage::Compute),
        other => Err(AssetImportError::Parse(format!(
            "unsupported shader stage `{other}`"
        ))),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn shader_entry_points(module: &naga::Module) -> Vec<ShaderEntryPointAsset> {
    module
        .entry_points
        .iter()
        .map(|entry| ShaderEntryPointAsset {
            name: entry.name.clone(),
            stage: format!("{:?}", entry.stage).to_ascii_lowercase(),
        })
        .collect()
}

fn descriptor(
    id: impl Into<String>,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Shader, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_shader_importer_capabilities() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"hlsl".to_string())));
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .capabilities
            .contains(&NAGA_IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn registration_contributes_module_and_shader_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 3);
    }

    #[test]
    fn wgsl_importer_validates_shader_asset() {
        let asset = import_fixture("main.wgsl", valid_wgsl(), Default::default());

        match asset {
            ImportedAsset::Shader(shader) => {
                assert_eq!(shader.source_language, ShaderSourceLanguage::Wgsl);
                assert_eq!(shader.entry_points.len(), 2);
                assert!(shader.wgsl_source.contains("vs_main"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn glsl_importer_emits_wgsl_shader_asset() {
        let asset = import_fixture("main.vert", valid_glsl_vertex(), Default::default());

        match asset {
            ImportedAsset::Shader(shader) => {
                assert_eq!(shader.source_language, ShaderSourceLanguage::Glsl);
                assert_eq!(shader.entry_points.len(), 1);
                assert_eq!(shader.entry_points[0].stage, "vertex");
                assert!(shader.wgsl_source.contains("@vertex"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn glsl_importer_uses_shader_stage_import_setting() {
        let mut settings = toml::Table::new();
        settings.insert("shader_stage".to_string(), "fragment".into());
        let asset = import_fixture("main.glsl", valid_glsl_fragment(), settings);

        match asset {
            ImportedAsset::Shader(shader) => {
                assert_eq!(shader.source_language, ShaderSourceLanguage::Glsl);
                assert_eq!(shader.entry_points[0].stage, "fragment");
                assert!(shader.wgsl_source.contains("@fragment"));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn invalid_glsl_returns_shader_error() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("broken.vert"))
            .unwrap();
        let context = context_for(
            "broken.vert",
            "#version 450\nvoid main( {",
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("wgsl validation failed"));
    }

    fn import_fixture(path: &str, source: &str, settings: toml::Table) -> ImportedAsset {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new(path))
            .unwrap();
        importer
            .import(&context_for(path, source, settings))
            .unwrap()
            .imported_asset
    }

    fn context_for(path: &str, source: &str, settings: toml::Table) -> AssetImportContext {
        let file_name = path.replace('\\', "/");
        let uri = format!("res://shaders/{file_name}");
        AssetImportContext::new(
            path.into(),
            zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
            source.as_bytes().to_vec(),
            settings,
        )
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

    fn valid_glsl_vertex() -> &'static str {
        r#"#version 450
layout(location = 0) in vec3 a_position;
void main() {
    gl_Position = vec4(a_position, 1.0);
}
"#
    }

    fn valid_glsl_fragment() -> &'static str {
        r#"#version 450
layout(location = 0) out vec4 o_color;
void main() {
    o_color = vec4(1.0, 0.4, 0.2, 1.0);
}
"#
    }
}
