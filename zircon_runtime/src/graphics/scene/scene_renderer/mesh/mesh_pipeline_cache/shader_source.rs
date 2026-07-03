use crate::core::framework::render::{
    builtin_geometry_source_descriptor, GeometrySourceDescriptor, GeometrySourceId, ShaderPassType,
    GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};
use crate::graphics::shader::{
    assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
    assemble_taa_reactive_mask_shader_template, standard_material_surface_source_for_features,
    DeferredGBufferShaderTemplateRequest, MaterialShaderTemplateAssembly,
    MaterialShaderTemplateRequest, ShaderTemplateAssemblyError,
    TaaReactiveMaskShaderTemplateRequest,
};

const MESH_SHADER_TEMPLATE_REVISION: &str = "mesh-template-v1";
const SURFACE_SHADER_ENTRY_POINT: &str = "zr_material_surface";
const DEFAULT_SURFACE_SHADER_MODULE_ID: &str = "self::surface";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MeshPipelineShaderSource {
    pub(crate) wgsl_source: String,
    pub(crate) source_hash: String,
    pub(crate) cache_content_hashes: Vec<String>,
    pub(crate) template_revision: String,
}

impl MeshPipelineShaderSource {
    fn from_template(assembly: MaterialShaderTemplateAssembly) -> Self {
        let source_hash = mesh_pipeline_wgsl_hash(&assembly.wgsl_source);
        let mut cache_content_hashes = assembly.include_content_hashes;
        cache_content_hashes.push(source_hash.clone());
        Self {
            wgsl_source: assembly.wgsl_source,
            source_hash,
            cache_content_hashes,
            template_revision: assembly.template_revision,
        }
    }

    fn from_raw_wgsl(source: &str) -> Self {
        let source_hash = mesh_pipeline_wgsl_hash(source);
        Self {
            wgsl_source: source.to_string(),
            source_hash: source_hash.clone(),
            cache_content_hashes: vec![source_hash],
            template_revision: MESH_SHADER_TEMPLATE_REVISION.to_string(),
        }
    }
}

pub(crate) fn mesh_pipeline_shader_source_for_geometry_descriptor(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    if key.uses_fallback_shader() {
        mesh_pipeline_standard_material_template_source_for_geometry_descriptor_with_streamer(
            streamer,
            key,
            geometry_source,
        )
    } else if shader_source_uses_runtime_material_surface(streamer, key) {
        mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass_with_streamer(
            streamer,
            key,
            geometry_source.clone(),
            ShaderPassType::Forward,
        )
    } else {
        streamer
            .shader_source(&key.shader_id)
            .map(MeshPipelineShaderSource::from_raw_wgsl)
            .map_or_else(
                || {
                    mesh_pipeline_standard_material_template_source_for_geometry_descriptor_with_streamer(
                        streamer,
                        key,
                        geometry_source,
                    )
                },
                Ok,
            )
    }
}

pub(crate) fn mesh_pipeline_standard_material_template_source(
    key: &PipelineKey,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_standard_material_template_source_for_geometry(
        key,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
    )
}

pub(crate) fn mesh_pipeline_standard_material_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    mesh_pipeline_standard_material_template_source_for_geometry_descriptor(key, &geometry_source)
}

pub(crate) fn mesh_pipeline_standard_material_template_source_for_geometry_descriptor(
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
        key,
        geometry_source.clone(),
        ShaderPassType::Forward,
    )
}

fn mesh_pipeline_standard_material_template_source_for_geometry_descriptor_with_streamer(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass_with_streamer(
        streamer,
        key,
        geometry_source.clone(),
        ShaderPassType::Forward,
    )
}

pub(crate) fn mesh_pipeline_standard_material_template_source_for_shader_pass(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor(
        key,
        &geometry_source,
        pass_type,
    )
}

