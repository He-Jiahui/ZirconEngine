use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::render_graph::CompiledRenderGraph;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene)
struct HistoryGraphResourceBindingFlags
{
    pub taa_scene_color: bool,
    pub screen_space_reflection: bool,
    pub hzb: bool,
    pub hybrid_global_illumination: bool,
    pub exposure: bool,
    pub volumetric_scattering: bool,
}

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_history_graph_resources(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    history_textures: Option<&SceneFrameHistoryTextures>,
    flags: HistoryGraphResourceBindingFlags,
) {
    let Some(history_textures) = history_textures else {
        return;
    };

    if flags.taa_scene_color {
        if let (Some(texture), Some(view), Some(desc), Some(identity)) = (
            history_textures.taa_scene_color_previous_texture(),
            history_textures.taa_scene_color_previous_view(),
            history_textures
                .taa_scene_color_desc(PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS),
            history_textures.taa_scene_color_previous_identity(),
        ) {
            bind_live_taa_texture(
                graph,
                resources,
                PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
                texture,
                view,
                desc,
                identity,
            );
        }
        if let (Some(texture), Some(view), Some(desc), Some(identity)) = (
            history_textures.taa_scene_color_current_texture(),
            history_textures.taa_scene_color_current_view(),
            history_textures
                .taa_scene_color_desc(PostProcessGraphResourceNames::TAA_HISTORY_CURRENT),
            history_textures.taa_scene_color_current_identity(),
        ) {
            bind_live_taa_texture(
                graph,
                resources,
                PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
                texture,
                view,
                desc,
                identity,
            );
        }
    }

    if flags.screen_space_reflection {
        if let (Some(texture), Some(view), Some(desc)) = (
            history_textures.screen_space_reflection_texture(),
            history_textures.screen_space_reflection_view(),
            history_textures.screen_space_reflection_desc(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION,
            ),
        ) {
            bind_live_physical_texture(
                graph,
                resources,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION,
                texture,
                view,
                desc,
            );
        }
    }

    if flags.hzb {
        if let (Some(texture), Some(view), Some(desc)) = (
            history_textures.hzb_furthest_texture(),
            history_textures.hzb_furthest_view(),
            history_textures
                .hzb_furthest_desc(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST),
        ) {
            bind_live_physical_texture(
                graph,
                resources,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
                texture,
                view,
                desc,
            );
        }
    }

    if flags.hybrid_global_illumination {
        if let (Some(texture), Some(view), Some(desc)) = (
            history_textures.global_illumination_texture(),
            history_textures.global_illumination_view(),
            history_textures.global_illumination_desc(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
            ),
        ) {
            bind_live_physical_texture(
                graph,
                resources,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
                texture,
                view,
                desc,
            );
        }
        if let (Some(texture), Some(view), Some(desc)) = (
            history_textures.global_illumination_temporal_metadata_texture(),
            history_textures.global_illumination_temporal_metadata_view(),
            history_textures.global_illumination_desc(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
            ),
        ) {
            bind_live_physical_texture(
                graph,
                resources,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
                texture,
                view,
                desc,
            );
        }
    }

    if flags.exposure {
        if let (Some(buffer), Some(desc)) = (
            history_textures.exposure_previous_buffer(),
            history_textures.exposure_buffer_desc(PostProcessGraphResourceNames::EXPOSURE_PREVIOUS),
        ) {
            bind_live_physical_buffer(
                graph,
                resources,
                PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
                buffer,
                desc,
            );
        }
        if let (Some(buffer), Some(desc)) = (
            history_textures.exposure_current_buffer(),
            history_textures.exposure_buffer_desc(PostProcessGraphResourceNames::EXPOSURE_CURRENT),
        ) {
            bind_live_physical_buffer(
                graph,
                resources,
                PostProcessGraphResourceNames::EXPOSURE_CURRENT,
                buffer,
                desc,
            );
        }
    }

    if flags.volumetric_scattering {
        if let (Some(texture), Some(view), Some(desc)) = (
            history_textures.volumetric_history_texture(),
            history_textures.volumetric_history_view(),
            history_textures.volumetric_history_desc(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
            ),
        ) {
            bind_live_physical_texture(
                graph,
                resources,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
                texture,
                view,
                desc,
            );
        }
    }
}

fn bind_live_physical_texture(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    texture: &wgpu::Texture,
    view: &wgpu::TextureView,
    desc: crate::rhi::TextureDesc,
) {
    if graph.resource_lifetime_by_name(logical_name).is_some() {
        resources.import_borrowed_texture(logical_name, texture, view, desc);
    }
}

fn bind_live_taa_texture(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    texture: &wgpu::Texture,
    view: &wgpu::TextureView,
    desc: crate::rhi::TextureDesc,
    identity: crate::graphics::resource_identity::SampledTextureIdentity,
) {
    if graph.resource_lifetime_by_name(logical_name).is_some() {
        resources.import_borrowed_texture_with_identity(
            logical_name,
            texture,
            view,
            desc,
            identity,
        );
    }
}

fn bind_live_physical_buffer(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    buffer: &wgpu::Buffer,
    desc: crate::rhi::BufferDesc,
) {
    if graph.resource_lifetime_by_name(logical_name).is_some() {
        resources.import_borrowed_buffer_with_physical_desc(logical_name, buffer, desc);
    }
}

#[cfg(test)]
#[path = "tests/bind_history_graph_resources.rs"]
mod tests;
