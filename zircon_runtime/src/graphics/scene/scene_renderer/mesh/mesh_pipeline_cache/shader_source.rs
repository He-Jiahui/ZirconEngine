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

pub(crate) fn mesh_pipeline_shader_source(
    streamer: &ResourceStreamer,
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    if key.uses_fallback_shader() {
        mesh_pipeline_standard_material_template_source_for_geometry(key, geometry_source)
    } else {
        streamer
            .shader_source(&key.shader_id)
            .map(MeshPipelineShaderSource::from_raw_wgsl)
            .map_or_else(
                || {
                    mesh_pipeline_standard_material_template_source_for_geometry(
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
    mesh_pipeline_material_template_source_for_geometry_and_pass(
        key,
        geometry_source,
        ShaderPassType::Forward,
    )
}

pub(crate) fn mesh_pipeline_standard_material_template_source_for_shader_pass(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_and_pass(key, geometry_source, pass_type)
}

pub(crate) fn mesh_pipeline_velocity_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_and_pass(
        key,
        geometry_source,
        ShaderPassType::Velocity,
    )
}

pub(crate) fn mesh_pipeline_depth_prepass_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_and_pass(
        key,
        geometry_source,
        ShaderPassType::GBuffer,
    )
}

pub(crate) fn mesh_pipeline_deferred_gbuffer_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    let material_surface = standard_material_surface_source_for_features(
        key.shader_feature_bits(),
        mesh_pipeline_alpha_cutoff(key),
    );
    let request = DeferredGBufferShaderTemplateRequest::new(
        geometry_source,
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);

    assemble_deferred_gbuffer_shader_template(request).map(MeshPipelineShaderSource::from_template)
}

pub(crate) fn mesh_pipeline_shadow_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    mesh_pipeline_material_template_source_for_geometry_and_pass(
        key,
        geometry_source,
        ShaderPassType::Shadow,
    )
}

pub(crate) fn mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
    let material_surface = standard_material_surface_source_for_features(
        key.shader_feature_bits(),
        mesh_pipeline_alpha_cutoff(key),
    );
    let request = TaaReactiveMaskShaderTemplateRequest::new(
        geometry_source,
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);

    assemble_taa_reactive_mask_shader_template(request).map(MeshPipelineShaderSource::from_template)
}

