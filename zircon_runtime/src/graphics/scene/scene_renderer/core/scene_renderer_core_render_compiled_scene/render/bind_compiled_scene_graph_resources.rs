use crate::graphics::CompiledRenderPipeline;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::SceneRendererNeutralGraphBuffers;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderGraphImportedFinalTarget, RenderPassMeshCommandLists,
    TransientResourcePool,
};
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryAvailability, SceneHistoryDomain,
};
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::post_process::{
    ScenePostProcessResources, SceneRuntimeFeatureFlags,
};
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::rhi::RenderDeviceProfile;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::bind_environment_ibl_graph_resources::bind_environment_ibl_graph_resources;
use super::bind_execution_owned_graph_resources::bind_execution_owned_graph_resources;
use super::bind_frame_graph_resources::bind_frame_graph_resources;
use super::bind_history_graph_resources::{
    HistoryGraphResourceBindingFlags, bind_history_graph_resources,
};
use super::bind_plugin_graph_resources::bind_plugin_graph_resources;
use super::bind_ssao_compute_graph_resources::bind_ssao_compute_graph_resources;
use super::bind_taa_reactive_mask_graph_resource::bind_taa_reactive_mask_graph_resource;
use super::final_target_output::{FinalTargetOutputSelection, select_final_target_output};

pub(super) struct CompiledSceneGraphResourceBindingFlags {
    pub(super) taa_history_enabled: bool,
    pub(super) screen_space_reflection_history_enabled: bool,
    pub(super) hzb_history_enabled: bool,
    pub(super) exposure_history_enabled: bool,
    pub(super) volumetric_history_enabled: bool,
    pub(super) history_availability: SceneHistoryAvailability,
    pub(super) runtime_features: SceneRuntimeFeatureFlags,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_compiled_scene_graph_resources(
    device: &wgpu::Device,
    device_profile: &RenderDeviceProfile,
    pipeline: &CompiledRenderPipeline,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    target: &mut OffscreenTarget,
    post_process: &ScenePostProcessResources,
    scene_light_data_buffer: &wgpu::Buffer,
    history_textures: Option<&SceneFrameHistoryTextures>,
    flags: CompiledSceneGraphResourceBindingFlags,
    graph_resources: &mut RenderGraphExecutionResources,
    transient_resource_pool: &mut TransientResourcePool,
    neutral_graph_buffers: &mut SceneRendererNeutralGraphBuffers,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    shadow_atlas_resources: &ShadowAtlasResources,
    plugin_external_buffer_binding_packet: Option<
        &crate::graphics::RuntimePrepareExternalBufferBindingPacket,
    >,
    environment_source_cubemap_view: Option<&wgpu::TextureView>,
    frame_buffer_uploads: &mut WgpuBufferUploadBatch,
) -> Result<FinalTargetOutputSelection, GraphicsError> {
    let final_target_output = select_final_target_output(streamer, frame)?;
    let imported_final_target = final_target_output
        .imported_resource()
        .map(|resource| {
            Ok(RenderGraphImportedFinalTarget {
                texture: resource.texture(),
                view: resource.view(),
                desc: resource.graph_texture_desc("imported-final-target")?,
            })
        })
        .transpose()?;
    bind_frame_graph_resources(
        device,
        pipeline.graph(),
        graph_resources,
        target,
        scene_light_data_buffer,
        imported_final_target,
        final_target_output.output_target_resource(),
        Some(shadow_atlas_resources),
    )?;
    bind_history_graph_resources(
        pipeline.graph(),
        graph_resources,
        history_textures,
        HistoryGraphResourceBindingFlags {
            taa_scene_color: flags.taa_history_enabled,
            screen_space_reflection: flags
                .history_availability
                .is_available(SceneHistoryDomain::ScreenSpaceReflection)
                && flags.screen_space_reflection_history_enabled,
            hzb: flags
                .history_availability
                .is_available(SceneHistoryDomain::HzbFurthest)
                && flags.hzb_history_enabled,
            hybrid_global_illumination: flags.runtime_features.hybrid_global_illumination_enabled,
            exposure: flags.exposure_history_enabled,
            volumetric_scattering: flags
                .history_availability
                .is_available(SceneHistoryDomain::VolumetricScattering)
                && flags.volumetric_history_enabled,
        },
    );
    bind_ssao_compute_graph_resources(
        pipeline,
        target,
        post_process,
        graph_resources,
        frame_buffer_uploads,
    )?;
    bind_environment_ibl_graph_resources(
        pipeline.graph(),
        environment_source_cubemap_view,
        graph_resources,
    );
    bind_taa_reactive_mask_graph_resource(
        pipeline.graph(),
        post_process,
        mesh_draw_lists,
        graph_resources,
    );
    graph_resources
        .materialize_transient_resources_with_pool(
            device,
            device_profile,
            pipeline.graph(),
            transient_resource_pool,
        )
        .map_err(GraphicsError::Asset)?;
    bind_execution_owned_graph_resources(
        device,
        neutral_graph_buffers,
        pipeline.graph(),
        graph_resources,
        mesh_draw_lists,
        hzb_occlusion_culler,
    );
    bind_plugin_graph_resources(
        device,
        neutral_graph_buffers,
        pipeline.graph(),
        plugin_external_buffer_binding_packet,
        graph_resources,
    )
    .map_err(GraphicsError::Asset)?;
    graph_resources
        .materialize_external_access_bindings(pipeline.graph())
        .map_err(GraphicsError::Asset)?;
    Ok(final_target_output)
}
