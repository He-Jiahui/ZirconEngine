use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use crate::asset::{AssetUri, ShaderAsset};
use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_STATIC_MESH, ShaderAssetKind, ShaderIdePreviewSegment,
    ShaderIdePreviewVariant, builtin_geometry_source_descriptor,
};

use super::template::{
    MaterialShaderTemplateRequest, ShaderAssemblySegmentKind, ShaderTemplateAssemblyError,
    ShaderTemplateInclude, assemble_material_shader_template,
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
    let shader_index = shader_include_index(shader_includes);
    assemble_shader_ide_surface_preview_with_index(shader, &shader_index, variant)
}

pub(super) fn assemble_shader_ide_surface_preview_with_index(
    shader: &ShaderAsset,
    shader_index: &HashMap<String, &ShaderAsset>,
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
    let module_includes = shader_module_include_sources(shader, shader_index);
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
        segments: {
            let mut output = Vec::with_capacity(assembly.segments.len());
            for segment in assembly.segments {
                output.push(ShaderIdePreviewSegment {
                    module_id: segment.module_id,
                    kind: preview_segment_kind(segment.kind).to_string(),
                    assembled_start_line: segment.assembled_start_line,
                    assembled_line_count: segment.assembled_line_count,
                    source_line_offset: segment.source_line_offset,
                });
            }
            output
        },
    })
}

pub(super) fn shader_include_index<'a>(
    shader_includes: impl IntoIterator<Item = &'a ShaderAsset>,
) -> HashMap<String, &'a ShaderAsset> {
    let mut shader_includes = shader_includes.into_iter();
    let (lower_bound, upper_bound) = shader_includes.size_hint();
    let mut index = HashMap::with_capacity(upper_bound.unwrap_or(lower_bound));
    for shader in shader_includes {
        index.insert(shader.uri.to_string(), shader);
    }
    index
}

fn shader_module_include_sources(
    shader: &ShaderAsset,
    shader_index: &HashMap<String, &ShaderAsset>,
) -> Vec<ShaderTemplateInclude> {
    let mut includes = Vec::with_capacity(shader.imports.len());
    let mut visited = HashSet::with_capacity(shader.imports.len());
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

#[cfg(test)]
mod optimization_batch_20260830ca_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const ENTRIES_PER_SAMPLE: usize = 256;

    #[test]
    fn shader_preview_reserves_segment_index_and_include_capacity() {
        let source = include_str!("ide_preview.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(assembly.segments.len())"));
        assert!(
            implementation.contains("HashMap::with_capacity(upper_bound.unwrap_or(lower_bound))")
        );
        assert!(implementation.contains("Vec::with_capacity(shader.imports.len())"));
        assert!(implementation.contains("HashSet::with_capacity(shader.imports.len())"));
    }

    #[test]
    fn shader_preview_keeps_assembly_before_recursive_include_collection() {
        let source = include_str!("ide_preview.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let assembly = implementation
            .find("assemble_material_shader_template")
            .expect("assembly");
        let include = implementation
            .find("collect_shader_module_include_sources")
            .expect("recursive include collection");
        assert!(assembly < include);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830ca_runtime_shader_preview_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME379_SHADER_IDE_PREVIEW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entries_per_sample={ENTRIES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut segments = if optimized {
                Vec::with_capacity(ENTRIES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            let mut includes = if optimized {
                Vec::with_capacity(ENTRIES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..ENTRIES_PER_SAMPLE {
                segments.push(index);
                includes.push(index);
            }
            checksum ^= segments.len() ^ includes.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
