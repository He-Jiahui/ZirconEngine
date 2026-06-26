use crate::core::framework::render::{GeometrySourceDescriptor, ShaderFeatureBits};

use super::assemble::{
    format_defines_header, push_include_chunk, rename_material_surface_entry,
    MaterialShaderTemplateAssembly, ShaderTemplateAssemblyError,
};
use super::include_registry::{
    geometry_source_include_for, gpu_scene_include, scene_runtime_include, surface_types_include,
    ShaderTemplateInclude, ShaderTemplateIncludeRegistry,
};
use super::pass_specialization::MATERIAL_SHADER_TEMPLATE_REVISION;

const DEFERRED_GBUFFER_TEMPLATE_TOKEN: &str = "zr_template_deferred_gbuffer.wgsl";
const DEFERRED_GBUFFER_TEMPLATE: &str = include_str!("../wgsl/zr_template_deferred_gbuffer.wgsl");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeferredGBufferShaderTemplateRequest {
    pub(crate) geometry_source: GeometrySourceDescriptor,
    pub(crate) features: ShaderFeatureBits,
    pub(crate) material_surface_source: String,
    pub(crate) material_surface_entry: String,
}

impl DeferredGBufferShaderTemplateRequest {
    pub(crate) fn new(
        geometry_source: GeometrySourceDescriptor,
        material_surface_source: impl Into<String>,
        material_surface_entry: impl Into<String>,
    ) -> Self {
        Self {
            geometry_source,
            features: ShaderFeatureBits::default(),
            material_surface_source: material_surface_source.into(),
            material_surface_entry: material_surface_entry.into(),
        }
    }

    pub(crate) fn with_features(mut self, features: ShaderFeatureBits) -> Self {
        self.features = features;
        self
    }
}

pub(crate) fn assemble_deferred_gbuffer_shader_template(
    request: DeferredGBufferShaderTemplateRequest,
) -> Result<MaterialShaderTemplateAssembly, ShaderTemplateAssemblyError> {
    let mut registry = ShaderTemplateIncludeRegistry::default();
    let mut chunks = Vec::new();

    chunks.push(format_defines_header(
        &request.geometry_source,
        request.features,
    ));

    push_include_chunk(&mut registry, &mut chunks, scene_runtime_include());
    push_include_chunk(&mut registry, &mut chunks, gpu_scene_include());
    push_include_chunk(&mut registry, &mut chunks, surface_types_include());

    let geometry_include =
        geometry_source_include_for(&request.geometry_source).ok_or_else(|| {
            ShaderTemplateAssemblyError::UnknownGeometryInclude {
                token: request.geometry_source.wgsl_include.clone(),
            }
        })?;
    push_include_chunk(&mut registry, &mut chunks, geometry_include);

    chunks.push(rename_material_surface_entry(
        &request.material_surface_source,
        &request.material_surface_entry,
    )?);
    push_include_chunk(
        &mut registry,
        &mut chunks,
        ShaderTemplateInclude::new(DEFERRED_GBUFFER_TEMPLATE_TOKEN, DEFERRED_GBUFFER_TEMPLATE),
    );

    Ok(MaterialShaderTemplateAssembly {
        wgsl_source: chunks.join("\n\n"),
        include_tokens: registry.include_tokens(),
        include_content_hashes: registry.content_hashes(),
        template_revision: MATERIAL_SHADER_TEMPLATE_REVISION.to_string(),
    })
}
