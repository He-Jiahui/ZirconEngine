use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderPassMeshCommandLists,
};
use crate::graphics::scene::scene_renderer::post_process::ScenePostProcessResources;
use crate::render_graph::CompiledRenderGraph;

pub(super) fn bind_taa_reactive_mask_graph_resource(
    graph: &CompiledRenderGraph,
    post_process: &ScenePostProcessResources,
    mesh_draw_lists: RenderPassMeshCommandLists<'_>,
    resources: &mut RenderGraphExecutionResources,
) {
    if !mesh_draw_lists.taa_reactive_mask_stream().is_empty()
        || graph
            .resource_lifetime_by_name(PostProcessGraphResourceNames::TAA_REACTIVE_MASK)
            .is_none()
    {
        return;
    }

    resources.import_borrowed_texture_view_with_identity(
        PostProcessGraphResourceNames::TAA_REACTIVE_MASK,
        post_process.black_texture_view(),
        post_process.black_texture_identity(),
    );
}
