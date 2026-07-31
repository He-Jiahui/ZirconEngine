use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderGraphImportedFinalTarget,
};
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::render_graph::{CompiledRenderGraph, RenderGraphResourceKind};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_frame_graph_resources(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    target: &OffscreenTarget,
    scene_light_data_buffer: &wgpu::Buffer,
    imported_final_target: Option<RenderGraphImportedFinalTarget<'_>>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
) {
    debug_assert_eq!(
        target.retained_frame_texture_count(),
        OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT,
        "fixed offscreen frame target must retain every WGPU texture owner backing imported views"
    );

    bind_live_frame_target_owned_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::SCENE_COLOR,
        &target.scene_color,
        &target.scene_color_view,
        TextureDesc::new(
            PostProcessGraphResourceNames::SCENE_COLOR,
            target.render_size.x,
            target.render_size.y,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
        ),
    );
    bind_live_frame_target_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        &target.depth_view,
    );
    bind_live_final_target_aliases(graph, resources, target, imported_final_target);
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
    bind_live_frame_target_texture(
        graph,
        resources,
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        &target.ambient_occlusion_view,
    );
    bind_live_frame_target_buffer(
        graph,
        resources,
        PostProcessGraphResourceNames::LIGHT_LIST,
        &target.cluster_buffer,
    );
    bind_live_frame_target_buffer(
        graph,
        resources,
        PostProcessGraphResourceNames::SCENE_LIGHT_DATA,
        scene_light_data_buffer,
    );
    if let Some(shadow_atlas_resources) = shadow_atlas_resources {
        bind_live_frame_target_texture(
            graph,
            resources,
            PostProcessGraphResourceNames::SHADOW_ATLAS,
            shadow_atlas_resources.atlas_view(),
        );
    }
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
        if let Some(imported_final_target) = imported_final_target {
            resources.import_borrowed_texture_view(alias, imported_final_target.view);
        } else {
            resources.import_texture_alias(alias, &target.final_color);
        }
    }
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

