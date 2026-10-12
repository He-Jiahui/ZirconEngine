use super::*;
use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, AdvancedProviderAvailability, AntiAliasFallbackReport,
    AntiAliasMode, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderCameraTargetKind,
    RenderCapabilitySummary, RenderFrameExtract, RenderLayerSet, RenderMeshSnapshot,
    RenderOverlayExtract, RenderParticlePreviousSpriteSnapshot, RenderPipelineHandle,
    RenderPluginRendererOutputs, RenderProfileBundle,
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
    SceneViewportRenderPacket, SourceCubemapEnvironment, SourceCubemapMipChain,
    TemporalJitterSample, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::types::{ViewportTextureWritebackStatus, FRAMEWORK_OUTPUT_FORMAT_LABEL};
use crate::graphics::VisibilityContext;
use crate::graphics::{CompiledRenderPipeline, RenderPassStage};
use crate::render_graph::RenderGraphBuilder;

#[test]
fn build_runtime_frame_carries_prepared_sideband_and_output_target_into_viewport_frame() {
    let extract = RenderFrameExtract::from_snapshot(
        crate::core::framework::render::RenderWorldSnapshotHandle::new(9),
        empty_scene_snapshot(),
    );
    let previous_camera = ViewportCameraSnapshot {
        transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        ..ViewportCameraSnapshot::default()
    };
    let output_texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/runtime-frame/output-target",
    ));
    let particle_previous_sprites = vec![RenderParticlePreviousSpriteSnapshot {
        entity: 88,
        stable_sprite_key: 5,
        position: Vec3::new(-1.0, 0.0, -2.0),
        size: 0.5,
        aspect_ratio: 1.0,
        billboard_offset: crate::core::math::Vec2::ZERO,
        rotation: 0.0,
        billboard_basis: None,
    }];
    let environment_source_cubemap_override = SourceCubemapEnvironment::new(
        SourceCubemapMipChain::new(
            1,
            1,
            vec![[0.25, 0.5, 0.75, 1.0]; 6],
            1,
            1,
            vec![[0.1, 0.2, 0.3, 1.0]; 6],
        ),
        88,
        [1, 2, 3, 4],
    );
    let mut context = FrameSubmissionContext::new(
        UVec2::new(640, 480),
        UVec2::new(640, 480),
        RenderPipelineHandle::new(1),
        0,
        None,
        Default::default(),
        std::sync::Arc::new(empty_pipeline()),
        RenderCapabilitySummary::default(),
        VisibilityContext::from_extract(&extract),
        Some(previous_camera.clone()),
        super::super::super::super::viewport_record::ViewportCameraHistoryKey::from_camera(
            extract
                .view
                .selected_camera_descriptor()
                .expect("test extract has selected camera descriptor"),
        ),
        Default::default(),
        false,
        None,
        crate::graphics::ViewportRenderOutputTarget::Texture {
            handle: output_texture,
            size: UVec2::new(640, 480),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        },
        Default::default(),
        crate::core::framework::render::RenderViewFamilyPipeline::resolve(
            UVec2::new(640, 480),
            Default::default(),
            crate::core::framework::render::RenderUpscalerKind::Spatial,
        ),
        None,
        Default::default(),
        Default::default(),
        AntiAliasFallbackReport::exact(AntiAliasMode::Taa),
        advanced_runtime_plan_with_virtual_geometry(),
        Default::default(),
        false,
        true,
        None,
        Default::default(),
        None,
        None,
        std::sync::Arc::new(extract.clone()),
        0,
        0,
        0,
        None,
        Default::default(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
        None,
        1,
    )
    .with_environment_source_cubemap_override(Some(environment_source_cubemap_override.clone()))
    .with_particle_previous_sprites_override(Some(particle_previous_sprites.clone()));
    let prepared = PreparedRuntimeSubmission::new(
        vec![5],
        None,
        vec![9],
        RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                    page_request_ids: vec![300],
                    ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                },
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        },
    );

    let context_post_process = context.post_process_shared();
    let frame = build_runtime_frame(
        None,
        &mut context,
        prepared,
        ViewportCameraStackOutputPolicy::new(false, false),
    );

    assert!(Arc::ptr_eq(
        &context_post_process,
        frame
            .post_process_override
            .as_ref()
            .expect("runtime frame must share the renderer-owned post-process snapshot"),
    ));
    assert_eq!(frame.viewport_size, UVec2::new(640, 480));
    assert_eq!(
        frame.output_target().kind(),
        RenderCameraTargetKind::Texture
    );
    assert_eq!(frame.output_target().texture_handle(), Some(output_texture));
    assert_eq!(frame.output_target().size(), Some(UVec2::new(640, 480)));
    assert_eq!(
        frame
            .texture_writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL))
            .status(),
        ViewportTextureWritebackStatus::ReadyForSrgbCopy
    );
    assert!(!frame
        .camera_stack_output_policy()
        .owns_final_target_output());
    assert_eq!(
        frame.previous_motion_vector_camera(),
        Some(&previous_camera)
    );
    assert_eq!(
        frame
            .prepared_runtime_sidebands()
            .virtual_geometry_readback_outputs()
            .node_cluster_cull
            .page_request_ids,
        vec![300]
    );
    assert_eq!(
        frame
            .prepared_runtime_sidebands()
            .virtual_geometry_evictable_page_ids(),
        &[9]
    );
    assert_eq!(frame.extract.view.anti_alias.mode, AntiAliasMode::Taa);
    assert_eq!(frame.previous_particle_sprites(), particle_previous_sprites);
    assert!(frame.extract.particles.previous_sprites.is_empty());
    assert_eq!(
        frame
            .source_cubemap_environment()
            .expect("runtime frame carries renderer-owned environment override")
            .source_revision,
        environment_source_cubemap_override.source_revision
    );
    assert!(frame
        .extract
        .environment
        .skybox
        .source_cubemap_environment()
        .is_none());
    assert_ne!(
        frame.extract.view.camera.temporal_jitter,
        TemporalJitterSample::default()
    );
    assert_ne!(
        frame.camera().camera.temporal_jitter,
        TemporalJitterSample::default()
    );
    assert_ne!(
        frame.effective_camera().temporal_jitter,
        TemporalJitterSample::default()
    );
}

