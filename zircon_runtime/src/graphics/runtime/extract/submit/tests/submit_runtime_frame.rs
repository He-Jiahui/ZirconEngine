use super::*;
use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, AdvancedProviderAvailability, FallbackSkyboxKind,
    PreviewEnvironmentExtract, RenderCameraTargetKind, RenderCapabilitySummary, RenderFrameExtract,
    RenderOverlayExtract, RenderPluginRendererOutputs, RenderProfileBundle,
    RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};

use super::super::super::prepared_runtime_submission::PreparedRuntimeSubmission;

#[test]
fn direct_runtime_frame_submit_projects_prepared_sidebands() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(44),
        empty_scene_snapshot(),
    );
    let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 720));
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

    attach_prepared_sidebands_to_runtime_frame(&mut frame, prepared);

    assert_eq!(
        frame
            .prepared_runtime_sidebands()
            .hybrid_gi_evictable_probe_ids(),
        &[5]
    );
    assert_eq!(
        frame
            .prepared_runtime_sidebands()
            .virtual_geometry_evictable_page_ids(),
        &[9]
    );
    assert_eq!(
        frame
            .prepared_runtime_sidebands()
            .virtual_geometry_readback_outputs()
            .node_cluster_cull
            .page_request_ids,
        vec![300]
    );
}

#[test]
fn direct_runtime_frame_submit_projects_resolved_output_target() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(45),
        empty_scene_snapshot(),
    );
    let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(1280, 720));
    let mut context = frame_submission_context_with_output_target(
        UVec2::new(96, 54),
        crate::graphics::ViewportRenderOutputTarget::Headless {
            size: UVec2::new(96, 54),
        },
    );

    apply_submission_extract_to_runtime_frame(&mut frame, &mut context);
    apply_submission_output_target_to_runtime_frame(&mut frame, &context);

    assert_eq!(frame.viewport_size, UVec2::new(96, 54));
    assert_eq!(
        frame.output_target().kind(),
        RenderCameraTargetKind::Headless
    );
    assert_eq!(frame.output_target().size(), Some(UVec2::new(96, 54)));
}

fn empty_scene_snapshot() -> RenderSceneSnapshot {
    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
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

fn frame_submission_context_with_output_target(
    size: UVec2,
    output_target: crate::graphics::ViewportRenderOutputTarget,
) -> super::super::super::frame_submission_context::FrameSubmissionContext {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(46),
        empty_scene_snapshot(),
    );
    super::super::super::frame_submission_context::FrameSubmissionContext::new(
        size,
        size,
        crate::core::framework::render::RenderPipelineHandle::new(1),
        0,
        None,
        Default::default(),
        std::sync::Arc::new(empty_pipeline()),
        RenderCapabilitySummary::default(),
        crate::graphics::VisibilityContext::from_extract(&extract),
        None,
        super::super::super::super::viewport_record::ViewportCameraHistoryKey::from_camera(
            extract
                .view
                .selected_camera_descriptor()
                .expect("test extract has selected camera descriptor"),
        ),
        Default::default(),
        false,
        None,
        output_target,
        Default::default(),
        crate::core::framework::render::RenderViewFamilyPipeline::resolve(
            size,
            Default::default(),
            crate::core::framework::render::RenderUpscalerKind::Spatial,
        ),
        None,
        Default::default(),
        Default::default(),
        Default::default(),
        default_render_advanced_runtime_plan(),
        Default::default(),
        false,
        false,
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
}

fn empty_pipeline() -> crate::graphics::CompiledRenderPipeline {
    let graph = crate::render_graph::RenderGraphBuilder::new("direct-runtime-frame-test")
        .compile()
        .unwrap();
    crate::graphics::CompiledRenderPipeline::from_parts(
        crate::graphics::pipeline::CompiledRenderPipelineParts {
            handle: crate::core::framework::render::RenderPipelineHandle::new(1),
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
        },
    )
    .expect("empty direct runtime frame pipeline execution packet")
}

fn default_render_advanced_runtime_plan() -> AdvancedProfileRuntimePlan {
    AdvancedProfileRuntimePlan::from_profile_bundle(
        &RenderProfileBundle::default_render(),
        &RenderCapabilitySummary::default(),
        &AdvancedProviderAvailability::new(),
    )
}
