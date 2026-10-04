use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::pipeline::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME;
use crate::graphics::scene::resources::OutputTargetTextureResource;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderGraphImportedFinalTarget,
};
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::types::GraphicsError;
use crate::render_graph::{CompiledRenderGraph, RenderGraphResourceKind};
use crate::rhi::{BufferDesc, TextureDesc, TextureFormat, TextureUsage};

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_frame_graph_resources(
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    target: &mut OffscreenTarget,
    scene_light_data_buffer: &wgpu::Buffer,
    scene_light_data_desc: BufferDesc,
    imported_final_target: Option<RenderGraphImportedFinalTarget<'_>>,
    output_target_resource: Option<&OutputTargetTextureResource>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
) -> Result<(), GraphicsError> {
    let retained_texture_count = target.retained_frame_texture_count();
    debug_assert!(
        retained_texture_count == OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT
            || retained_texture_count == OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT + 1,
        "fixed offscreen frame target must retain every WGPU texture owner backing imported views"
    );

    bind_live_frame_target_owned_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::SCENE_COLOR,
        &target.scene_color,
        &target.scene_color_view,
        target.scene_color_identity,
        TextureDesc::new(
            PostProcessGraphResourceNames::SCENE_COLOR,
            target.render_size.x,
            target.render_size.y,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
        ),
    );
    bind_live_frame_target_physical_texture_with_identity(
        graph,
        resources,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        &target.depth,
        &target.depth_view,
        target.depth_identity,
        TextureDesc::new(
            PostProcessGraphResourceNames::SCENE_DEPTH,
            target.render_size.x,
            target.render_size.y,
            TextureFormat::Depth32Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ),
    );
    bind_live_scene_velocity(device, graph, resources, target)?;
    bind_live_final_target_aliases(graph, resources, target, imported_final_target);
    bind_live_output_target(graph, resources, output_target_resource)?;
    bind_live_frame_target_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        &target.gbuffer_albedo_view,
    );
    bind_live_frame_target_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        &target.normal_view,
    );
    bind_live_frame_target_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        &target.gbuffer_material_view,
    );
    bind_live_frame_target_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::GBUFFER_EMISSIVE,
        &target.gbuffer_emissive_view,
    );
    bind_live_frame_target_physical_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        &target.ambient_occlusion,
        &target.ambient_occlusion_view,
        TextureDesc::new(
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            target.render_size.x,
            target.render_size.y,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT
                | TextureUsage::SAMPLED
                | TextureUsage::STORAGE
                | TextureUsage::COPY_SRC,
        ),
    );
    bind_live_frame_target_report_only_buffer(
        graph,
        resources,
        PostProcessGraphResourceNames::LIGHT_LIST,
        &target.cluster_buffer,
    );
    bind_live_frame_target_physical_buffer(
        graph,
        resources,
        PostProcessGraphResourceNames::SCENE_LIGHT_DATA,
        scene_light_data_buffer,
        scene_light_data_desc,
    );
    if let Some(shadow_atlas_resources) = shadow_atlas_resources {
        bind_live_frame_target_texture(
            graph,
            resources,
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            shadow_atlas_resources.atlas_view(),
        );
    }
    Ok(())
}

fn bind_live_final_target_aliases(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    target: &OffscreenTarget,
    imported_final_target: Option<RenderGraphImportedFinalTarget<'_>>,
) {
    for &alias in FINAL_TARGET_ALIASES {
        if !graph_has_live_resource(graph, alias) {
            continue;
        }
        if let Some(imported_final_target) = imported_final_target.as_ref() {
            let mut desc = imported_final_target.desc.clone();
            desc.label = Some(alias.to_string());
            resources.import_borrowed_texture(
                alias,
                imported_final_target.texture,
                imported_final_target.view,
                desc,
            );
        } else {
            resources.import_borrowed_texture(
                alias,
                &target.final_color,
                &target.final_color_view,
                TextureDesc::new(
                    alias,
                    target.size.x,
                    target.size.y,
                    TextureFormat::Rgba8UnormSrgb,
                    TextureUsage::RENDER_ATTACHMENT
                        | TextureUsage::SAMPLED
                        | TextureUsage::COPY_SRC,
                ),
            );
        }
    }
}

