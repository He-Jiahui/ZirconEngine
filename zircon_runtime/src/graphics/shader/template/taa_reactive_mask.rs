use crate::core::framework::render::{
    GeometrySourceDescriptor, RenderShaderDefinitionValue, ShaderFeatureBits,
    strip_wgsl_include_directives,
};

use super::assemble::{
    MaterialShaderTemplateAssembly, ShaderAssemblyBuilder, ShaderAssemblySegmentKind,
    ShaderTemplateAssemblyError, format_defines_header, generated_material_include,
    push_include_chunk, push_source_module_includes, rename_material_surface_entry,
};
use super::module_registry::{
    ShaderTemplateInclude, ShaderTemplateIncludeRegistry, geometry_source_include_for,
    gpu_scene_include, scene_runtime_include, surface_types_include,
};
use super::pass_specialization::MATERIAL_SHADER_TEMPLATE_REVISION;

const TAA_REACTIVE_MASK_TEMPLATE_TOKEN: &str = "zr_template_taa_reactive_mask.wgsl";
const TAA_REACTIVE_MASK_TEMPLATE: &str = include_str!("../wgsl/zr_template_taa_reactive_mask.wgsl");
const TAA_REACTIVE_MASK_DEFINES_MODULE_ID: &str = "zircon::template::defines";
const TAA_REACTIVE_MASK_SURFACE_MODULE_ID: &str = "self::surface";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaaReactiveMaskShaderTemplateRequest {
    pub(crate) geometry_source: GeometrySourceDescriptor,
    pub(crate) features: ShaderFeatureBits,
    pub(crate) generated_material_source: Option<String>,
    pub(crate) module_include_sources: Vec<ShaderTemplateInclude>,
    pub(crate) material_option_defines: Vec<RenderShaderDefinitionValue>,
    pub(crate) material_surface_source: String,
    pub(crate) material_surface_entry: String,
    pub(crate) material_surface_module_id: String,
}

impl TaaReactiveMaskShaderTemplateRequest {
    pub(crate) fn new(
        geometry_source: GeometrySourceDescriptor,
        material_surface_source: impl Into<String>,
        material_surface_entry: impl Into<String>,
    ) -> Self {
        Self {
            geometry_source,
            features: ShaderFeatureBits::default(),
            generated_material_source: None,
            module_include_sources: Vec::new(),
            material_option_defines: Vec::new(),
            material_surface_source: material_surface_source.into(),
            material_surface_entry: material_surface_entry.into(),
            material_surface_module_id: TAA_REACTIVE_MASK_SURFACE_MODULE_ID.to_string(),
        }
    }

    pub(crate) fn with_features(mut self, features: ShaderFeatureBits) -> Self {
        self.features = features;
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

pub(crate) fn assemble_taa_reactive_mask_shader_template(
    request: TaaReactiveMaskShaderTemplateRequest,
) -> Result<MaterialShaderTemplateAssembly, ShaderTemplateAssemblyError> {
    let mut registry = ShaderTemplateIncludeRegistry::default();
    let mut builder = ShaderAssemblyBuilder::default();

    builder.push(
        TAA_REACTIVE_MASK_DEFINES_MODULE_ID,
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
    push_include_chunk(
        &mut registry,
        &mut builder,
        ShaderTemplateInclude::new(TAA_REACTIVE_MASK_TEMPLATE_TOKEN, TAA_REACTIVE_MASK_TEMPLATE),
    );
    let (wgsl_source, segments) = builder.finish();
    let (include_tokens, include_content_hashes) = registry.into_manifest();

    Ok(MaterialShaderTemplateAssembly {
        wgsl_source,
        include_tokens,
        include_content_hashes,
        template_revision: MATERIAL_SHADER_TEMPLATE_REVISION.to_string(),
        segments,
    })
}
