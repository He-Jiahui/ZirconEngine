use crate::asset::AssetReference;
use crate::core::framework::render::{
    GeometrySourceDescriptor, ShaderPassType, ShaderVariantPrewarmRequest,
    ShaderVariantPrewarmSource,
};
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};

use super::super::shader_source::{
    mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer,
    mesh_pipeline_depth_prepass_template_source_for_geometry_descriptor_with_streamer,
    mesh_pipeline_hit_proxy_template_source_for_geometry_descriptor_with_streamer,
    mesh_pipeline_shader_source_for_geometry_descriptor,
    mesh_pipeline_shadow_template_source_for_geometry_descriptor_with_streamer,
    mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor_with_streamer,
    mesh_pipeline_velocity_template_source_for_geometry_descriptor_with_streamer,
};
use super::pipeline_key_from_prewarm_request;

pub(super) fn bind_runtime_prewarm_request(
    streamer: &mut ResourceStreamer,
    request: &ShaderVariantPrewarmRequest,
    source: &ShaderVariantPrewarmSource,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<PipelineKey, String> {
    if !source.has_canonical_id() {
        return Err("prewarm source does not match its content-addressed identity".to_string());
    }
    let mut key = pipeline_key_from_prewarm_request(request).map_err(str::to_string)?;
    let assets = streamer
        .asset_manager()
        .map_err(|error| error.to_string())?;
    let resources = assets.resource_manager();
    let reference = resources
        .registry()
        .get(key.shader_id)
        .map(|record| AssetReference::from_locator(record.primary_locator.clone()))
        .ok_or_else(|| format!("prewarm shader {} is not registered", key.shader_id))?;
    let prepared = streamer
        .ensure_shader_source(&reference)
        .map_err(|error| error.to_string())?;
    // Cold loading can change readiness state. Reprepare against that publication,
    // preserving the snapshot captured by source preparation instead of relabeling it.
    let (id, revision, publication, fallback) = if resources
        .readiness_generation()
        .row_identity(prepared.0)
        .as_ref()
        != Some(&prepared.2)
    {
        streamer
            .ensure_shader_source(&reference)
            .map_err(|error| error.to_string())?
    } else {
        prepared
    };
    if id != key.shader_id
        || revision != key.shader_revision
        || fallback.is_some()
        || publication.row().record.revision != revision
    {
        return Err(format!(
            "prewarm shader {} does not match its requested resource revision",
            key.shader_id
        ));
    }
    key.shader_dependency_identity = Some(publication.clone());

    let live_source = match request.key.pass_type {
        ShaderPassType::Forward => {
            mesh_pipeline_shader_source_for_geometry_descriptor(streamer, &key, geometry_source)
        }
        ShaderPassType::GBuffer => {
            mesh_pipeline_deferred_gbuffer_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                &key,
                geometry_source,
            )
        }
        ShaderPassType::DepthPrepass => {
            mesh_pipeline_depth_prepass_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                &key,
                geometry_source,
            )
        }
        ShaderPassType::HitProxy => {
            mesh_pipeline_hit_proxy_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                &key,
                geometry_source,
            )
        }
        ShaderPassType::Shadow => {
            mesh_pipeline_shadow_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                &key,
                geometry_source,
            )
        }
        ShaderPassType::Velocity => {
            mesh_pipeline_velocity_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                &key,
                geometry_source,
            )
        }
        ShaderPassType::TaaReactiveMask => {
            mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                &key,
                geometry_source,
            )
        }
    }
    .map_err(|error| format!("prewarm runtime shader assembly failed: {error:?}"))?;
    if live_source.wgsl_source != source.wgsl_source
        || live_source.template_revision != source.template_revision
    {
        return Err(format!(
            "prewarm shader {} source does not match the runtime publication",
            key.shader_id
        ));
    }
    if resources.readiness_generation().row_identity(id).as_ref() != Some(&publication) {
        return Err(format!(
            "prewarm shader {id} publication changed during source assembly"
        ));
    }
    Ok(key)
}