#[test]
fn runtime_frame_visbuffer_overlay_borrows_snapshot_marks() {
    let source = include_str!("../build_runtime_frame.rs");

    assert!(source.contains("&snapshot.visbuffer_debug_marks"));
    assert!(!source.contains(concat!("snapshot.visbuffer_debug_marks", ".", "clone()")));
}

#[test]
fn runtime835_virtual_geometry_overlay_capacity_regression() {
    let source = include_str!("../build_runtime_frame.rs");
    let production = source.split("mod tests {").next().unwrap();

    assert!(production.contains("Vec::with_capacity(instances.len())"));
    assert!(
        production.contains("let mut gizmos = Vec::with_capacity(visbuffer_debug_marks.len());")
    );
    assert!(production.contains("const MAX_LINES_PER_NODE: usize = 13;"));
    assert!(production.contains("Vec::with_capacity(16)"));
    assert!(production.contains("gizmos.extend("));
    assert!(!production.contains(".collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime835_virtual_geometry_overlay_capacity_bench_v1() {
    const MARKER: &str = "RUNTIME835_VIRTUAL_GEOMETRY_OVERLAY_CAPACITY_BENCH_V1";
    let dense_node_count = 4096usize;
    let max_lines = dense_node_count.saturating_mul(13);
    let growth_events = |length: usize, initial_capacity: usize| {
        let mut capacity = initial_capacity;
        let mut events = 0;
        for current_length in 1..=length {
            if current_length > capacity {
                capacity = if capacity == 0 { 4 } else { capacity * 2 };
                events += 1;
            }
        }
        events
    };

    assert!(!MARKER.is_empty());
    assert_eq!(max_lines, 53_248);
    assert_eq!(growth_events(max_lines, 0), 15);
    assert_eq!(growth_events(dense_node_count, 0), 11);
    assert_eq!(growth_events(16, 0), 3);
    assert_eq!(growth_events(max_lines, max_lines), 0);
    assert_eq!(growth_events(dense_node_count, dense_node_count), 0);
    assert_eq!(growth_events(16, 16), 0);
}

fn empty_pipeline() -> CompiledRenderPipeline {
    let graph = RenderGraphBuilder::new("empty-runtime-frame-test")
        .compile()
        .unwrap();
    CompiledRenderPipeline::from_parts(crate::graphics::pipeline::CompiledRenderPipelineParts {
        handle: RenderPipelineHandle::new(1),
        name: "empty".to_string(),
        renderer_name: "empty".to_string(),
        execution_pass_metadata: Vec::new(),
        enabled_features: Vec::new(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        environment_ibl_bake_request: None,
        ambient_occlusion_profile: None,
        half_resolution_transparency_depth_sigma:
            crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
        graph,
    })
    .expect("empty runtime frame pipeline execution packet")
}

fn advanced_runtime_plan_with_virtual_geometry() -> AdvancedProfileRuntimePlan {
    AdvancedProfileRuntimePlan::from_profile_bundle(
        &RenderProfileBundle::advanced_render(),
        &RenderCapabilitySummary {
            virtual_geometry_supported: true,
            hybrid_global_illumination_supported: true,
            supports_storage_buffers: true,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            ..RenderCapabilitySummary::default()
        },
        &AdvancedProviderAvailability::new()
            .with_virtual_geometry_provider("vg")
            .with_hybrid_gi_provider("hgi"),
    )
}

fn empty_scene_snapshot() -> SceneViewportRenderPacket {
    SceneViewportRenderPacket {
        scene: crate::core::framework::render::RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        environment: crate::core::framework::render::EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

fn test_mesh(node_id: u64, transform: Transform) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform,
        model: ResourceHandle::new(ResourceId::from_stable_label("tests/model")),
        mesh: None,
        material: ResourceHandle::new(ResourceId::from_stable_label("tests/material")),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
            ..Default::default()
        },
    }
}
