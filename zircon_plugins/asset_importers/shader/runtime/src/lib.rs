use naga::valid::{Capabilities, ValidationFlags, Validator};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, ShaderAsset,
    ShaderEntryPointAsset, ShaderSourceLanguage,
};
use zircon_runtime::core::framework::render::ShaderAssetKind;

mod capability;
mod plugin;

pub use capability::{
    IMPORTER_FAMILY, MODULE_NAME, NAGA_IMPORTER_CAPABILITY, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST,
    PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME, SHADER_ASSET_IMPORTER_DECLARATION,
    WGSL_IMPORTER_CAPABILITY,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    ShaderAssetImporterRuntimePlugin, SHADER_ASSET_IMPORTER_DIST_CRATE_NAME,
    SHADER_ASSET_IMPORTER_DIST_RUNTIME_ENTRY,
};

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
    let source = context.source_str()?;
    let module = naga::front::wgsl::parse_str(source).map_err(|error| {
        AssetImportError::ShaderValidation(format!(
            "{}: {}",
            context.uri,
            error.emit_to_string(source)
        ))
    })?;
    module_to_shader_asset(context, ShaderSourceLanguage::Wgsl, source, module)
}

fn import_glsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let stage = infer_shader_stage(context)?;
    let mut frontend = naga::front::glsl::Frontend::default();
    let module = frontend
        .parse(&naga::front::glsl::Options::from(stage), source)
        .map_err(|error| {
            AssetImportError::ShaderValidation(format!(
                "{}: {}",
                context.uri,
                error.emit_to_string(source)
            ))
        })?;
    let info = validate_naga_module(context, &module)?;
    let wgsl_source = module_to_wgsl(context, &module, &info)?;
    shader_outcome(
        context,
        ShaderSourceLanguage::Glsl,
        source.to_owned(),
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
    source: &str,
    module: naga::Module,
) -> Result<AssetImportOutcome, AssetImportError> {
    validate_naga_module(context, &module)?;
    let source = source.to_owned();
    shader_outcome(
        context,
        source_language,
        source.clone(),
        source,
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
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Shader(ShaderAsset {
            uri: context.uri.clone(),
            kind: ShaderAssetKind::Module,
            source_language,
            source,
            wgsl_source,
            import_path: None,
            entry_points,
            dependencies: Vec::new(),
            source_files: Vec::new(),
            imports: Vec::new(),
            shader_defs: Vec::new(),
            property_schema: Vec::new(),
            options: Vec::new(),
            texture_slots: Vec::new(),
            shading_model: None,
            render_state: Default::default(),
            queue: None,
            disabled_passes: Vec::new(),
            resources: Vec::new(),
            material_property_layout: Default::default(),
            material_option_table: Default::default(),
            generated_material_wgsl: String::new(),
            editor: Default::default(),
            pipeline_layout: Default::default(),
            validation_diagnostics: Vec::new(),
        }),
    ))
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
        .import_settings()
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
            stage: shader_stage_name(&entry.stage).to_owned(),
        })
        .collect()
}

fn shader_stage_name(stage: &naga::ShaderStage) -> &'static str {
    match stage {
        naga::ShaderStage::Vertex => "vertex",
        naga::ShaderStage::Task => "task",
        naga::ShaderStage::Mesh => "mesh",
        naga::ShaderStage::Fragment => "fragment",
        naga::ShaderStage::Compute => "compute",
        naga::ShaderStage::RayGeneration => "raygeneration",
        naga::ShaderStage::Miss => "miss",
        naga::ShaderStage::AnyHit => "anyhit",
        naga::ShaderStage::ClosestHit => "closesthit",
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