pub(crate) fn mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor(
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    match pass_type {
        ShaderPassType::GBuffer => {
            mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor(
                key,
                geometry_source,
            )
        }
        ShaderPassType::TaaReactiveMask => {
            mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor(
                key,
                geometry_source,
            )
        }
        _ => mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
            key,
            geometry_source.clone(),
            pass_type,
        ),
    }
}

pub(crate) fn mesh_pipeline_velocity_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    mesh_pipeline_velocity_template_source_for_geometry_descriptor(key, &geometry_source)
}

pub(crate) fn mesh_pipeline_velocity_template_source_for_geometry_descriptor(
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
        key,
        geometry_source.clone(),
        ShaderPassType::Velocity,
    )
}

pub(crate) fn mesh_pipeline_velocity_template_source_for_geometry_descriptor_with_streamer(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass_with_streamer(
        streamer,
        key,
        geometry_source.clone(),
        ShaderPassType::Velocity,
    )
}

pub(crate) fn mesh_pipeline_depth_prepass_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    mesh_pipeline_depth_prepass_template_source_for_geometry_descriptor(key, &geometry_source)
}

pub(crate) fn mesh_pipeline_depth_prepass_template_source_for_geometry_descriptor(
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
        key,
        geometry_source.clone(),
        ShaderPassType::DepthPrepass,
    )
}

pub(crate) fn mesh_pipeline_depth_prepass_template_source_for_geometry_descriptor_with_streamer(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass_with_streamer(
        streamer,
        key,
        geometry_source.clone(),
        ShaderPassType::DepthPrepass,
    )
}

pub(crate) fn mesh_pipeline_deferred_gbuffer_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor(key, &geometry_source)
}

pub(crate) fn mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor(
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let material_surface = standard_material_surface_source_for_features(
        key.shader_feature_bits(),
        mesh_pipeline_alpha_cutoff(key),
    );
    let request = DeferredGBufferShaderTemplateRequest::new(
        geometry_source.clone(),
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);

    assemble_deferred_gbuffer_shader_template(request).map(MeshPipelineShaderSource::from_template)
}

pub(crate) fn mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    if key.uses_fallback_shader() {
        return mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor(
            key,
            geometry_source,
        );
    }

    if shader_source_uses_runtime_material_surface(streamer, key) {
        let Some(surface_source) = streamer.shader_source(&key.shader_id) else {
            return mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor(
                key,
                geometry_source,
            );
        };
        let mut request = DeferredGBufferShaderTemplateRequest::new(
            geometry_source.clone(),
            surface_source,
            SURFACE_SHADER_ENTRY_POINT,
        )
        .with_features(key.shader_feature_bits())
        .with_material_surface_module_id(runtime_surface_module_id(streamer, key));
        if let Some(generated_material_source) =
            streamer.shader_generated_material_source(&key.shader_id)
        {
            request = request.with_generated_material_source(generated_material_source);
        }
        let request = with_runtime_gbuffer_material_modules_and_options(request, streamer, key);
        let request = with_runtime_gbuffer_shading_model_sources(request, streamer, key)?;
        return assemble_deferred_gbuffer_shader_template(request)
            .map(MeshPipelineShaderSource::from_template);
    }

    let material_surface = standard_material_surface_source_for_features(
        key.shader_feature_bits(),
        mesh_pipeline_alpha_cutoff(key),
    );
    let request = DeferredGBufferShaderTemplateRequest::new(
        geometry_source.clone(),
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);
    let request = with_runtime_gbuffer_shading_model_sources(request, streamer, key)?;

    assemble_deferred_gbuffer_shader_template(request).map(MeshPipelineShaderSource::from_template)
}

pub(crate) fn mesh_pipeline_shadow_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    mesh_pipeline_shadow_template_source_for_geometry_descriptor(key, &geometry_source)
}

pub(crate) fn mesh_pipeline_shadow_template_source_for_geometry_descriptor(
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
        key,
        geometry_source.clone(),
        ShaderPassType::Shadow,
    )
}

