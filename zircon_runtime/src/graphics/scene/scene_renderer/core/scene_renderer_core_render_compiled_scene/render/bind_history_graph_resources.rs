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
mod tests {
    use crate::core::framework::render::FroxelGridQuality;
    use crate::core::math::UVec2;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
    use crate::graphics::scene::scene_renderer::history::{
        SceneFrameHistoryRequirements, SceneFrameHistoryTextures,
    };
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
        let requirements = SceneFrameHistoryRequirements::new(
            true,
            true,
            true,
            true,
            true,
            Some(FroxelGridQuality::High),
        );
        let (mut history_textures, _) =
            SceneFrameHistoryTextures::new_with_requirements_and_initialization(
                &backend.device,
                UVec2::new(16, 16),
                UVec2::new(16, 16),
                requirements,
            );
        let mut frame = history_textures.begin_history_frame();
        let mut writes =
            crate::graphics::scene::scene_renderer::history::SceneHistoryWriteIntent::default();
        writes.record(
            crate::graphics::scene::scene_renderer::history::SceneHistoryDomain::VolumetricScattering,
            true,
        );
        frame.absorb_writes(writes);
        history_textures.commit_history_frame(frame, 1);
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
        assert!(
            resources
                .physical_texture(PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS)
                .is_some()
        );
        assert!(
            resources
                .physical_texture(PostProcessGraphResourceNames::TAA_HISTORY_CURRENT)
                .is_some()
        );
        assert_eq!(
            resources
                .physical_texture_desc(PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS)
                .map(|desc| (desc.width, desc.height, desc.format)),
            Some((16, 16, crate::rhi::TextureFormat::Rgba16Float))
        );
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
        ));
        assert!(
            resources
                .physical_texture(
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
                )
                .is_some()
        );
        assert_eq!(
            resources
                .physical_texture_desc(
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION
                )
                .map(|desc| (desc.width, desc.height, desc.format)),
            Some((16, 16, crate::rhi::TextureFormat::Rgba16Float))
        );
        assert!(
            resources
                .has_texture_view(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST)
        );
        assert!(
            resources
                .physical_texture(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST)
                .is_some()
        );
        assert_eq!(
            resources
                .physical_texture_desc(
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_HZB_FURTHEST,
                )
                .map(|desc| (desc.width, desc.height, desc.mip_levels, desc.format)),
            Some((8, 8, 4, crate::rhi::TextureFormat::Rgba16Float))
        );
        assert!(
            resources.has_texture_view(PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI)
        );
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA
        ));
        for logical_name in [
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA,
        ] {
            assert!(resources.physical_texture(logical_name).is_some());
            assert_eq!(
                resources.physical_texture_desc(logical_name).map(|desc| (
                    desc.width,
                    desc.height,
                    desc.format
                )),
                Some((16, 16, crate::rhi::TextureFormat::Rgba16Float))
            );
        }
        assert!(resources.has_buffer(PostProcessGraphResourceNames::EXPOSURE_PREVIOUS));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::EXPOSURE_CURRENT));
        for logical_name in [
            PostProcessGraphResourceNames::EXPOSURE_PREVIOUS,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
        ] {
            assert_eq!(
                resources
                    .physical_buffer_desc(logical_name)
                    .map(|desc| (desc.size_bytes, desc.usage,)),
                Some((
                    u64::from(crate::core::framework::render::EXPOSURE_BUFFER_WORD_COUNT)
                        * std::mem::size_of::<f32>() as u64,
                    crate::rhi::BufferUsage::STORAGE
                        | crate::rhi::BufferUsage::COPY_SRC
                        | crate::rhi::BufferUsage::COPY_DST,
                ))
            );
        }
        assert!(resources.has_texture_view(
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING
        ));
        assert!(
            resources
                .physical_texture(
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING
                )
                .is_some()
        );
        assert_eq!(
            resources
                .physical_texture_desc(
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
                )
                .map(|desc| (
                    desc.width,
                    desc.height,
                    desc.depth,
                    desc.dimension,
                    desc.format,
                    desc.usage,
                )),
            Some((
                160,
                90,
                96,
                crate::rhi::TextureDimension::D3,
                crate::rhi::TextureFormat::Rgba16Float,
                crate::rhi::TextureUsage::SAMPLED | crate::rhi::TextureUsage::COPY_DST,
            ))
        );

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
    fn history_binder_reuses_the_persistent_screen_space_reflection_physical_owner() {
        let source = include_str!("bind_history_graph_resources.rs");

        assert!(source.contains("screen_space_reflection_texture()"));
        assert!(source.contains("screen_space_reflection_view()"));
        assert!(source.contains("screen_space_reflection_desc("));
    }

    #[test]
    fn history_binder_skips_enabled_resources_absent_from_live_graph() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let requirements =
            SceneFrameHistoryRequirements::new(false, false, false, false, true, None);
        let (history_textures, _) =
            SceneFrameHistoryTextures::new_with_requirements_and_initialization(
                &backend.device,
                UVec2::new(16, 16),
                UVec2::new(16, 16),
                requirements,
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
        assert!(
            resources
                .physical_buffer_desc(PostProcessGraphResourceNames::EXPOSURE_PREVIOUS)
                .is_some()
        );
        assert!(
            resources
                .physical_buffer_desc(PostProcessGraphResourceNames::EXPOSURE_CURRENT)
                .is_some()
        );
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
        builder.import_present_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    fn report_only_buffer(
        builder: &mut RenderGraphBuilder,
        name: &'static str,
    ) -> crate::render_graph::ExternalResource {
        builder.import_present_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }
}
