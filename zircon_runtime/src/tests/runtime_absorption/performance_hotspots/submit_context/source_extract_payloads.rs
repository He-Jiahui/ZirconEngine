use super::sources::SubmitContextSources;

pub(super) fn assert_source_extract_payloads_are_shared(sources: &SubmitContextSources) {
    let context = sources.context;
    let build_context = sources.build_context;
    let build_runtime_frame = sources.build_runtime_frame;
    let submit_extract = sources.submit_extract;
    let present_frame_extract = sources.present_frame_extract;
    let submit_runtime_frame = sources.submit_runtime_frame;
    let viewport_render_frame = sources.viewport_render_frame;
    let viewport_render_frame_from_extract = sources.viewport_render_frame_from_extract;

    assert!(
        context.contains("submission_extract: Arc<RenderFrameExtract>"),
        "FrameSubmissionContext should own one renderer-resolved overlay sharing the scene payload"
    );
    assert!(
        viewport_render_frame.contains("pub extract: Arc<RenderFrameExtract>"),
        "ViewportRenderFrame should store RenderFrameExtract as an Arc for shared submit sources"
    );

    for forbidden_owned_payload in [
        "scene_meshes: Vec<RenderMeshSnapshot>",
        "scene_directional_lights: Vec<RenderDirectionalLightSnapshot>",
        "scene_point_lights: Vec<RenderPointLightSnapshot>",
        "scene_spot_lights: Vec<RenderSpotLightSnapshot>",
        "scene_ambient_lights: Vec<RenderAmbientLightSnapshot>",
        "scene_rect_lights: Vec<RenderRectLightSnapshot>",
        "particle_previous_sprites: Vec<RenderParticlePreviousSpriteSnapshot>",
    ] {
        assert!(
            !context.contains(forbidden_owned_payload),
            "FrameSubmissionContext should not reclaim cloned payload field `{forbidden_owned_payload}`"
        );
    }

    for forbidden_build_clone in [
        "extract.geometry.meshes.clone(),",
        "extract.lighting.directional_lights.clone(),",
        "extract.lighting.point_lights.clone(),",
        "extract.lighting.spot_lights.clone(),",
        "extract.lighting.ambient_lights.clone(),",
        "extract.lighting.rect_lights.clone(),",
        "extract.particles.previous_sprites.clone(),",
    ] {
        assert!(
            !build_context.contains(forbidden_build_clone),
            "build_frame_submission_context should not clone large extract payload `{forbidden_build_clone}` into FrameSubmissionContext::new"
        );
    }

    for forbidden_effective_clone_helper in [
        "let mut sized_extract = extract.clone();",
        "fn post_process_extract_with_effective_settings",
        "fn visibility_extract_with_effective_advanced_features",
        "Arc::try_unwrap(extract).unwrap_or_else(|extract| (*extract).clone())",
    ] {
        assert!(
            !build_context.contains(forbidden_effective_clone_helper),
            "build_frame_submission_context should not restore full-extract clone helper `{forbidden_effective_clone_helper}`"
        );
    }
    assert!(
        !build_runtime_frame
            .contains("Arc::try_unwrap(extract).unwrap_or_else(|extract| (*extract).clone())"),
        "build_runtime_frame should not clone the shared extract to append virtual-geometry debug overlays"
    );

    for required_anchor in [
        "build_frame_submission_context_from_runtime_frame_extract(",
        "fn build_frame_submission_context_from_source(",
        "source_extract: &Arc<RenderFrameExtract>",
        "let mut submission_extract = source_extract.as_ref().clone();",
        "let submission_extract = Arc::new(submission_extract);",
        "for_camera_submission",
        "runtime_virtual_geometry_debug_overlays(",
        "source_overlays: &RenderOverlayExtract",
        "with_runtime_overlays(runtime_overlays)",
        "runtime_overlay_override: Option<RenderOverlayExtract>",
        ".unwrap_or(&self.extract.debug.overlays)",
        "pub(super) fn submission_extract(&self) -> Arc<RenderFrameExtract>",
        "&self.submission_extract.geometry.meshes",
        "&self.submission_extract.lighting.directional_lights",
        "VisibilityContext::from_extract_with_history_static_index_task_pool_and_feature_payloads",
        "FrameHistoryValidationKey::from_extract",
        "build_renderer_owned_post_process_snapshot(",
        "post_process: Arc<RendererPostProcessSnapshot>",
        "with_post_process_override(context.post_process_shared())",
    ] {
        assert!(
            context.contains(required_anchor)
                || build_context.contains(required_anchor)
                || build_runtime_frame.contains(required_anchor)
                || viewport_render_frame.contains(required_anchor),
            "Runtime 07 F3 source-extract sharing should retain anchor `{required_anchor}`"
        );
    }
    for required_shared_frame_anchor in [
        "pub(crate) fn from_shared_extract(",
        "ViewportRenderFrame::from_shared_extract(extract, context.size())",
        "let extract = context.submission_extract();",
        "build_frame_submission_context_from_runtime_frame_extract(",
        "extract: &Arc<RenderFrameExtract>",
        "build_runtime_frame(ui, &mut context, prepared, output_policy)",
        "frame.extract = context.submission_extract();",
    ] {
        assert!(
            viewport_render_frame_from_extract.contains(required_shared_frame_anchor)
                || build_runtime_frame.contains(required_shared_frame_anchor)
                || submit_extract.contains(required_shared_frame_anchor)
                || present_frame_extract.contains(required_shared_frame_anchor)
                || submit_runtime_frame.contains(required_shared_frame_anchor),
            "Runtime 07 F3 shared effective frame source should retain anchor `{required_shared_frame_anchor}`"
        );
    }
    for forbidden_mutable_scene_lane in [
        "FrameSubmissionSourcePayloads",
        "Arc::make_mut(extract_source)",
        "pub(crate) fn extract_mut(&mut self) -> &mut RenderFrameExtract",
    ] {
        assert!(
            !build_context.contains(forbidden_mutable_scene_lane)
                && !viewport_render_frame.contains(forbidden_mutable_scene_lane),
            "submission architecture must not restore mutable scene lane `{forbidden_mutable_scene_lane}`"
        );
    }
}
