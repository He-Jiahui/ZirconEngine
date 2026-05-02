use super::validate_wgsl::{validate_naga_module, validate_wgsl};
use crate::asset::assets::{
    ImportedAsset, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_shader(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
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
    let (module, _info) = validate_wgsl(&context.uri, &source)?;
    let entry_points = shader_entry_points(&module);
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
    module_to_shader_asset(context, ShaderSourceLanguage::Glsl, source, module)
}

fn import_spirv(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let module = naga::front::spv::parse_u8_slice(
        &context.source_bytes,
        &naga::front::spv::Options::default(),
    )
    .map_err(|error| {
        AssetImportError::ShaderValidation(format!("{}: spir-v parse failed: {error}", context.uri))
    })?;
    module_to_shader_asset(
        context,
        ShaderSourceLanguage::SpirV,
        hex_encode(&context.source_bytes),
        module,
    )
}

fn module_to_shader_asset(
    context: &AssetImportContext,
    source_language: ShaderSourceLanguage,
    source: String,
    module: naga::Module,
) -> Result<AssetImportOutcome, AssetImportError> {
    let info = validate_naga_module(&context.uri, &module)?;
    let wgsl_source =
        naga::back::wgsl::write_string(&module, &info, naga::back::wgsl::WriterFlags::empty())
            .map_err(|error| {
                AssetImportError::ShaderValidation(format!(
                    "{}: wgsl emission failed: {error}",
                    context.uri
                ))
            })?;
    let entry_points = shader_entry_points(&module);
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
