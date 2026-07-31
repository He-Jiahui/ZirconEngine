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
        bind_live_texture_view(
            graph,
            resources,
            PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
            history_textures.taa_scene_color_previous_view(),
        );
        bind_live_texture_view(
            graph,
            resources,
            PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
            history_textures.taa_scene_color_current_view(),
        );
    }

    if flags.screen_space_reflection {
        bind_live_texture_view(
            graph,
            resources,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION,
            history_textures.screen_space_reflection_view.clone(),
        );
    }

    if flags.hzb {
        bind_live_texture_view(
            graph,
            resources,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
            history_textures.hzb_furthest_view.clone(),
        );
    }

    if flags.hybrid_global_illumination {
        bind_live_texture_view(
            graph,
            resources,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
            history_textures.global_illumination_view.clone(),
        );
        bind_live_texture_view(
            graph,
            resources,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
            history_textures
                .global_illumination_temporal_metadata_view
                .clone(),
        );
    }

    if flags.exposure {
        bind_live_buffer(
            graph,
            resources,
            PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
            history_textures.exposure_previous_buffer(),
        );
        bind_live_buffer(
            graph,
            resources,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            history_textures.exposure_current_buffer(),
        );
    }

    if flags.volumetric_scattering && history_textures.volumetric_history_valid() {
        if let Some(view) = history_textures.volumetric_history_view() {
            bind_live_texture_view(
                graph,
                resources,
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
                view,
            );
        }
    }
}

fn bind_live_texture_view(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    view: wgpu::TextureView,
) {
    if graph.resource_lifetime_by_name(logical_name).is_some() {
        resources.import_texture_view(logical_name, view);
    }
}

fn bind_live_buffer(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    buffer: wgpu::Buffer,
) {
    if graph.resource_lifetime_by_name(logical_name).is_some() {
        resources.insert_buffer(logical_name, buffer);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::FroxelGridQuality;
    use crate::core::math::UVec2;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
    use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
    use crate::render_graph::{
        CompiledRenderGraph, PassFlags, QueueLane, RenderGraphBuilder,
        RenderGraphExternalResourceBinding,
    };

    use super::*;

    #[test]
    fn history_binder_imports_enabled_live_history_externals() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let mut history_textures = SceneFrameHistoryTextures::new_with_volumetric_history(
            &backend.device,
            &backend.queue,
            UVec2::new(16, 16),
            UVec2::new(16, 16),
            Some(FroxelGridQuality::High),
        );
        history_textures.set_volumetric_history_valid(true);
        let graph = full_history_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_history_graph_resources(
            &graph,
            &mut resources,
            Some(&history_textures),
            HistoryGraphResourceBindingFlags {
                taa_scene_color: true,
                screen_space_reflection: true,
                hzb: true,
                hybrid_global_illumination: true,
                exposure: true,
                volumetric_scattering: true,
            },
        );

        assert!(resources.has_texture_view(PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::TAA_HISTORY_CURRENT));
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
        ));
        assert!(
            resources
                .has_texture_view(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST)
        );
        assert!(
            resources.has_texture_view(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI)
        );
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA
        ));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::EXPOSURE_PREVIOUS));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::EXPOSURE_CURRENT));
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING
        ));

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("live history externals should be bound before validation");
        assert_eq!(report.required_external_count, 0);
        assert_eq!(report.report_only_external_count, 9);
        assert_eq!(report.bound_report_only_external_count, 9);
        assert_eq!(report.bound_external_count(), 9);
        assert_eq!(report.missing_external_count(), 0);
    }

    #[test]
    fn history_binder_reuses_the_persistent_screen_space_reflection_view() {
        let source = include_str!("bind_history_graph_resources.rs");
        let persistent_view = ["screen_space_reflection_", "view.clone()"].concat();

        assert!(source.contains(&persistent_view));
    }

    #[test]
    fn history_binder_skips_enabled_resources_absent_from_live_graph() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let history_textures = SceneFrameHistoryTextures::new(
            &backend.device,
            &backend.queue,
            UVec2::new(16, 16),
            UVec2::new(16, 16),
        );
        let graph = exposure_history_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_history_graph_resources(
            &graph,
            &mut resources,
            Some(&history_textures),
            HistoryGraphResourceBindingFlags {
                taa_scene_color: true,
                screen_space_reflection: true,
                hzb: true,
                hybrid_global_illumination: true,
                exposure: true,
                volumetric_scattering: false,
            },
        );

        assert!(!resources.has_texture_view(PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS));
        assert!(
            !resources
                .has_texture_view(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST)
        );
        assert!(
            !resources.has_texture_view(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI)
        );
        assert!(!resources.has_texture_view(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA
        ));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::EXPOSURE_PREVIOUS));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::EXPOSURE_CURRENT));
    }

    fn full_history_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("history-external-binding");
        let taa_previous = report_only_texture(
            &mut builder,
            PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS,
        );
        let taa_current = report_only_texture(
            &mut builder,
            PostProcessGraphResourceNames::TAA_HISTORY_CURRENT,
        );
        let ssr_previous = report_only_texture(
            &mut builder,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION,
        );
        let hzb_previous = report_only_texture(
            &mut builder,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
        );
        let exposure_previous = report_only_buffer(
            &mut builder,
            PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
        );
        let exposure_current = report_only_buffer(
            &mut builder,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
        );
        let hybrid_gi_history = report_only_texture(
            &mut builder,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
        );
        let hybrid_gi_temporal_metadata_history = report_only_texture(
            &mut builder,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
        );
        let volumetric_scattering_history = report_only_texture(
            &mut builder,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
        );
        let pass = side_effect_pass(&mut builder, "history-use");
        builder.read_external(pass, taa_previous).unwrap();
        builder.write_external(pass, taa_current).unwrap();
        builder.read_external(pass, ssr_previous).unwrap();
        builder.read_external(pass, hzb_previous).unwrap();
        builder.write_external(pass, hybrid_gi_history).unwrap();
        builder
            .write_external(pass, hybrid_gi_temporal_metadata_history)
            .unwrap();
        builder.read_external(pass, exposure_previous).unwrap();
        builder
            .read_external(pass, volumetric_scattering_history)
            .unwrap();
        builder
            .write_storage_external(pass, exposure_current)
            .unwrap();
        builder.compile().unwrap()
    }

    fn exposure_history_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("exposure-history-binding");
        let exposure_previous = report_only_buffer(
            &mut builder,
            PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
        );
        let exposure_current = report_only_buffer(
            &mut builder,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
        );
        let pass = side_effect_pass(&mut builder, "exposure-history-use");
        builder.read_external(pass, exposure_previous).unwrap();
        builder
            .write_storage_external(pass, exposure_current)
            .unwrap();
        builder.compile().unwrap()
    }

    fn side_effect_pass(
        builder: &mut RenderGraphBuilder,
        name: &'static str,
    ) -> crate::render_graph::RenderPassId {
        let pass = builder.add_pass(name, QueueLane::Graphics);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        pass
    }

    fn report_only_texture(
        builder: &mut RenderGraphBuilder,
        name: &'static str,
    ) -> crate::render_graph::ExternalResource {
        builder.import_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    fn report_only_buffer(
        builder: &mut RenderGraphBuilder,
        name: &'static str,
    ) -> crate::render_graph::ExternalResource {
        builder.import_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }
}
