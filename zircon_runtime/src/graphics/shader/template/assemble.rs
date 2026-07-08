use crate::core::framework::render::{
    strip_wgsl_include_directives, GeometrySourceDescriptor, RenderShaderDefinitionValue,
    ShaderFeatureBits, ShaderPassType, ShadingModelDescriptor,
    GENERATED_MATERIAL_MODULE_IMPORT_PATH,
};
use crate::graphics::material::ShadingModelIncludeSourceSet;

use super::module_registry::{
    geometry_source_include_for, gpu_scene_include, scene_runtime_include,
    shading_model_forward_include_for, shading_model_forward_include_token, surface_types_include,
    ShaderModuleRegistry, ShaderModuleResolutionError, ShaderTemplateInclude,
    ShaderTemplateIncludeRegistry,
};
use super::pass_specialization::{pass_template_for, MATERIAL_SHADER_TEMPLATE_REVISION};

const MATERIAL_SURFACE_ENTRY_POINT: &str = "zr_material_surface";
const MATERIAL_DEFINES_MODULE_ID: &str = "zircon::template::defines";
const MATERIAL_SURFACE_MODULE_ID: &str = "self::surface";
const RESERVED_MATERIAL_SYMBOL_PREFIXES: &[&str] =
    &["zr_", "ZR_OPT_", "ZrMaterial", "fetch_", "shade_"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MaterialShaderTemplateRequest {
    pub(crate) geometry_source: GeometrySourceDescriptor,
    pub(crate) pass_type: ShaderPassType,
    pub(crate) features: ShaderFeatureBits,
    pub(crate) shading_model_descriptor: Option<ShadingModelDescriptor>,
    pub(crate) shading_model_forward_include_sources: Vec<ShaderTemplateInclude>,
    pub(crate) generated_material_source: Option<String>,
    pub(crate) module_include_sources: Vec<ShaderTemplateInclude>,
    pub(crate) material_option_defines: Vec<RenderShaderDefinitionValue>,
    pub(crate) material_surface_source: String,
    pub(crate) material_surface_entry: String,
    pub(crate) material_surface_module_id: String,
}

impl MaterialShaderTemplateRequest {
    pub(crate) fn new(
        geometry_source: GeometrySourceDescriptor,
        pass_type: ShaderPassType,
        material_surface_source: impl Into<String>,
        material_surface_entry: impl Into<String>,
    ) -> Self {
        Self {
            geometry_source,
            pass_type,
            features: ShaderFeatureBits::default(),
            shading_model_descriptor: None,
            shading_model_forward_include_sources: Vec::new(),
            generated_material_source: None,
            module_include_sources: Vec::new(),
            material_option_defines: Vec::new(),
            material_surface_source: material_surface_source.into(),
            material_surface_entry: material_surface_entry.into(),
            material_surface_module_id: MATERIAL_SURFACE_MODULE_ID.to_string(),
        }
    }

    pub(crate) fn with_features(mut self, features: ShaderFeatureBits) -> Self {
        self.features = features;
        self
    }

    pub(crate) fn with_shading_model_descriptor(
        mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Self {
        self.shading_model_descriptor = Some(descriptor);
        self
    }

    pub(crate) fn with_shading_model_forward_include_source(
        mut self,
        token: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        self.shading_model_forward_include_sources
            .push(ShaderTemplateInclude::new(token, source));
        self
    }

    pub(crate) fn with_shading_model_forward_include_sources(
        mut self,
        sources: &ShadingModelIncludeSourceSet,
    ) -> Self {
        for source in sources.forward() {
            self.shading_model_forward_include_sources
                .push(ShaderTemplateInclude::new(
                    source.token.clone(),
                    source.source.clone(),
                ));
        }
        self
    }

    pub(crate) fn with_generated_material_source(mut self, source: impl Into<String>) -> Self {
        let source = source.into();
        if !source.trim().is_empty() {
            self.generated_material_source = Some(source);
        }
        self
    }

    pub(crate) fn with_module_include_sources(
        mut self,
        includes: impl IntoIterator<Item = ShaderTemplateInclude>,
    ) -> Self {
        self.module_include_sources.extend(includes);
        self
    }

    pub(crate) fn with_material_option_defines(
        mut self,
        defines: impl IntoIterator<Item = RenderShaderDefinitionValue>,
    ) -> Self {
        self.material_option_defines.extend(defines);
        self
    }

    pub(crate) fn with_material_surface_module_id(mut self, module_id: impl Into<String>) -> Self {
        let module_id = module_id.into();
        if !module_id.trim().is_empty() {
            self.material_surface_module_id = module_id;
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MaterialShaderTemplateAssembly {
    pub(crate) wgsl_source: String,
    pub(crate) include_tokens: Vec<String>,
    pub(crate) include_content_hashes: Vec<String>,
    pub(crate) template_revision: String,
    pub(crate) segments: Vec<ShaderAssemblySegment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShaderAssemblySegmentKind {
    Defines,
    Include,
    GeneratedMaterial,
    UserMaterialSurface,
    PassTemplate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderAssemblySegment {
    pub(crate) module_id: String,
    pub(crate) kind: ShaderAssemblySegmentKind,
    pub(crate) assembled_start_line: u32,
    pub(crate) assembled_line_count: u32,
    pub(crate) source_line_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderAssemblySourceLocation {
    pub(crate) module_id: String,
    pub(crate) kind: ShaderAssemblySegmentKind,
    pub(crate) assembled_line: u32,
    pub(crate) local_line: u32,
}

#[derive(Default)]
pub(super) struct ShaderAssemblyBuilder {
    wgsl_source: String,
    segments: Vec<ShaderAssemblySegment>,
    current_line: u32,
}

impl ShaderAssemblyBuilder {
    pub(super) fn push(
        &mut self,
        module_id: impl Into<String>,
        kind: ShaderAssemblySegmentKind,
        source: impl Into<String>,
        source_line_offset: u32,
    ) {
        let source = source.into();
        if source.is_empty() {
            return;
        }
        if !self.wgsl_source.is_empty() {
            self.wgsl_source.push_str("\n\n");
            self.current_line += 2;
        }
        let line_count = shader_source_line_count(&source);
        self.segments.push(ShaderAssemblySegment {
            module_id: module_id.into(),
            kind,
            assembled_start_line: self.current_line + 1,
            assembled_line_count: line_count,
            source_line_offset,
        });
        self.wgsl_source.push_str(&source);
        self.current_line += line_count;
    }

    pub(super) fn finish(self) -> (String, Vec<ShaderAssemblySegment>) {
        (self.wgsl_source, self.segments)
    }
}

pub(crate) fn shader_assembly_source_location_for_line(
    segments: &[ShaderAssemblySegment],
    assembled_line: u32,
) -> Option<ShaderAssemblySourceLocation> {
    segments.iter().find_map(|segment| {
        let end_line = segment
            .assembled_start_line
            .saturating_add(segment.assembled_line_count.saturating_sub(1));
        if assembled_line < segment.assembled_start_line || assembled_line > end_line {
            return None;
        }
        let relative_line = assembled_line - segment.assembled_start_line + 1;
        if relative_line <= segment.source_line_offset {
            return None;
        }
        Some(ShaderAssemblySourceLocation {
            module_id: segment.module_id.clone(),
            kind: segment.kind,
            assembled_line,
            local_line: relative_line - segment.source_line_offset,
        })
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShaderTemplateAssemblyError {
    UnknownGeometryInclude {
        token: String,
    },
    UnknownShadingInclude {
        token: String,
    },
    MissingSurfaceEntry {
        entry: String,
    },
    DuplicateSurfaceEntry {
        entry: String,
    },
    ReservedMaterialSymbol {
        symbol: String,
        prefix: &'static str,
    },
    UnknownModuleInclude {
        token: String,
    },
    CircularModuleInclude {
        cycle: Vec<String>,
    },
}

pub(crate) fn assemble_material_shader_template(
    request: MaterialShaderTemplateRequest,
) -> Result<MaterialShaderTemplateAssembly, ShaderTemplateAssemblyError> {
    let pass_template = pass_template_for(request.pass_type, request.features);
    let mut registry = ShaderTemplateIncludeRegistry::default();
    let mut builder = ShaderAssemblyBuilder::default();

    builder.push(
        MATERIAL_DEFINES_MODULE_ID,
        ShaderAssemblySegmentKind::Defines,
        format_defines_header(
            &request.geometry_source,
            request.features,
            &request.material_option_defines,
        ),
        0,
    );

    push_include_chunk(&mut registry, &mut builder, scene_runtime_include());
    push_include_chunk(&mut registry, &mut builder, gpu_scene_include());
    push_include_chunk(&mut registry, &mut builder, surface_types_include());

    let geometry_include =
        geometry_source_include_for(&request.geometry_source).ok_or_else(|| {
            ShaderTemplateAssemblyError::UnknownGeometryInclude {
                token: request.geometry_source.wgsl_include.clone(),
            }
        })?;
    push_include_chunk(&mut registry, &mut builder, geometry_include);

    for include in pass_template.support_includes.iter().cloned() {
        push_include_chunk(&mut registry, &mut builder, include);
    }

    if pass_template.requires_shading_include {
        let shading_include = shading_model_forward_include_for(
            request.shading_model_descriptor.as_ref(),
            &request.shading_model_forward_include_sources,
        )
        .ok_or_else(|| ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: shading_model_forward_include_token(request.shading_model_descriptor.as_ref())
                .to_string(),
        })?;
        push_include_chunk(&mut registry, &mut builder, shading_include);
    }

    if pass_template.requires_material_surface {
        if let Some(source) = request.generated_material_source.as_ref() {
            push_include_chunk(
                &mut registry,
                &mut builder,
                generated_material_include(source.clone()),
            );
        }
        push_source_module_includes(
            &mut registry,
            &mut builder,
            &request.material_surface_source,
            &request.module_include_sources,
        )?;
        builder.push(
            request.material_surface_module_id,
            ShaderAssemblySegmentKind::UserMaterialSurface,
            rename_material_surface_entry(
                &strip_wgsl_include_directives(&request.material_surface_source),
                &request.material_surface_entry,
            )?,
            0,
        );
    }

    push_include_chunk(&mut registry, &mut builder, pass_template.include);
    let (wgsl_source, segments) = builder.finish();

    Ok(MaterialShaderTemplateAssembly {
        wgsl_source,
        include_tokens: registry.include_tokens(),
        include_content_hashes: registry.content_hashes(),
        template_revision: MATERIAL_SHADER_TEMPLATE_REVISION.to_string(),
        segments,
    })
}

pub(super) fn generated_material_include(source: impl Into<String>) -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(GENERATED_MATERIAL_MODULE_IMPORT_PATH, source)
}

pub(super) fn push_include_chunk(
    registry: &mut ShaderTemplateIncludeRegistry,
    builder: &mut ShaderAssemblyBuilder,
    include: ShaderTemplateInclude,
) {
    let kind = include_segment_kind(&include.token);
    let module_id = include.token.clone();
    let chunk = format!("// include: {}\n{}", include.token, include.source);
    if registry.push(include) {
        builder.push(module_id, kind, chunk, 1);
    }
}

pub(super) fn push_source_module_includes(
    registry: &mut ShaderTemplateIncludeRegistry,
    builder: &mut ShaderAssemblyBuilder,
    source: &str,
    module_include_sources: &[ShaderTemplateInclude],
) -> Result<(), ShaderTemplateAssemblyError> {
    let mut module_registry = ShaderModuleRegistry::with_builtin_modules();
    for include in module_include_sources.iter().cloned() {
        module_registry.register(include);
    }
    let resolved = module_registry
        .resolve_for_source(source)
        .map_err(shader_module_resolution_error)?;
    for include in resolved.ordered_sources {
        push_include_chunk(registry, builder, include);
    }
    Ok(())
}

fn include_segment_kind(token: &str) -> ShaderAssemblySegmentKind {
    if token == GENERATED_MATERIAL_MODULE_IMPORT_PATH {
        ShaderAssemblySegmentKind::GeneratedMaterial
    } else if token.starts_with("zr_template_") {
        ShaderAssemblySegmentKind::PassTemplate
    } else {
        ShaderAssemblySegmentKind::Include
    }
}

fn shader_source_line_count(source: &str) -> u32 {
    if source.is_empty() {
        0
    } else {
        source.lines().count() as u32
    }
}

fn shader_module_resolution_error(
    error: ShaderModuleResolutionError,
) -> ShaderTemplateAssemblyError {
    match error {
        ShaderModuleResolutionError::UnknownModule { token } => {
            ShaderTemplateAssemblyError::UnknownModuleInclude { token }
        }
        ShaderModuleResolutionError::CircularDependency { cycle } => {
            ShaderTemplateAssemblyError::CircularModuleInclude { cycle }
        }
    }
}

pub(super) fn format_defines_header(
    geometry_source: &GeometrySourceDescriptor,
    features: ShaderFeatureBits,
    material_option_defines: &[RenderShaderDefinitionValue],
) -> String {
    let mut lines = vec![
        "// generated by zircon shader template assembler".to_string(),
        format!(
            "const ZR_GEOMETRY_SOURCE_TOKEN: u32 = {};",
            geometry_source.id.value()
        ),
        format!(
            "const ZR_FEATURE_ALPHA_TEST: bool = {};",
            features.contains(ShaderFeatureBits::ALPHA_TEST)
        ),
        format!(
            "const ZR_FEATURE_RECEIVE_SHADOWS: bool = {};",
            features.contains(ShaderFeatureBits::RECEIVE_SHADOWS)
        ),
        format!(
            "const ZR_FEATURE_DOUBLE_SIDED: bool = {};",
            features.contains(ShaderFeatureBits::DOUBLE_SIDED)
        ),
        format!(
            "const ZR_FEATURE_HAS_NORMAL_TEXTURE: bool = {};",
            features.contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE)
        ),
    ];
    for define in &geometry_source.shader_defines {
        lines.push(format_definition_value(define));
    }
    for define in material_option_defines {
        lines.push(format_definition_value(define));
    }
    lines.join("\n")
}

fn format_definition_value(define: &RenderShaderDefinitionValue) -> String {
    let name = define.normalized_name();
    match define {
        RenderShaderDefinitionValue::Bool { value, .. } => {
            format!("const {name}: bool = {value};")
        }
        RenderShaderDefinitionValue::Int { value, .. } => {
            format!("const {name}: i32 = {value};")
        }
        RenderShaderDefinitionValue::UInt { value, .. } => {
            format!("const {name}: u32 = {value}u;")
        }
    }
}

pub(super) fn rename_material_surface_entry(
    source: &str,
    entry: &str,
) -> Result<String, ShaderTemplateAssemblyError> {
    let entry = entry.trim();
    reject_reserved_material_symbols(source, entry)?;

    let needle = format!("fn {entry}(");
    let count = source.match_indices(&needle).count();
    if count == 0 {
        return Err(ShaderTemplateAssemblyError::MissingSurfaceEntry {
            entry: entry.to_string(),
        });
    }
    if count > 1 {
        return Err(ShaderTemplateAssemblyError::DuplicateSurfaceEntry {
            entry: entry.to_string(),
        });
    }
    if entry == MATERIAL_SURFACE_ENTRY_POINT {
        return Ok(source.to_string());
    }
    Ok(source.replacen(&needle, &format!("fn {MATERIAL_SURFACE_ENTRY_POINT}("), 1))
}

fn reject_reserved_material_symbols(
    source: &str,
    surface_entry: &str,
) -> Result<(), ShaderTemplateAssemblyError> {
    for symbol in declared_functions(source) {
        if symbol == MATERIAL_SURFACE_ENTRY_POINT && surface_entry == MATERIAL_SURFACE_ENTRY_POINT {
            continue;
        }
        if let Some(prefix) = RESERVED_MATERIAL_SYMBOL_PREFIXES
            .iter()
            .copied()
            .find(|prefix| symbol.starts_with(prefix))
        {
            return Err(ShaderTemplateAssemblyError::ReservedMaterialSymbol {
                symbol: symbol.to_string(),
                prefix,
            });
        }
    }
    Ok(())
}

fn declared_functions(source: &str) -> impl Iterator<Item = &str> {
    source.lines().filter_map(|line| {
        let trimmed = line.trim_start();
        let rest = trimmed.strip_prefix("fn ")?;
        rest.split_once('(').map(|(name, _)| name.trim())
    })
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        builtin_geometry_source_descriptor, ShaderPassType, GENERATED_MATERIAL_MODULE_IMPORT_PATH,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
    };

    use super::{
        assemble_material_shader_template, shader_assembly_source_location_for_line,
        MaterialShaderTemplateRequest, ShaderAssemblySegmentKind, MATERIAL_SURFACE_ENTRY_POINT,
    };

    const GENERATED_MATERIAL: &str = r#"
fn generated_material_value() -> vec4<f32> {
    return vec4<f32>(0.2, 0.4, 0.8, 1.0);
}
"#;

    const USER_SURFACE: &str = r#"
#include <self::material>

fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(generated_material_value() + input.color * 0.0);
}
"#;

    #[test]
    fn shader_template_assembly_records_source_segments_for_diagnostics() {
        let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
            .expect("static geometry source");
        let assembly = assemble_material_shader_template(
            MaterialShaderTemplateRequest::new(
                geometry_source,
                ShaderPassType::Forward,
                USER_SURFACE,
                "user_surface",
            )
            .with_generated_material_source(GENERATED_MATERIAL)
            .with_material_surface_module_id("project::materials::hero"),
        )
        .expect("template assembly");

        let generated_segment = assembly
            .segments
            .iter()
            .find(|segment| segment.module_id == GENERATED_MATERIAL_MODULE_IMPORT_PATH)
            .expect("generated material segment");
        let generated_source_line = generated_segment.assembled_start_line + 1;
        let generated_location =
            shader_assembly_source_location_for_line(&assembly.segments, generated_source_line)
                .expect("generated material source location");
        assert_eq!(
            generated_location.module_id,
            GENERATED_MATERIAL_MODULE_IMPORT_PATH
        );
        assert_eq!(
            generated_location.kind,
            ShaderAssemblySegmentKind::GeneratedMaterial
        );
        assert_eq!(generated_location.local_line, 1);

        let surface_line = assembly
            .wgsl_source
            .lines()
            .position(|line| line.contains(MATERIAL_SURFACE_ENTRY_POINT))
            .expect("renamed material surface line") as u32
            + 1;
        let surface_location =
            shader_assembly_source_location_for_line(&assembly.segments, surface_line)
                .expect("surface source location");
        assert_eq!(surface_location.module_id, "project::materials::hero");
        assert_eq!(
            surface_location.kind,
            ShaderAssemblySegmentKind::UserMaterialSurface
        );
        assert_eq!(surface_location.local_line, 2);
    }
}