pub(crate) fn mesh_pipeline_shadow_template_source_for_geometry_descriptor_with_streamer(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass_with_streamer(
        streamer,
        key,
        geometry_source.clone(),
        ShaderPassType::Shadow,
    )
}

pub(crate) fn mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor(key, &geometry_source)
}

pub(crate) fn mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor(
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let material_surface = standard_material_surface_source_for_features(
        key.shader_feature_bits(),
        mesh_pipeline_alpha_cutoff(key),
    );
    let request = TaaReactiveMaskShaderTemplateRequest::new(
        geometry_source.clone(),
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);

    assemble_taa_reactive_mask_shader_template(request).map(MeshPipelineShaderSource::from_template)
}

pub(crate) fn mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor_with_streamer(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    if key.uses_fallback_shader() {
        return mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor(
            key,
            geometry_source,
        );
    }

    if shader_source_uses_runtime_material_surface(streamer, key) {
        let Some(surface_source) = streamer.shader_source(&key.shader_id) else {
            return mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor(
                key,
                geometry_source,
            );
        };
        let mut request = TaaReactiveMaskShaderTemplateRequest::new(
            geometry_source.clone(),
            surface_source,
            SURFACE_SHADER_ENTRY_POINT,
        )
        .with_features(key.shader_feature_bits())
        .with_material_surface_module_id(runtime_surface_module_id(streamer, key));
        if let Some(generated_material_source) =
            streamer.shader_generated_material_source(&key.shader_id)
        {
            request = request.with_generated_material_source(generated_material_source);
        }
        let request = request
            .with_module_include_sources(streamer.shader_module_include_sources(&key.shader_id))
            .with_material_option_defines(
                streamer.shader_material_option_defines(&key.shader_id, key.material_option_bits),
            );
        return assemble_taa_reactive_mask_shader_template(request)
            .map(MeshPipelineShaderSource::from_template);
    }

    mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor(key, geometry_source)
}

fn mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
    key: &PipelineKey,
    geometry_source: GeometrySourceDescriptor,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let material_surface = standard_material_surface_source_for_features(
        key.shader_feature_bits(),
        mesh_pipeline_alpha_cutoff(key),
    );
    let request = MaterialShaderTemplateRequest::new(
        geometry_source,
        pass_type,
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);

    assemble_material_shader_template(request).map(MeshPipelineShaderSource::from_template)
}

fn mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass_with_streamer(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: GeometrySourceDescriptor,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    if key.uses_fallback_shader() {
        return mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
            key,
            geometry_source,
            pass_type,
        );
    }

    if shader_source_uses_runtime_material_surface(streamer, key) {
        let Some(surface_source) = streamer.shader_source(&key.shader_id) else {
            return mesh_pipeline_material_template_source_for_geometry_descriptor_and_pass(
                key,
                geometry_source,
                pass_type,
            );
        };
        let mut request = MaterialShaderTemplateRequest::new(
            geometry_source,
            pass_type,
            surface_source,
            SURFACE_SHADER_ENTRY_POINT,
        )
        .with_features(key.shader_feature_bits())
        .with_material_surface_module_id(runtime_surface_module_id(streamer, key));
        if let Some(generated_material_source) =
            streamer.shader_generated_material_source(&key.shader_id)
        {
            request = request.with_generated_material_source(generated_material_source);
        }
        let request = with_runtime_material_modules_and_options(request, streamer, key);
        let request = with_runtime_shading_model_sources(request, streamer, key)?;
        return assemble_material_shader_template(request)
            .map(MeshPipelineShaderSource::from_template);
    }

    let material_surface = standard_material_surface_source_for_features(
        key.shader_feature_bits(),
        mesh_pipeline_alpha_cutoff(key),
    );
    let request = MaterialShaderTemplateRequest::new(
        geometry_source,
        pass_type,
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);
    let request = with_runtime_shading_model_sources(request, streamer, key)?;

    assemble_material_shader_template(request).map(MeshPipelineShaderSource::from_template)
}

