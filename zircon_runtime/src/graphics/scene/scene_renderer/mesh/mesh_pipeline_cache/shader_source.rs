use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_STATIC_MESH, GeometrySourceDescriptor, GeometrySourceId, ShaderFeatureBits,
    ShaderPassType, builtin_geometry_source_descriptor,
};
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};
use crate::graphics::shader::{
    DeferredGBufferShaderTemplateRequest, MaterialShaderTemplateAssembly,
    MaterialShaderTemplateRequest, ShaderAssemblySegment, ShaderAssemblySegmentKind,
    ShaderTemplateAssemblyError, ShaderTemplateValidationError,
    TaaReactiveMaskShaderTemplateRequest, assemble_deferred_gbuffer_shader_template,
    assemble_material_shader_template, assemble_taa_reactive_mask_shader_template,
    standard_material_surface_source_for_features,
    validate_material_shader_template_wgsl_with_segments,
};

const MESH_SHADER_TEMPLATE_REVISION: &str = "mesh-template-v1";
const OIT_SHADER_TEMPLATE_REVISION: &str = "oit-fragment-store-v1";
const OIT_DRAW_SHADER_SOURCE: &str = include_str!("../../../../shader/includes/zr_oit.wgsl");
const OIT_FRAGMENT_ENTRY_SOURCE: &str = r#"
@fragment
fn fs_oit(input: ZrVertexOutput) {
    oit_draw(input.clip_position, zr_fs_main_impl(input));
}
"#;
const OIT_DRAW_SHADER_MODULE_ID: &str = "zircon::oit::draw";
const OIT_FRAGMENT_ENTRY_MODULE_ID: &str = "zircon::oit::fragment_store_entry";
const SURFACE_SHADER_ENTRY_POINT: &str = "zr_material_surface";
const DEFAULT_SURFACE_SHADER_MODULE_ID: &str = "self::surface";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MeshPipelineShaderSource {
    pub(crate) wgsl_source: String,
    pub(crate) source_hash: String,
    pub(crate) cache_content_hashes: Vec<String>,
    pub(crate) template_revision: String,
    pub(crate) segments: Vec<ShaderAssemblySegment>,
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
            segments: assembly.segments,
        }
    }

    fn from_raw_wgsl(source: &str) -> Self {
        let source_hash = mesh_pipeline_wgsl_hash(source);
        Self {
            wgsl_source: source.to_string(),
            source_hash: source_hash.clone(),
            cache_content_hashes: vec![source_hash],
            template_revision: MESH_SHADER_TEMPLATE_REVISION.to_string(),
            segments: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(super) fn validate_wgsl(&self, wgsl_source: &str) -> Result<(), String> {
        Self::validate_wgsl_with_segments(wgsl_source, &self.segments)
    }

    pub(super) fn validate_wgsl_with_segments(
        wgsl_source: &str,
        segments: &[ShaderAssemblySegment],
    ) -> Result<(), String> {
        validate_material_shader_template_wgsl_with_segments(wgsl_source, segments)
            .map(|_| ())
            .map_err(|error| match error {
                ShaderTemplateValidationError::Parse { message }
                | ShaderTemplateValidationError::Validate { message } => message,
            })
    }

    pub(super) fn validation_cache_key(&self) -> String {
        use std::fmt::Write;

        let mut key = self.source_hash.clone();
        for segment in &self.segments {
            let _ = write!(
                key,
                "|{}:{}:{}:{}",
                segment.module_id,
                segment.assembled_start_line,
                segment.assembled_line_count,
                segment.source_line_offset
            );
        }
        key
    }

    pub(crate) fn into_oit_fragment_store_source(mut self) -> Option<Self> {
        if self.wgsl_source.contains("fn fs_oit(") {
            return Some(self);
        }
        if !self.wgsl_source.contains("fn zr_fs_main_impl(") {
            return None;
        }

        let oit_draw_start_line = shader_source_append_start_line(&self.wgsl_source);
        self.wgsl_source.push('\n');
        self.wgsl_source.push_str(OIT_DRAW_SHADER_SOURCE);
        self.segments.push(ShaderAssemblySegment {
            module_id: OIT_DRAW_SHADER_MODULE_ID.to_string(),
            kind: ShaderAssemblySegmentKind::Include,
            assembled_start_line: oit_draw_start_line,
            assembled_line_count: shader_source_line_count(OIT_DRAW_SHADER_SOURCE),
            source_line_offset: 0,
        });
        let oit_entry_start_line = shader_source_append_start_line(&self.wgsl_source);
        self.wgsl_source.push('\n');
        self.wgsl_source.push_str(OIT_FRAGMENT_ENTRY_SOURCE);
        self.segments.push(ShaderAssemblySegment {
            module_id: OIT_FRAGMENT_ENTRY_MODULE_ID.to_string(),
            kind: ShaderAssemblySegmentKind::PassTemplate,
            assembled_start_line: oit_entry_start_line,
            assembled_line_count: shader_source_line_count(OIT_FRAGMENT_ENTRY_SOURCE),
            source_line_offset: 0,
        });
        self.source_hash = mesh_pipeline_wgsl_hash(&self.wgsl_source);
        self.cache_content_hashes
            .push(mesh_pipeline_wgsl_hash(OIT_DRAW_SHADER_SOURCE));
        self.cache_content_hashes.push(self.source_hash.clone());
        self.template_revision = format!(
            "{}+{}",
            self.template_revision, OIT_SHADER_TEMPLATE_REVISION
        );
        Some(self)
    }
}

fn shader_source_line_count(source: &str) -> u32 {
    (!source.is_empty())
        .then(|| source.lines().count() as u32)
        .unwrap_or(0)
}

fn shader_source_append_start_line(source: &str) -> u32 {
    shader_source_line_count(source)
        .saturating_add(1)
        .saturating_add(u32::from(source.ends_with('\n')))
}

pub(crate) fn mesh_pipeline_shader_source_for_geometry_descriptor(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_shader_source_for_geometry_descriptor_with_features(
        streamer,
        key,
        geometry_source,
        key.shader_feature_bits(),
    )
}

pub(crate) fn mesh_pipeline_shader_source_for_geometry_descriptor_with_features(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
    features: ShaderFeatureBits,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    if key.uses_fallback_shader() {
        mesh_pipeline_standard_material_template_source_for_geometry_descriptor_with_streamer_and_features(
            streamer,
            key,
            geometry_source,
            features,
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
    mesh_pipeline_standard_material_template_source_for_geometry_descriptor_with_streamer_and_features(
        streamer,
        key,
        geometry_source,
        key.shader_feature_bits(),
    )
}

fn mesh_pipeline_standard_material_template_source_for_geometry_descriptor_with_streamer_and_features(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
    features: ShaderFeatureBits,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let material_surface =
        standard_material_surface_source_for_features(features, mesh_pipeline_alpha_cutoff(key));
    let request = MaterialShaderTemplateRequest::new(
        geometry_source.clone(),
        ShaderPassType::Forward,
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);
    let request = with_runtime_shading_model_sources(request, streamer, key)?;

    assemble_material_shader_template(request).map(MeshPipelineShaderSource::from_template)
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