fn mesh_pipeline_material_template_source_for_geometry_and_pass(
    key: &PipelineKey,
    geometry_source: GeometrySourceId,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, ShaderTemplateAssemblyError> {
    let geometry_source = mesh_pipeline_builtin_geometry_source_descriptor(geometry_source)?;
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
mod tests {
    use crate::core::framework::render::{
        ShaderFeatureBits, ShaderPassType, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    };
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::{
        mesh_pipeline_deferred_gbuffer_template_source_for_geometry,
        mesh_pipeline_depth_prepass_template_source_for_geometry,
        mesh_pipeline_shadow_template_source_for_geometry,
        mesh_pipeline_standard_material_template_source,
        mesh_pipeline_standard_material_template_source_for_geometry,
        mesh_pipeline_standard_material_template_source_for_shader_pass,
        mesh_pipeline_taa_reactive_mask_template_source_for_geometry,
        mesh_pipeline_velocity_template_source_for_geometry, MESH_SHADER_TEMPLATE_REVISION,
    };

    #[test]
    fn mesh_pipeline_standard_material_template_source_assembles_forward_base_source() {
        let mut key = default_pipeline_key();
        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(0.5f32.to_bits());
        key.double_sided = true;
        key.receive_shadows = false;

        let source = match mesh_pipeline_standard_material_template_source(&key) {
            Ok(source) => source,
            Err(error) => panic!("standard material template assembly failed: {error:?}"),
        };

        assert!(source
            .wgsl_source
            .contains("// include: zr_scene_runtime.wgsl"));
        assert!(source.wgsl_source.contains("// include: zr_gpu_scene.wgsl"));
        assert!(source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl"));
        assert!(source.wgsl_source.contains("// include: zr_shadow.wgsl"));
        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(source.wgsl_source.contains("fn fs_main("));
        assert!(source.wgsl_source.contains("fn zr_material_surface("));
        assert!(source
            .wgsl_source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
        assert!(source
            .wgsl_source
            .contains("const ZR_FEATURE_ALPHA_TEST: bool = true;"));
        assert!(source
            .wgsl_source
            .contains("const ZR_FEATURE_RECEIVE_SHADOWS: bool = false;"));
        assert!(source
            .wgsl_source
            .contains("const ZR_FEATURE_DOUBLE_SIDED: bool = true;"));
        assert!(source
            .wgsl_source
            .contains("standard_material_shading_model_id"));
        assert!(source
            .wgsl_source
            .contains("surface.shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID"));
        assert!(key
            .shader_feature_bits()
            .contains(ShaderFeatureBits::ALPHA_TEST));
    }

    #[test]
    fn mesh_pipeline_depth_prepass_template_source_writes_normal_target() {
        let mut key = default_pipeline_key();
        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(0.5f32.to_bits());

        let source = match mesh_pipeline_depth_prepass_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ) {
            Ok(source) => source,
            Err(error) => panic!("depth prepass template source assembly failed: {error:?}"),
        };

        assert!(source.wgsl_source.contains("zr_template_gbuffer.wgsl"));
        assert!(source.wgsl_source.contains("zr_geometry_skinned.wgsl"));
        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(source.wgsl_source.contains("fn fs_main("));
        assert!(source.wgsl_source.contains("fn zr_material_surface("));
        assert!(source.wgsl_source.contains("surface.normal_ws * 0.5"));
        assert!(source.wgsl_source.contains("zr_surface_fails_alpha_clip"));
        assert!(!source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl"));
        assert!(!source.wgsl_source.contains("// include: zr_shadow.wgsl"));
    }

    #[test]
    fn mesh_pipeline_standard_material_shader_pass_source_keeps_depth_only_contract() {
        let opaque_key = default_pipeline_key();
        let opaque_source = match mesh_pipeline_standard_material_template_source_for_shader_pass(
            &opaque_key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
            ShaderPassType::DepthPrepass,
        ) {
            Ok(source) => source,
            Err(error) => panic!("opaque depth-only template source failed: {error:?}"),
        };

        assert!(opaque_source
            .wgsl_source
            .contains("// include: zr_template_depth.wgsl"));
        assert!(opaque_source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl"));
        assert!(opaque_source.wgsl_source.contains("fn vs_main("));
        assert!(!opaque_source.wgsl_source.contains("fn fs_main("));
        assert!(!opaque_source.wgsl_source.contains("zr_material_surface"));
        assert!(!opaque_source
            .wgsl_source
            .contains("surface.normal_ws * 0.5"));
        assert!(!opaque_source
            .wgsl_source
            .contains("// include: zr_template_gbuffer.wgsl"));
        assert!(!opaque_source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl"));
        assert!(!opaque_source
            .wgsl_source
            .contains("// include: zr_shadow.wgsl"));
        assert!(opaque_source
            .cache_content_hashes
            .contains(&opaque_source.source_hash));

        let mut alpha_key = default_pipeline_key();
        alpha_key.alpha_mask = true;
        alpha_key.alpha_cutoff_bits = Some(0.5f32.to_bits());
        let alpha_source = match mesh_pipeline_standard_material_template_source_for_shader_pass(
            &alpha_key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
            ShaderPassType::DepthPrepass,
        ) {
            Ok(source) => source,
            Err(error) => panic!("alpha depth-only template source failed: {error:?}"),
        };

        assert!(alpha_source
            .wgsl_source
            .contains("// include: zr_template_depth_alpha.wgsl"));
        assert!(alpha_source.wgsl_source.contains("fn fs_main("));
        assert!(alpha_source.wgsl_source.contains("zr_material_surface"));
        assert!(alpha_source
            .wgsl_source
            .contains("zr_apply_alpha_clip(surface);"));
        assert!(!alpha_source.wgsl_source.contains("surface.normal_ws * 0.5"));
        assert!(!alpha_source
            .wgsl_source
            .contains("// include: zr_template_gbuffer.wgsl"));
        assert_ne!(opaque_source.wgsl_source, alpha_source.wgsl_source);
        assert_ne!(
            opaque_source.cache_content_hashes,
            alpha_source.cache_content_hashes
        );
        assert_eq!(alpha_source.template_revision, "zr-material-template-v1");
    }

    #[test]
    fn mesh_pipeline_deferred_gbuffer_template_source_writes_albedo_and_material_targets() {
        let mut key = default_pipeline_key();
        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(0.5f32.to_bits());

        let source = match mesh_pipeline_deferred_gbuffer_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ) {
            Ok(source) => source,
            Err(error) => panic!("deferred gbuffer template source assembly failed: {error:?}"),
        };

        assert!(source
            .wgsl_source
            .contains("// include: zr_template_deferred_gbuffer.wgsl"));
        assert!(source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl"));
        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(source.wgsl_source.contains("fn fs_main("));
        assert!(source.wgsl_source.contains("ZrDeferredGBufferOutput"));
        assert!(source.wgsl_source.contains("@location(0) albedo"));
        assert!(source.wgsl_source.contains("@location(1) material"));
        assert!(source
            .wgsl_source
            .contains("zr_deferred_encode_material_flags(surface.shading_model_id"));
        assert!(source.wgsl_source.contains("zr_surface_fails_alpha_clip"));
        assert!(!source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl"));
        assert!(!source.wgsl_source.contains("// include: zr_shadow.wgsl"));
        assert_eq!(source.template_revision, "zr-material-template-v1");
    }

    #[test]
    fn mesh_pipeline_template_source_hashes_include_template_revision() {
        let key = default_pipeline_key();

        let source = match mesh_pipeline_standard_material_template_source(&key) {
            Ok(source) => source,
            Err(error) => panic!("standard material template assembly failed: {error:?}"),
        };

        assert!(source.cache_content_hashes.len() > 1);
        assert!(source.cache_content_hashes.contains(&source.source_hash));
        assert_ne!(source.template_revision, MESH_SHADER_TEMPLATE_REVISION);
    }

    #[test]
    fn mesh_pipeline_standard_material_template_source_uses_requested_geometry_source() {
        let key = default_pipeline_key();

        let source = match mesh_pipeline_standard_material_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ) {
            Ok(source) => source,
            Err(error) => panic!("skinned standard material template assembly failed: {error:?}"),
        };

        assert!(source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl"));
        assert!(source
            .wgsl_source
            .contains("const ZR_GEOMETRY_SOURCE_SKINNED_MESH: bool = true;"));
        assert!(source
            .wgsl_source
            .contains("zr_skinned_joint_matrix(v.joints.x)"));
        assert_eq!(source.template_revision, "zr-material-template-v1");
    }

    #[test]
    fn mesh_pipeline_velocity_template_source_uses_previous_position_vertex_input() {
        let mut key = default_pipeline_key();
        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(0.5f32.to_bits());

        let source = match mesh_pipeline_velocity_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ) {
            Ok(source) => source,
            Err(error) => panic!("skinned velocity template assembly failed: {error:?}"),
        };

        assert!(source
            .wgsl_source
            .contains("// include: zr_template_velocity_alpha.wgsl"));
        assert!(source.wgsl_source.contains("struct ZrVelocityVertexInput"));
        assert!(source
            .wgsl_source
            .contains("@location(8) previous_position"));
        assert!(source
            .wgsl_source
            .contains("let previous_input = zr_velocity_vertex_input(v, v.previous_position);"));
        assert!(source
            .wgsl_source
            .contains("fetch_prev_position(previous_input, instance_index)"));
        assert!(source
            .wgsl_source
            .contains("scene.previous_view_proj_unjittered * previous_world"));
        assert!(source.wgsl_source.contains("fn fs_main("));
        assert!(source
            .wgsl_source
            .contains("zr_velocity_apply_alpha_clip(input);"));
        assert!(source.wgsl_source.contains("zr_material_surface"));
        assert_eq!(source.template_revision, "zr-material-template-v1");
    }

    #[test]
    fn mesh_pipeline_taa_reactive_mask_template_source_uses_material_surface_without_lighting() {
        let mut key = default_pipeline_key();
        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(0.5f32.to_bits());

        let source = match mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ) {
            Ok(source) => source,
            Err(error) => panic!("skinned TAA reactive mask template assembly failed: {error:?}"),
        };

        assert!(source
            .wgsl_source
            .contains("// include: zr_template_taa_reactive_mask.wgsl"));
        assert!(source
            .wgsl_source
            .contains("// include: zr_geometry_skinned.wgsl"));
        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(source.wgsl_source.contains("fn fs_taa_reactive_mask("));
        assert!(source
            .wgsl_source
            .contains("fn fs_taa_reactive_material_mask("));
        assert!(source.wgsl_source.contains("fn zr_material_surface("));
        assert!(source
            .wgsl_source
            .contains("standard_material_properties.data8.x"));
        assert!(source.wgsl_source.contains("surface.custom0.x"));
        assert!(source
            .wgsl_source
            .contains("const ZR_FEATURE_ALPHA_TEST: bool = true;"));
        assert!(!source
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl"));
        assert!(!source.wgsl_source.contains("// include: zr_shadow.wgsl"));
        assert_eq!(source.template_revision, "zr-material-template-v1");
    }

    #[test]
    fn mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked() {
        let mut key = default_pipeline_key();
        let opaque = match mesh_pipeline_shadow_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ) {
            Ok(source) => source,
            Err(error) => panic!("skinned shadow template assembly failed: {error:?}"),
        };

        assert!(opaque
            .wgsl_source
            .contains("// include: zr_template_shadow.wgsl"));
        assert!(opaque.wgsl_source.contains("fn vs_main("));
        assert!(!opaque.wgsl_source.contains("fn fs_main("));
        assert!(!opaque.wgsl_source.contains("fn zr_material_surface("));
        assert!(!opaque
            .wgsl_source
            .contains("// include: zr_light_grid.wgsl"));
        assert!(!opaque.wgsl_source.contains("// include: zr_shadow.wgsl"));

        key.alpha_mask = true;
        key.alpha_cutoff_bits = Some(0.5f32.to_bits());
        let alpha = match mesh_pipeline_shadow_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ) {
            Ok(source) => source,
            Err(error) => panic!("skinned alpha shadow template assembly failed: {error:?}"),
        };

        assert!(alpha
            .wgsl_source
            .contains("// include: zr_template_shadow_alpha.wgsl"));
        assert!(alpha.wgsl_source.contains("fn vs_main("));
        assert!(alpha.wgsl_source.contains("fn fs_main("));
        assert!(alpha.wgsl_source.contains("fn zr_material_surface("));
        assert!(alpha.wgsl_source.contains("zr_surface_fails_alpha_clip"));
        assert!(alpha
            .wgsl_source
            .contains("const ZR_FEATURE_ALPHA_TEST: bool = true;"));
        assert!(!alpha.wgsl_source.contains("// include: zr_light_grid.wgsl"));
        assert!(!alpha.wgsl_source.contains("// include: zr_shadow.wgsl"));
        assert_eq!(alpha.template_revision, "zr-material-template-v1");
    }
}