fn bind_live_output_target(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    output_target_resource: Option<&OutputTargetTextureResource>,
) -> Result<(), GraphicsError> {
    if !graph_has_live_resource(graph, OUTPUT_TARGET_TEXTURE_RESOURCE_NAME) {
        return Ok(());
    }
    let Some(output_target_resource) = output_target_resource else {
        return Ok(());
    };
    resources.import_borrowed_texture(
        OUTPUT_TARGET_TEXTURE_RESOURCE_NAME,
        output_target_resource.texture(),
        output_target_resource.view(),
        output_target_resource.graph_texture_desc(OUTPUT_TARGET_TEXTURE_RESOURCE_NAME)?,
    );
    Ok(())
}

fn bind_live_frame_target_texture(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    view: &wgpu::TextureView,
) {
    if graph_has_live_resource(graph, logical_name) {
        resources.import_borrowed_texture_view(logical_name, view);
    }
}

fn bind_live_frame_target_physical_texture_with_identity(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    texture: &wgpu::Texture,
    view: &wgpu::TextureView,
    identity: crate::graphics::resource_identity::SampledTextureIdentity,
    desc: TextureDesc,
) {
    if graph_has_live_resource(graph, logical_name) {
        resources.import_borrowed_texture_with_identity(
            logical_name,
            texture,
            view,
            desc,
            identity,
        );
    }
}

fn bind_live_scene_velocity(
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    target: &mut OffscreenTarget,
) -> Result<(), GraphicsError> {
    let logical_name = PostProcessGraphResourceNames::SCENE_VELOCITY;
    if !graph_has_live_resource(graph, logical_name) {
        return Ok(());
    }
    target.ensure_scene_velocity(device);
    let (texture, view, identity) =
        target
            .scene_velocity()
            .ok_or(GraphicsError::MissingFrameGraphResourceBacking {
                resource: logical_name,
            })?;
    bind_live_frame_target_owned_texture(
        graph,
        resources,
        logical_name,
        texture,
        view,
        identity,
        TextureDesc::new(
            logical_name,
            target.render_size.x,
            target.render_size.y,
            TextureFormat::Rg16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
        ),
    );
    Ok(())
}

fn bind_live_frame_target_owned_texture(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    texture: &wgpu::Texture,
    view: &wgpu::TextureView,
    identity: crate::graphics::resource_identity::SampledTextureIdentity,
    desc: TextureDesc,
) {
    if graph_has_live_resource(graph, logical_name) {
        resources.import_borrowed_texture_with_identity(
            logical_name,
            texture,
            view,
            desc,
            identity,
        );
    }
}

fn bind_live_frame_target_physical_texture(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    texture: &wgpu::Texture,
    view: &wgpu::TextureView,
    desc: TextureDesc,
) {
    if graph_has_live_resource(graph, logical_name) {
        resources.import_borrowed_texture(logical_name, texture, view, desc);
    }
}

fn bind_live_frame_target_report_only_buffer(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    buffer: &wgpu::Buffer,
) {
    if graph_has_live_external_resource(graph, logical_name) {
        resources.insert_buffer(logical_name, buffer.clone());
    }
}

fn bind_live_frame_target_physical_buffer(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    buffer: &wgpu::Buffer,
    desc: BufferDesc,
) {
    if graph_has_live_external_resource(graph, logical_name) {
        resources.import_borrowed_buffer_with_physical_desc(logical_name, buffer, desc);
    }
}

fn graph_has_live_resource(graph: &CompiledRenderGraph, logical_name: &str) -> bool {
    graph.resource_lifetime_by_name(logical_name).is_some()
}

fn graph_has_live_external_resource(graph: &CompiledRenderGraph, logical_name: &str) -> bool {
    graph
        .resource_lifetime_by_name(logical_name)
        .is_some_and(|lifetime| lifetime.kind == RenderGraphResourceKind::External)
}

const FINAL_TARGET_ALIASES: &[&str] = &[
    PostProcessGraphResourceNames::FINAL_COLOR,
    PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
    PostProcessGraphResourceNames::FINAL_COMPOSITED,
    PostProcessGraphResourceNames::COLOR_GRADED,
    PostProcessGraphResourceNames::EFFECT_STACKED,
];

#[cfg(test)]
#[path = "tests/bind_frame_graph_resources.rs"]
mod tests;
