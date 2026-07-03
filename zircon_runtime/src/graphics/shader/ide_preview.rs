use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use crate::asset::{AssetUri, ShaderAsset};
use crate::core::framework::render::{
    builtin_geometry_source_descriptor, ShaderAssetKind, ShaderIdePreviewSegment,
    ShaderIdePreviewVariant, GEOMETRY_SOURCE_ID_STATIC_MESH,
};

use super::template::{
    assemble_material_shader_template, MaterialShaderTemplateRequest, ShaderAssemblySegmentKind,
    ShaderTemplateAssemblyError, ShaderTemplateInclude,
};

const SURFACE_SHADER_ENTRY_POINT: &str = "zr_material_surface";
const DEFAULT_SURFACE_SHADER_MODULE_ID: &str = "self::surface";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderIdeSurfacePreview {
    pub wgsl_source: String,
    pub segments: Vec<ShaderIdePreviewSegment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShaderIdePreviewError {
    UnsupportedKind {
        uri: AssetUri,
        kind: ShaderAssetKind,
    },
    MissingRuntimeSource {
        uri: AssetUri,
    },
    MissingStaticGeometrySource,
    Assemble {
        uri: AssetUri,
        message: String,
    },
}

impl Display for ShaderIdePreviewError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedKind { uri, kind } => {
                write!(
                    f,
                    "shader IDE preview supports surface shaders only, but {uri} is {}",
                    kind.token()
                )
            }
            Self::MissingRuntimeSource { uri } => {
                write!(
                    f,
                    "shader IDE preview requires runtime WGSL source for {uri}"
                )
            }
            Self::MissingStaticGeometrySource => {
                write!(
                    f,
                    "shader IDE preview could not resolve static mesh geometry source"
                )
            }
            Self::Assemble { uri, message } => {
                write!(f, "assemble shader IDE preview for {uri}: {message}")
            }
        }
    }
}

impl std::error::Error for ShaderIdePreviewError {}

pub fn assemble_shader_ide_surface_preview<'a>(
    shader: &ShaderAsset,
    shader_includes: impl IntoIterator<Item = &'a ShaderAsset>,
    variant: &ShaderIdePreviewVariant,
) -> Result<ShaderIdeSurfacePreview, ShaderIdePreviewError> {
    if shader.kind != ShaderAssetKind::Surface {
        return Err(ShaderIdePreviewError::UnsupportedKind {
            uri: shader.uri.clone(),
            kind: shader.kind,
        });
    }
    let source = shader.runtime_wgsl_source().ok_or_else(|| {
        ShaderIdePreviewError::MissingRuntimeSource {
            uri: shader.uri.clone(),
        }
    })?;
    let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
        .ok_or(ShaderIdePreviewError::MissingStaticGeometrySource)?;
    let shader_index = shader_include_index(shader_includes);
    let module_includes = shader_module_include_sources(shader, &shader_index);
    let module_id = shader
        .import_path
        .as_deref()
        .filter(|path| !path.trim().is_empty())
        .unwrap_or(DEFAULT_SURFACE_SHADER_MODULE_ID)
        .to_string();
    let assembly = assemble_material_shader_template(
        MaterialShaderTemplateRequest::new(
            geometry_source,
            variant.pass_type,
            source,
            SURFACE_SHADER_ENTRY_POINT,
        )
        .with_generated_material_source(shader.generated_material_wgsl.as_str())
        .with_module_include_sources(module_includes)
        .with_material_option_defines(
            shader
                .material_option_table
                .definition_values_for_bits(variant.material_option_bits),
        )
        .with_material_surface_module_id(module_id),
    )
    .map_err(|error| ShaderIdePreviewError::Assemble {
        uri: shader.uri.clone(),
        message: shader_template_assembly_error_message(error),
    })?;

    Ok(ShaderIdeSurfacePreview {
        wgsl_source: assembly.wgsl_source,
        segments: assembly
            .segments
            .into_iter()
            .map(|segment| ShaderIdePreviewSegment {
                module_id: segment.module_id,
                kind: preview_segment_kind(segment.kind).to_string(),
                assembled_start_line: segment.assembled_start_line,
                assembled_line_count: segment.assembled_line_count,
                source_line_offset: segment.source_line_offset,
            })
            .collect(),
    })
}

fn shader_include_index<'a>(
    shader_includes: impl IntoIterator<Item = &'a ShaderAsset>,
) -> HashMap<String, &'a ShaderAsset> {
    shader_includes
        .into_iter()
        .map(|shader| (shader.uri.to_string(), shader))
        .collect()
}

fn shader_module_include_sources(
    shader: &ShaderAsset,
    shader_index: &HashMap<String, &ShaderAsset>,
) -> Vec<ShaderTemplateInclude> {
    let mut includes = Vec::new();
    let mut visited = HashSet::new();
    collect_shader_module_include_sources(shader, shader_index, &mut visited, &mut includes);
    includes
}

fn collect_shader_module_include_sources(
    shader: &ShaderAsset,
    shader_index: &HashMap<String, &ShaderAsset>,
    visited: &mut HashSet<String>,
    includes: &mut Vec<ShaderTemplateInclude>,
) {
    if !visited.insert(shader.uri.to_string()) {
        return;
    }
    for import in &shader.imports {
        let Some(reference) = import.redirect.as_ref() else {
            continue;
        };
        let key = reference.locator.to_string();
        let Some(import_shader) = shader_index.get(&key).copied() else {
            continue;
        };
        if import_shader.kind.is_include() {
            if let (Some(import_path), Some(source)) = (
                import_shader.import_path.as_ref(),
                import_shader.runtime_wgsl_source(),
            ) {
                includes.push(ShaderTemplateInclude::new(import_path.clone(), source));
            }
        }
        collect_shader_module_include_sources(import_shader, shader_index, visited, includes);
    }
}

fn preview_segment_kind(kind: ShaderAssemblySegmentKind) -> &'static str {
    match kind {
        ShaderAssemblySegmentKind::Defines => "defines",
        ShaderAssemblySegmentKind::Include => "include",
        ShaderAssemblySegmentKind::GeneratedMaterial => "generated_material",
        ShaderAssemblySegmentKind::UserMaterialSurface => "user_material_surface",
        ShaderAssemblySegmentKind::PassTemplate => "pass_template",
    }
}

fn shader_template_assembly_error_message(error: ShaderTemplateAssemblyError) -> String {
    format!("{error:?}")
}