fn bind_live_frame_target_owned_texture(
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

fn bind_live_frame_target_buffer(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    buffer: &wgpu::Buffer,
) {
    if graph_has_live_external_resource(graph, logical_name) {
        resources.insert_buffer(logical_name, buffer.clone());
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
mod tests {
    use crate::core::framework::render::PostProcessGraphResourceNames;
    use crate::core::math::UVec2;
    use crate::graphics::backend::{OffscreenTarget, RenderBackend};
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphExecutionResources, RenderGraphImportedFinalTarget, TransientResourcePool,
    };
    use crate::render_graph::{
        CompiledRenderGraph, PassFlags, QueueLane, RenderGraphBuilder,
        RenderGraphExternalResourceBinding,
    };

    use super::*;

    #[test]
    fn frame_binder_imports_only_live_compiled_frame_resources() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let graph = live_frame_resource_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let scene_light_data = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene-light-data-test"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        bind_frame_graph_resources(
            &graph,
            &mut resources,
            &target,
            &scene_light_data,
            None,
            None,
        );

        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_COLOR));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::FINAL_COLOR));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::LIGHT_LIST));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::SCENE_LIGHT_DATA));
        assert!(
            !resources.has_texture_view(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION),
            "unused fixed frame targets must not be pre-bound into the graph resource table"
        );
        assert!(
            !resources.has_texture_view(PostProcessGraphResourceNames::BLOOM),
            "unused optional post-process frame targets must stay absent until the graph declares them"
        );
    }

    #[test]
    fn frame_binder_reuses_fixed_scene_color_and_depth_targets() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let graph = live_scene_target_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_frame_graph_resources(
            &graph,
            &mut resources,
            &target,
            &target.cluster_buffer,
            None,
            None,
        );

        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_COLOR));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH));
        assert!(
            resources
                .physical_texture(PostProcessGraphResourceNames::SCENE_COLOR)
                .is_some(),
            "scene-color copy consumers require the retained frame texture owner"
        );
        assert_eq!(
            resources
                .physical_texture_desc(PostProcessGraphResourceNames::SCENE_COLOR)
                .map(|desc| desc.format),
            Some(crate::rhi::TextureFormat::Rgba16Float)
        );
        assert!(
            resources
                .owned_texture(PostProcessGraphResourceNames::SCENE_COLOR)
                .is_none(),
            "scene-color must stay bound to the fixed frame target instead of a graph-owned transient"
        );
        assert!(
            resources
                .owned_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
                .is_none(),
            "scene-depth must stay bound to the fixed frame target instead of a graph-owned transient"
        );
        let report = resources.resource_report();
        assert_eq!(report.texture_view_count, 2);
        assert_eq!(report.external_texture_view_count, 2);
        assert_eq!(report.owned_texture_count, 0);
    }

    #[test]
    fn frame_binder_rebinds_live_final_aliases_to_imported_texture_target() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let imported = backend.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-test-imported-final-target"),
            size: wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let imported_view = imported.create_view(&wgpu::TextureViewDescriptor::default());
        let graph = final_alias_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_frame_graph_resources(
            &graph,
            &mut resources,
            &target,
            &target.cluster_buffer,
            Some(RenderGraphImportedFinalTarget {
                view: &imported_view,
            }),
            None,
        );

        for &resource in FINAL_TARGET_ALIASES {
            assert!(
                resources.has_texture_view(resource),
                "`{resource}` should bind to the imported final target when live"
            );
        }
        let report = resources.resource_report();
        assert!(
            report.external_texture_view_count >= FINAL_TARGET_ALIASES.len(),
            "imported final target aliases should count as external graph views; report={report:?}"
        );
    }

    #[test]
    fn frame_binder_leaves_advanced_transients_for_materialization() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let graph = advanced_transient_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_frame_graph_resources(
            &graph,
            &mut resources,
            &target,
            &target.cluster_buffer,
            None,
            None,
        );

        for resource in ADVANCED_POST_PROCESS_TRANSIENTS {
            assert!(
                !resources.has_texture_view(resource),
                "`{resource}` should be graph-owned, not pre-bound to the fixed offscreen target"
            );
        }

        let mut transient_pool = TransientResourcePool::default();
        transient_pool.begin_frame();
        resources
            .materialize_transient_resources_with_pool(&backend.device, &graph, &mut transient_pool)
            .expect("advanced post-process transient graph resources should materialize");

        for resource in ADVANCED_POST_PROCESS_TRANSIENTS {
            assert!(
                resources.has_texture_view(resource),
                "`{resource}` should be backed by graph materialization"
            );
        }
    }

    fn live_frame_resource_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("live-frame-resource-binding");
        let scene_color = builder.import_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let ambient_occlusion = builder.import_external_resource_with_binding(
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let final_color = builder.import_external_resource_with_binding(
            PostProcessGraphResourceNames::FINAL_COLOR,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let light_list = builder.import_external_resource_with_binding(
            PostProcessGraphResourceNames::LIGHT_LIST,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        );
        let scene_light_data = builder.import_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_LIGHT_DATA,
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = side_effect_pass(&mut builder, "frame-resource-use");
        builder.read_external(pass, scene_color).unwrap();
        builder.read_external(pass, ambient_occlusion).unwrap();
        builder.read_external(pass, light_list).unwrap();
        builder.read_external(pass, scene_light_data).unwrap();
        builder.write_external(pass, final_color).unwrap();
        builder.compile().unwrap()
    }

    fn live_scene_target_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("live-scene-target-binding");
        let scene_color = builder.import_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let scene_depth = builder.import_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let pass = side_effect_pass(&mut builder, "scene-target-use");
        builder.read_external(pass, scene_color).unwrap();
        builder.read_external(pass, scene_depth).unwrap();
        builder.compile().unwrap()
    }

    fn final_alias_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("final-alias-binding");
        let pass = side_effect_pass(&mut builder, "final-alias-use");
        for &alias in FINAL_TARGET_ALIASES {
            let external = builder.import_external_resource_with_binding(
                alias,
                RenderGraphExternalResourceBinding::report_only_texture(),
            );
            builder.write_external(pass, external).unwrap();
        }
        builder.compile().unwrap()
    }

    fn advanced_transient_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("advanced-transient-binding");
        let output = builder.import_external_resource(PostProcessGraphResourceNames::FINAL_COLOR);
        let pass = side_effect_pass(&mut builder, "advanced-transient-use");
        for resource in ADVANCED_POST_PROCESS_TRANSIENTS {
            let texture = builder.create_texture(crate::rhi::TextureDesc::new(
                *resource,
                16,
                16,
                crate::rhi::TextureFormat::Rgba16Float,
                crate::rhi::TextureUsage::RENDER_ATTACHMENT | crate::rhi::TextureUsage::SAMPLED,
            ));
            builder.write_texture(pass, texture).unwrap();
        }
        builder.write_external(pass, output).unwrap();
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

    const ADVANCED_POST_PROCESS_TRANSIENTS: &[&str] = &[
        PostProcessGraphResourceNames::SCENE_VELOCITY,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
        PostProcessGraphResourceNames::COLOR_LUT,
        PostProcessGraphResourceNames::HZB_FURTHEST,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
    ];
}
