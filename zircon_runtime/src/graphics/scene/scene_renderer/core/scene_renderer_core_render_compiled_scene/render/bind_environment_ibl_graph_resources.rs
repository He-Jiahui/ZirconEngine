use crate::graphics::scene::scene_renderer::environment::ibl_bake_graph_plan::IBL_BAKE_SOURCE_CUBEMAP_RESOURCE;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceType, RenderGraphResourceDesc,
};

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
mod tests {
    use crate::core::framework::render::{
        IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams,
    };
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::environment::ibl_bake_graph_plan::append_ibl_bake_artifact_graph_plan;
    use crate::graphics::scene::scene_renderer::graph_execution::TransientResourcePool;
    use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};

    use super::*;

    #[test]
    fn environment_ibl_source_binder_imports_only_when_graph_declares_source_cubemap() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = ibl_bake_graph();
        let texture = create_source_cubemap_texture(&backend.device);
        let view = texture.create_view(&source_cubemap_view_descriptor());
        let mut resources = RenderGraphExecutionResources::new();

        bind_environment_ibl_graph_resources(&graph, Some(&view), &mut resources);

        assert!(resources.has_texture_view(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE));

        let graph_without_ibl = graph_without_ibl_source_cubemap();
        let mut unrelated_resources = RenderGraphExecutionResources::new();
        bind_environment_ibl_graph_resources(
            &graph_without_ibl,
            Some(&view),
            &mut unrelated_resources,
        );

        assert!(!unrelated_resources.has_texture_view(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE));
    }

    #[test]
    fn environment_ibl_source_binder_preserves_missing_required_source_when_frame_has_no_view() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = ibl_bake_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let mut transient_pool = TransientResourcePool::default();
        transient_pool.begin_frame();
        resources
            .materialize_transient_resources_with_pool(&backend.device, &graph, &mut transient_pool)
            .expect("IBL bake transient resources should materialize");

        bind_environment_ibl_graph_resources(&graph, None, &mut resources);

        assert!(!resources.has_texture_view(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE));
        let error = resources
            .validate_materialized_graph_resources(&graph)
            .expect_err("missing required IBL source must fail materialization validation");
        assert!(error.contains("required external resource bindings"));
        assert!(error.contains("external texture `environment.ibl.source_cubemap`"));
    }

    fn ibl_bake_graph() -> CompiledRenderGraph {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            4,
            3,
        )
        .with_required_contents(IblBakeArtifactContents::PMREM);
        let mut builder = RenderGraphBuilder::new("environment-ibl-source-binding");
        append_ibl_bake_artifact_graph_plan(&mut builder, &request)
            .expect("IBL bake graph plan should append");
        builder.compile().expect("IBL bake graph should compile")
    }

    fn graph_without_ibl_source_cubemap() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("environment-without-ibl-source");
        let pass = builder.add_pass_with_executor("side-effect", QueueLane::Graphics, Some("noop"));
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .expect("side effect pass should be rootable");
        builder.compile().expect("graph should compile")
    }

    fn create_source_cubemap_texture(device: &wgpu::Device) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-test-environment-ibl-source-cube"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 6,
            },
            mip_level_count: 3,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    fn source_cubemap_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
        wgpu::TextureViewDescriptor {
            label: Some("zircon-test-environment-ibl-source-cube-view"),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(3),
            base_array_layer: 0,
            array_layer_count: Some(6),
        }
    }
}
