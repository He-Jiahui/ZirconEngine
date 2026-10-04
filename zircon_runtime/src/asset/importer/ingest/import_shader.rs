use super::validate_wgsl::{validate_naga_module, validate_wgsl};
use crate::asset::assets::{
    ImportedAsset, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};
use crate::core::framework::render::ShaderAssetKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShaderSourceKind {
    Wgsl,
    Glsl,
    SpirV,
}

impl ShaderSourceKind {
    fn from_extension(extension: &str) -> Option<Self> {
        match extension.len() {
            2 if ["vs", "fs", "cs"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate)) =>
            {
                Some(Self::Glsl)
            }
            3 if extension.eq_ignore_ascii_case("spv") => Some(Self::SpirV),
            4 if extension.eq_ignore_ascii_case("wgsl") => Some(Self::Wgsl),
            4 if ["glsl", "vert", "frag", "comp"]
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate)) =>
            {
                Some(Self::Glsl)
            }
            _ => None,
        }
    }
}

pub(crate) fn import_shader(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let extension = context
        .source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    match ShaderSourceKind::from_extension(extension) {
        Some(ShaderSourceKind::Wgsl) => import_wgsl(context),
        Some(ShaderSourceKind::Glsl) => import_glsl(context),
        Some(ShaderSourceKind::SpirV) => import_spirv(context),
        None => Err(AssetImportError::UnsupportedFormat(format!(
            "shader importer does not handle {}",
            context.source_path.display()
        ))),
    }
}

fn import_wgsl(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let (module, _info) = validate_wgsl(&context.uri, &source)?;
    let entry_points = shader_entry_points(&module);
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Shader(ShaderAsset {
            uri: context.uri.clone(),
            kind: ShaderAssetKind::Module,
            source_language: ShaderSourceLanguage::Wgsl,
            source: source.clone(),
            wgsl_source: source,
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
    let extension_stage = shader_stage_extension_hint(extension);
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
    let stage = stage.trim();
    if let Some(stage) = shader_stage_hint(stage) {
        return Ok(stage);
    }
    let normalized = stage.to_ascii_lowercase();
    Err(AssetImportError::Parse(format!(
        "unsupported shader stage `{normalized}`"
    )))
}

fn shader_stage_extension_hint(extension: &str) -> Option<naga::ShaderStage> {
    match extension.len() {
        2 if extension.eq_ignore_ascii_case("vs") => Some(naga::ShaderStage::Vertex),
        2 if extension.eq_ignore_ascii_case("fs") => Some(naga::ShaderStage::Fragment),
        2 if extension.eq_ignore_ascii_case("cs") => Some(naga::ShaderStage::Compute),
        4 if extension.eq_ignore_ascii_case("vert") => Some(naga::ShaderStage::Vertex),
        4 if extension.eq_ignore_ascii_case("frag") => Some(naga::ShaderStage::Fragment),
        4 if extension.eq_ignore_ascii_case("comp") => Some(naga::ShaderStage::Compute),
        _ => None,
    }
}

fn shader_stage_hint(stage: &str) -> Option<naga::ShaderStage> {
    let stage = stage.trim();
    match stage.len() {
        2 if stage.eq_ignore_ascii_case("vs") => Some(naga::ShaderStage::Vertex),
        2 if stage.eq_ignore_ascii_case("fs") => Some(naga::ShaderStage::Fragment),
        2 if stage.eq_ignore_ascii_case("cs") => Some(naga::ShaderStage::Compute),
        4 if stage.eq_ignore_ascii_case("vert") => Some(naga::ShaderStage::Vertex),
        4 if stage.eq_ignore_ascii_case("frag") => Some(naga::ShaderStage::Fragment),
        4 if stage.eq_ignore_ascii_case("comp") => Some(naga::ShaderStage::Compute),
        6 if stage.eq_ignore_ascii_case("vertex") => Some(naga::ShaderStage::Vertex),
        7 if stage.eq_ignore_ascii_case("compute") => Some(naga::ShaderStage::Compute),
        8 if stage.eq_ignore_ascii_case("fragment") => Some(naga::ShaderStage::Fragment),
        _ => None,
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

pub(super) fn shader_entry_points(module: &naga::Module) -> Vec<ShaderEntryPointAsset> {
    module
        .entry_points
        .iter()
        .map(|entry| ShaderEntryPointAsset {
            name: entry.name.clone(),
            stage: format!("{:?}", entry.stage).to_ascii_lowercase(),
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/import_shader_plugins07_builtin_shader_hotpath_tests.rs"]
mod plugins07_builtin_shader_hotpath_tests;