fn shader_source_uses_runtime_material_surface(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
) -> bool {
    !key.uses_fallback_shader() && streamer.shader_uses_material_surface_source(&key.shader_id)
}

fn runtime_surface_module_id(streamer: &ResourceStreamer, key: &PipelineKey) -> String {
    streamer
        .shader_import_path(&key.shader_id)
        .unwrap_or(DEFAULT_SURFACE_SHADER_MODULE_ID)
        .to_string()
}

fn with_runtime_material_modules_and_options(
    request: MaterialShaderTemplateRequest,
    streamer: &ResourceStreamer,
    key: &PipelineKey,
) -> MaterialShaderTemplateRequest {
    request
        .with_module_include_sources(streamer.shader_module_include_sources(&key.shader_id))
        .with_material_option_defines(
            streamer.shader_material_option_defines(&key.shader_id, key.material_option_bits),
        )
}

fn with_runtime_gbuffer_material_modules_and_options(
    request: DeferredGBufferShaderTemplateRequest,
    streamer: &ResourceStreamer,
    key: &PipelineKey,
) -> DeferredGBufferShaderTemplateRequest {
    request
        .with_module_include_sources(streamer.shader_module_include_sources(&key.shader_id))
        .with_material_option_defines(
            streamer.shader_material_option_defines(&key.shader_id, key.material_option_bits),
        )
}

fn with_runtime_shading_model_sources(
    request: MaterialShaderTemplateRequest,
    streamer: &ResourceStreamer,
    key: &PipelineKey,
) -> Result<MaterialShaderTemplateRequest, ShaderTemplateAssemblyError> {
    let Some(descriptor) = streamer
        .shading_model_descriptor_for_pipeline_key(key)
        .filter(|descriptor| descriptor.id.is_plugin_range())
        .cloned()
    else {
        return Ok(request);
    };
    let source_set = streamer
        .shading_model_include_source_set()
        .map_err(|error| ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: error.to_string(),
        })?;
    Ok(request
        .with_shading_model_descriptor(descriptor)
        .with_shading_model_forward_include_sources(&source_set))
}

fn with_runtime_gbuffer_shading_model_sources(
    request: DeferredGBufferShaderTemplateRequest,
    streamer: &ResourceStreamer,
    key: &PipelineKey,
) -> Result<DeferredGBufferShaderTemplateRequest, ShaderTemplateAssemblyError> {
    let Some(descriptor) = streamer
        .shading_model_descriptor_for_pipeline_key(key)
        .filter(|descriptor| descriptor.id.is_plugin_range())
        .cloned()
    else {
        return Ok(request);
    };
    let source_set = streamer
        .shading_model_include_source_set()
        .map_err(|error| ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: error.to_string(),
        })?;
    Ok(request
        .with_shading_model_descriptor(descriptor)
        .with_shading_model_gbuffer_include_sources(&source_set))
}

fn mesh_pipeline_builtin_geometry_source_descriptor(
    geometry_source: GeometrySourceId,
) -> Result<GeometrySourceDescriptor, ShaderTemplateAssemblyError> {
    match builtin_geometry_source_descriptor(geometry_source) {
        Some(descriptor) => Ok(descriptor),
        None => Err(ShaderTemplateAssemblyError::UnknownGeometryInclude {
            token: format!("geometry_source_{}", geometry_source.value()),
        }),
    }
}

fn mesh_pipeline_alpha_cutoff(key: &PipelineKey) -> f32 {
    if key.is_alpha_mask() {
        key.alpha_cutoff_bits.map(f32::from_bits).unwrap_or(0.0)
    } else {
        0.0
    }
}

fn mesh_pipeline_wgsl_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
#[path = "shader_source/tests.rs"]
mod tests;
