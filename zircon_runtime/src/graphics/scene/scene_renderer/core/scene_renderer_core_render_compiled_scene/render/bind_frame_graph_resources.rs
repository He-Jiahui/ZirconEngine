use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::OUTPUT_TARGET_TEXTURE_RESOURCE_NAME;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::resources::OutputTargetTextureResource;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionResources, RenderGraphImportedFinalTarget,
};
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::types::GraphicsError;
use crate::render_graph::{CompiledRenderGraph, RenderGraphResourceKind};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_frame_graph_resources(
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    target: &mut OffscreenTarget,
    scene_light_data_buffer: &wgpu::Buffer,
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
    bind_live_frame_target_texture_with_identity(
        graph,
        resources,
        PostProcessGraphResourceNames::SCENE_DEPTH,
        &target.depth_view,
        target.depth_identity,
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

fn bind_live_frame_target_texture_with_identity(
    graph: &CompiledRenderGraph,
    resources: &mut RenderGraphExecutionResources,
    logical_name: &'static str,
    view: &wgpu::TextureView,
    identity: crate::graphics::resource_identity::SampledTextureIdentity,
) {
    if graph_has_live_resource(graph, logical_name) {
        resources.import_borrowed_texture_view_with_identity(logical_name, view, identity);
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
    fn live_scene_velocity_missing_backing_is_fallible() {
        let source = include_str!("bind_frame_graph_resources.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(production.contains("GraphicsError::MissingFrameGraphResourceBacking"));
        assert!(!production.contains(".expect("));
    }

    #[test]
    fn frame_binder_imports_only_live_compiled_frame_resources() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let graph = live_frame_resource_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let scene_light_data = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene-light-data-test"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        bind_frame_graph_resources(
            &backend.device,
            &graph,
            &mut resources,
            &mut target,
            &scene_light_data,
            None,
            None,
            None,
        )
        .expect("live frame resources should bind");

        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_COLOR));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION));
        let ambient_occlusion_desc = resources
            .physical_texture_desc(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)
            .expect("the schema-backed SSAO external must retain its physical descriptor");
        assert_eq!(ambient_occlusion_desc.format, TextureFormat::Rgba8Unorm);
        assert!(ambient_occlusion_desc.usage.contains(TextureUsage::STORAGE));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::FINAL_COLOR));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::LIGHT_LIST));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::SCENE_LIGHT_DATA));
        assert!(
            target.scene_velocity().is_none(),
            "a compiled graph without scene velocity must not allocate its fixed backing"
        );
        assert_eq!(
            target.retained_frame_texture_count(),
            OffscreenTarget::RETAINED_FRAME_TEXTURE_COUNT,
            "an unused scene velocity target must not increase retained frame memory"
        );
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
        let mut target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let graph = live_scene_target_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let cluster_buffer = target.cluster_buffer.clone();

        bind_frame_graph_resources(
            &backend.device,
            &graph,
            &mut resources,
            &mut target,
            &cluster_buffer,
            None,
            None,
            None,
        )
        .expect("live scene targets should bind");

        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_COLOR));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_VELOCITY));
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
                .physical_texture(PostProcessGraphResourceNames::SCENE_VELOCITY)
                .is_some(),
            "scene-velocity debug readback requires the retained frame texture owner"
        );
        assert_eq!(
            resources
                .physical_texture_desc(PostProcessGraphResourceNames::SCENE_VELOCITY)
                .map(|desc| desc.format),
            Some(crate::rhi::TextureFormat::Rg16Float)
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
        assert!(
            resources
                .owned_texture(PostProcessGraphResourceNames::SCENE_VELOCITY)
                .is_none(),
            "scene-velocity must stay bound to the fixed frame target instead of a graph-owned transient"
        );
        let report = resources.resource_report();
        assert_eq!(report.texture_view_count, 3);
        assert_eq!(report.external_texture_view_count, 3);
        assert_eq!(report.owned_texture_count, 0);
    }

    #[test]
    fn frame_binder_rebinds_live_final_aliases_to_imported_texture_target() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let mut target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let imported_view = imported.create_view(&wgpu::TextureViewDescriptor::default());
        let graph = final_alias_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let cluster_buffer = target.cluster_buffer.clone();

        bind_frame_graph_resources(
            &backend.device,
            &graph,
            &mut resources,
            &mut target,
            &cluster_buffer,
            Some(RenderGraphImportedFinalTarget {
                texture: &imported,
                view: &imported_view,
                desc: TextureDesc::new(
                    "zircon-test-imported-final-target",
                    16,
                    16,
                    TextureFormat::Rgba8UnormSrgb,
                    TextureUsage::RENDER_ATTACHMENT
                        | TextureUsage::SAMPLED
                        | TextureUsage::COPY_SRC,
                ),
            }),
            None,
            None,
        )
        .expect("imported final aliases should bind");

        for &resource in FINAL_TARGET_ALIASES {
            assert!(
                resources.has_texture_view(resource),
                "`{resource}` should bind to the imported final target when live"
            );
            assert!(resources.physical_texture(resource).is_some());
            assert!(resources.physical_texture_desc(resource).is_some());
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
        let mut target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let graph = advanced_transient_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let cluster_buffer = target.cluster_buffer.clone();

        bind_frame_graph_resources(
            &backend.device,
            &graph,
            &mut resources,
            &mut target,
            &cluster_buffer,
            None,
            None,
            None,
        )
        .expect("advanced transient frame resources should bind");

        for resource in ADVANCED_POST_PROCESS_TRANSIENTS {
            assert!(
                !resources.has_texture_view(resource),
                "`{resource}` should be graph-owned, not pre-bound to the fixed offscreen target"
            );
        }

        let mut transient_pool = TransientResourcePool::default();
        transient_pool.begin_frame(backend.device_profile());
        resources
            .materialize_transient_resources_with_pool(
                &backend.device,
                backend.device_profile(),
                &graph,
                &mut transient_pool,
            )
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
        let scene_color = builder.import_present_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let ambient_occlusion = builder.import_present_external_resource_with_binding(
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let final_color = builder.import_present_external_resource_with_binding(
            PostProcessGraphResourceNames::FINAL_COLOR,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let light_list = builder.import_present_external_resource_with_binding(
            PostProcessGraphResourceNames::LIGHT_LIST,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        );
        let scene_light_data = builder.import_present_external_resource_with_binding(
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
        let scene_color = builder.import_present_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let scene_depth = builder.import_present_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let scene_velocity = builder.import_present_external_resource_with_binding(
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let pass = side_effect_pass(&mut builder, "scene-target-use");
        builder.read_external(pass, scene_color).unwrap();
        builder.read_external(pass, scene_depth).unwrap();
        builder.read_external(pass, scene_velocity).unwrap();
        builder.compile().unwrap()
    }

    fn final_alias_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("final-alias-binding");
        let pass = side_effect_pass(&mut builder, "final-alias-use");
        for &alias in FINAL_TARGET_ALIASES {
            let external = builder.import_present_external_resource_with_binding(
                alias,
                RenderGraphExternalResourceBinding::report_only_texture(),
            );
            builder.write_external(pass, external).unwrap();
        }
        builder.compile().unwrap()
    }

    fn advanced_transient_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("advanced-transient-binding");
        let output =
            builder.import_present_external_resource(PostProcessGraphResourceNames::FINAL_COLOR);
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
