use crate::graphics::scene::scene_renderer::environment::ibl_bake_graph_plan::IBL_BAKE_SOURCE_CUBEMAP_RESOURCE;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceType, RenderGraphResourceDesc,
};

/// 仅在编译图声明 IBL 输入时导入当前源 cubemap；资源缺失由图绑定校验给出明确错误。
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_environment_ibl_graph_resources(
    graph: &CompiledRenderGraph,
    source_cubemap_view: Option<&wgpu::TextureView>,
    resources: &mut RenderGraphExecutionResources,
) {
    if !graph_declares_ibl_source_cubemap_texture(graph)
        || resources.has_texture_view(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
    {
        return;
    }

    let Some(source_cubemap_view) = source_cubemap_view else {
        return;
    };

    resources.import_borrowed_texture_view(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE, source_cubemap_view);
}

fn graph_declares_ibl_source_cubemap_texture(graph: &CompiledRenderGraph) -> bool {
    graph
        .resource_lifetime_by_name(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
        .is_some_and(|lifetime| {
            matches!(&lifetime.desc, RenderGraphResourceDesc::External)
                && lifetime.external_binding.resource_type
                    == RenderGraphExternalResourceType::Texture
        })
}

#[cfg(test)]
#[path = "tests/bind_environment_ibl_graph_resources.rs"]
mod tests;
