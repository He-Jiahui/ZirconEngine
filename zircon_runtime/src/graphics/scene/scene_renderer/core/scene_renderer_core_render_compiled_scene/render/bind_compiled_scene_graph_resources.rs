use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderGraphImportedFinalTarget, RenderPassMeshCommandLists,
    TransientResourcePool,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::CompiledRenderPipeline;

use super::bind_environment_ibl_graph_resources::bind_environment_ibl_graph_resources;
use super::bind_execution_owned_graph_resources::bind_execution_owned_graph_resources;
use super::bind_frame_graph_resources::bind_frame_graph_resources;
use super::bind_history_graph_resources::{
    bind_history_graph_resources, HistoryGraphResourceBindingFlags,
};
use super::bind_plugin_graph_resources::bind_plugin_graph_resources;
use super::final_target_output::{select_final_target_output, FinalTargetOutputSelection};

pub(super) struct CompiledSceneGraphResourceBindingFlags {
    pub(super) taa_history_enabled: bool,
    pub(super) screen_space_reflection_history_enabled: bool,
    pub(super) hzb_history_enabled: bool,
    pub(super) exposure_history_enabled: bool,
    pub(super) history_available: bool,
    pub(super) runtime_features: SceneRuntimeFeatureFlags,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_compiled_scene_graph_resources(
    device: &wgpu::Device,
    pipeline: &CompiledRenderPipeline,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    target: &mut OffscreenTarget,
    history_textures: Option<&SceneFrameHistoryTextures>,
    flags: CompiledSceneGraphResourceBindingFlags,
    graph_resources: &mut RenderGraphExecutionResources,
    transient_resource_pool: &mut TransientResourcePool,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    shadow_atlas_resources: &ShadowAtlasResources,
    plugin_external_buffer_bindings: &[crate::graphics::RuntimePrepareExternalBufferBinding],
    environment_source_cubemap_view: Option<&wgpu::TextureView>,
) -> Result<FinalTargetOutputSelection, GraphicsError> {
    let final_target_output = select_final_target_output(streamer, frame);
    let imported_final_target =
        final_target_output
            .imported_resource()
            .map(|resource| RenderGraphImportedFinalTarget {
                view: resource.view(),
            });
    bind_frame_graph_resources(
        &pipeline.graph,
        graph_resources,
        target,
        imported_final_target,
        Some(shadow_atlas_resources),
    );
    bind_history_graph_resources(
        &pipeline.graph,
        graph_resources,
        history_textures,
        HistoryGraphResourceBindingFlags {
            taa_scene_color: flags.taa_history_enabled,
            screen_space_reflection: flags.history_available
                && flags.screen_space_reflection_history_enabled,
            hzb: flags.history_available && flags.hzb_history_enabled,
            hybrid_global_illumination: flags.history_available
                && flags.runtime_features.hybrid_global_illumination_enabled,
            exposure: flags.exposure_history_enabled,
        },
    );
    bind_environment_ibl_graph_resources(
        &pipeline.graph,
        environment_source_cubemap_view,
        graph_resources,
    );
    graph_resources
        .materialize_transient_resources_with_pool(device, &pipeline.graph, transient_resource_pool)
        .map_err(GraphicsError::Asset)?;
    bind_execution_owned_graph_resources(
        device,
        &pipeline.graph,
        graph_resources,
        mesh_draw_lists,
        hzb_occlusion_culler,
    );
    bind_plugin_graph_resources(
        device,
        &pipeline.graph,
        plugin_external_buffer_bindings,
        graph_resources,
    );
    Ok(final_target_output)
}
