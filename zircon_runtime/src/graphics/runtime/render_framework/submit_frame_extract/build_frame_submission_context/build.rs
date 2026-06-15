use crate::core::framework::render::{
    AntiAliasSettings, FrameHistoryInvalidationReason, PostProcessStackDescriptor,
    RenderBloomSettings, RenderCameraTargetResolutionReport, RenderColorGradingSettings,
    RenderFrameExtract, RenderFrameworkError, RenderHybridGiPayloadSource,
    RenderPostProcessEffectStackSettings, RenderViewportHandle, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPayloadSource,
};
use crate::graphics::runtime::FrameHistoryValidationKey;
use crate::graphics::ViewportRenderOutputTarget;
use zircon_runtime_interface::ui::surface::{UiRenderCommandKind, UiRenderExtract};

use crate::{VirtualGeometryRuntimeExtractOutput, VisibilityContext};

use super::super::super::compiled_feature_names::compiled_feature_names;
use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::frame_submission_context::{FrameSubmissionContext, UiSubmissionStats};
use super::compile_pipeline::{
    compile_submission_pipeline, compile_submission_pipeline_with_options,
};
use super::resolve_enabled_features::resolve_enabled_features;
use super::resolve_viewport_record_state::resolve_viewport_record_state;
use super::target_resolution::resolve_camera_target_size;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn build_frame_submission_context(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: &RenderFrameExtract,
    ui_extract: Option<&UiRenderExtract>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    let mut viewport_state = resolve_viewport_record_state(server, viewport, extract)?;
    let primary_target_size = viewport_state.size();
    let asset_manager = {
        let state = server.lock_state();
        state.renderer.asset_manager_for_runtime_extract()
    };
    let submission_size = resolve_camera_target_size(
        primary_target_size,
        &extract.view.camera.target,
        asset_manager.as_ref(),
    )?;
    let mut sized_extract = extract.clone();
    sized_extract.apply_viewport_size(submission_size);
    apply_renderer_owned_particle_previous_state(&mut sized_extract, &viewport_state);
    let extract = &sized_extract;
    let effective_view_size = extract.view.effective_view_size();
    let render_size = extract.view.effective_render_size();
    let camera_target_resolution = RenderCameraTargetResolutionReport::new(
        extract.view.camera.target.kind(),
        primary_target_size,
        submission_size,
        effective_view_size,
        render_size,
    );
    let output_target = ViewportRenderOutputTarget::from_camera_target(
        &extract.view.camera.target,
        submission_size,
    );
    let compiled_pipeline = compile_submission_pipeline(&viewport_state, extract)?;
    let advanced_runtime_plan = viewport_state.advanced_runtime_plan().clone();
    let solari_runtime_report = viewport_state.solari_runtime_report().clone();
    let (hybrid_gi_enabled, virtual_geometry_enabled) =
        resolve_enabled_features(&compiled_pipeline, &advanced_runtime_plan);
    let bloom_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(crate::BuiltinRenderFeature::Bloom));
    let color_grading_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(crate::BuiltinRenderFeature::ColorGrading));
    let temporal_history_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(crate::BuiltinRenderFeature::Temporal));
    let anti_alias_feature_enabled = compiled_pipeline
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(crate::BuiltinRenderFeature::AntiAlias));
    let resolved_post_process = extract
        .post_process
        .resolved_settings_for_camera(
            extract.view.camera.transform.translation,
            &extract.view.camera.render_layers,
        )
        .map_err(|error| {
            RenderFrameworkError::Backend(format!("post-process volume evaluation failed: {error}"))
        })?;
    let effective_bloom = bloom_enabled
        .then_some(resolved_post_process.bloom)
        .unwrap_or_else(RenderBloomSettings::default);
    let effective_exposure = resolved_post_process.exposure;
    let effective_color_grading = color_grading_enabled
        .then_some(resolved_post_process.color_grading)
        .unwrap_or_else(RenderColorGradingSettings::default);
    let effective_effect_stack = resolved_post_process.effect_stack;
    let authored_virtual_geometry_extract = apply_virtual_geometry_debug_override(
        extract.geometry.virtual_geometry.clone(),
        extract.geometry.virtual_geometry_debug,
    );
    let authored_virtual_geometry_present = authored_virtual_geometry_extract.is_some();
    let automatic_virtual_geometry_output =
        if virtual_geometry_enabled && !authored_virtual_geometry_present {
            build_automatic_virtual_geometry_extract(server, extract)
        } else {
            None
        };
    let virtual_geometry_payload_source = virtual_geometry_payload_source_for_extract(
        virtual_geometry_enabled,
        authored_virtual_geometry_present,
        automatic_virtual_geometry_output.is_some(),
    );
    let effective_virtual_geometry_extract = authored_virtual_geometry_extract.or_else(|| {
        automatic_virtual_geometry_output
            .as_ref()
            .map(|output| output.extract().clone())
    });
    let visibility_extract = visibility_extract_with_effective_advanced_features(
        extract,
        hybrid_gi_enabled,
        virtual_geometry_enabled
            .then(|| effective_virtual_geometry_extract.clone())
            .flatten(),
    );
    let effective_history_key_extract = post_process_extract_with_effective_settings(
        &visibility_extract,
        effective_bloom,
        effective_color_grading,
        effective_effect_stack,
    );
    let virtual_geometry_cpu_reference_instances = automatic_virtual_geometry_output
        .as_ref()
        .map(|output| output.cpu_reference_instances().to_vec())
        .unwrap_or_default();
    let virtual_geometry_bvh_visualization_instances = automatic_virtual_geometry_output
        .as_ref()
        .map(|output| output.bvh_visualization_instances().to_vec())
        .unwrap_or_default();
    let visibility_context = VisibilityContext::from_extract_with_history_and_static_index(
        &visibility_extract,
        viewport_state.previous_visibility(),
        viewport_state.previous_static_index(),
    );
    let history_validation_key = FrameHistoryValidationKey::from_extract(
        &effective_history_key_extract,
        compiled_feature_names(&compiled_pipeline),
    );
    let history_invalidation_reason = frame_history_invalidation_reason(
        server,
        viewport,
        submission_size,
        render_size,
        viewport_state.pipeline_handle(),
        &compiled_pipeline,
        &history_validation_key,
    );
    let history_available = temporal_history_enabled && history_invalidation_reason.is_none();
    let requested_anti_alias = if anti_alias_feature_enabled {
        extract.view.anti_alias.with_taa_quality(
            viewport_state
                .quality_profile_taa_quality()
                .unwrap_or(extract.view.anti_alias.taa_quality),
        )
    } else {
        AntiAliasSettings::off()
    };
    let taa_history_store_available =
        temporal_history_enabled && viewport_state.capabilities().supports_taa;
    let anti_alias_history_available = if requested_anti_alias.mode
        == crate::core::framework::render::AntiAliasMode::Taa
        && taa_history_store_available
    {
        true
    } else {
        history_available
    };
    let anti_alias_report =
        requested_anti_alias.resolve(viewport_state.capabilities(), anti_alias_history_available);
    let post_process_history_available = history_available
        || (anti_alias_report.effective_mode == crate::core::framework::render::AntiAliasMode::Taa
            && taa_history_store_available);
    let post_process_stack =
        PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
            &effective_bloom,
            &effective_color_grading,
            effective_exposure,
            &effective_effect_stack,
            temporal_history_enabled,
            post_process_history_available,
            &anti_alias_report.effective_settings(),
        );
    let compiled_pipeline = compile_submission_pipeline_with_options(
        &viewport_state,
        extract,
        &viewport_state
            .compile_options()
            .clone()
            .with_graph_msaa_sample_count(anti_alias_report.effective_graph_sample_count())
            .with_post_process_stack(post_process_stack.clone()),
    )?;
    let post_process_graph = post_process_stack.validated_graph();
    let hybrid_gi_update_plan =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_update_plan.clone());
    let hybrid_gi_feedback =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_feedback.clone());
    let hybrid_gi_payload_source = hybrid_gi_payload_source_for_extract(
        hybrid_gi_enabled,
        extract.lighting.hybrid_global_illumination.is_some(),
    );
    let virtual_geometry_page_upload_plan = virtual_geometry_enabled
        .then(|| visibility_context.virtual_geometry_page_upload_plan.clone());
    let virtual_geometry_feedback =
        virtual_geometry_enabled.then(|| visibility_context.virtual_geometry_feedback.clone());

    Ok(FrameSubmissionContext::new(
        submission_size,
        render_size,
        viewport_state.pipeline_handle(),
        viewport_state.viewport_generation(),
        viewport_state.take_quality_profile(),
        compiled_pipeline,
        viewport_state.capabilities().clone(),
        visibility_context,
        viewport_state.previous_motion_vector_camera().cloned(),
        history_validation_key,
        history_invalidation_reason,
        output_target,
        camera_target_resolution,
        extract.view.scene_camera_order_report.clone(),
        ui_extract
            .map(compute_ui_submission_stats)
            .unwrap_or_default(),
        effective_bloom,
        effective_exposure,
        effective_color_grading,
        effective_effect_stack,
        anti_alias_report,
        viewport_state.temporal_frame_index(),
        advanced_runtime_plan,
        solari_runtime_report,
        post_process_stack,
        post_process_graph,
        hybrid_gi_enabled,
        virtual_geometry_enabled,
        hybrid_gi_enabled
            .then(|| extract.lighting.hybrid_global_illumination.clone())
            .flatten(),
        hybrid_gi_payload_source,
        hybrid_gi_update_plan,
        hybrid_gi_feedback,
        extract.geometry.meshes.clone(),
        extract.lighting.directional_lights.clone(),
        extract.lighting.point_lights.clone(),
        extract.lighting.spot_lights.clone(),
        extract.lighting.ambient_lights.clone(),
        extract.lighting.rect_lights.clone(),
        extract.particles.previous_sprites.clone(),
        extract.particles.sprites.len(),
        extract.particles.previous_state_sprite_count(),
        extract.particles.anonymous_stream_ambiguity_sprite_count(),
        virtual_geometry_enabled
            .then(|| effective_virtual_geometry_extract.clone())
            .flatten(),
        virtual_geometry_payload_source,
        virtual_geometry_cpu_reference_instances,
        virtual_geometry_bvh_visualization_instances,
        virtual_geometry_page_upload_plan,
        virtual_geometry_feedback,
        viewport_state.predicted_generation(),
    ))
}

fn apply_renderer_owned_particle_previous_state(
    extract: &mut RenderFrameExtract,
    viewport_state: &super::viewport_record_state::ViewportRecordState,
) {
    if !extract.particles.previous_sprites.is_empty() {
        return;
    }
    extract.particles.previous_sprites = viewport_state.previous_particle_sprites().to_vec();
}

fn post_process_extract_with_effective_settings(
    extract: &RenderFrameExtract,
    bloom: RenderBloomSettings,
    color_grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
) -> RenderFrameExtract {
    let mut extract = extract.clone();
    extract.post_process.bloom = bloom;
    extract.post_process.color_grading = color_grading;
    extract.post_process.effect_stack = effect_stack;
    extract.post_process.volumes.clear();
    extract.post_process.rebuild_graph(false, false);
    extract
}

fn frame_history_invalidation_reason(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    target_size: crate::core::math::UVec2,
    render_size: crate::core::math::UVec2,
    pipeline_handle: crate::core::framework::render::RenderPipelineHandle,
    compiled_pipeline: &crate::CompiledRenderPipeline,
    history_validation_key: &FrameHistoryValidationKey,
) -> Option<FrameHistoryInvalidationReason> {
    let state = server.lock_state();
    let Some(history) = state
        .viewports
        .get(&viewport)
        .and_then(|record| record.history())
    else {
        return Some(FrameHistoryInvalidationReason::NoPreviousFrame);
    };
    history.incompatibility_reason(
        target_size,
        render_size,
        pipeline_handle,
        &compiled_pipeline.history_bindings,
        history_validation_key,
    )
}

fn apply_virtual_geometry_debug_override(
    extract: Option<RenderVirtualGeometryExtract>,
    debug_override: Option<crate::core::framework::render::RenderVirtualGeometryDebugState>,
) -> Option<RenderVirtualGeometryExtract> {
    let mut extract = extract?;
    if let Some(debug_override) = debug_override {
        extract.debug = debug_override;
    }
    Some(extract)
}

fn build_automatic_virtual_geometry_extract(
    server: &WgpuRenderFramework,
    extract: &RenderFrameExtract,
) -> Option<VirtualGeometryRuntimeExtractOutput> {
    let (registration, asset_manager) = {
        let state = server.lock_state();
        (
            state.virtual_geometry_runtime_provider.clone()?,
            state.renderer.asset_manager_for_runtime_extract(),
        )
    };
    let mut load_model = |model_id| asset_manager.load_model_asset(model_id).ok();
    registration.provider().build_extract_from_meshes(
        &extract.geometry.meshes,
        extract.geometry.virtual_geometry_debug,
        &mut load_model,
    )
}

fn hybrid_gi_payload_source_for_extract(
    hybrid_gi_enabled: bool,
    authored_hybrid_gi_present: bool,
) -> RenderHybridGiPayloadSource {
    if hybrid_gi_enabled && authored_hybrid_gi_present {
        RenderHybridGiPayloadSource::Authored
    } else {
        RenderHybridGiPayloadSource::None
    }
}

fn virtual_geometry_payload_source_for_extract(
    virtual_geometry_enabled: bool,
    authored_virtual_geometry_present: bool,
    automatic_virtual_geometry_present: bool,
) -> RenderVirtualGeometryPayloadSource {
    if !virtual_geometry_enabled {
        return RenderVirtualGeometryPayloadSource::None;
    }
    if authored_virtual_geometry_present {
        return RenderVirtualGeometryPayloadSource::Authored;
    }
    if automatic_virtual_geometry_present {
        return RenderVirtualGeometryPayloadSource::AutomaticFallback;
    }
    RenderVirtualGeometryPayloadSource::None
}

fn visibility_extract_with_effective_advanced_features(
    extract: &RenderFrameExtract,
    hybrid_gi_enabled: bool,
    virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
) -> RenderFrameExtract {
    let mut extract = extract.clone();
    if !hybrid_gi_enabled {
        extract.lighting.hybrid_global_illumination = None;
    }
    extract.geometry.virtual_geometry = virtual_geometry_extract;
    extract
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_geometry_payload_source_prefers_authored_extract() {
        let source = virtual_geometry_payload_source_for_extract(true, true, true);

        assert_eq!(source, RenderVirtualGeometryPayloadSource::Authored);
    }

    #[test]
    fn virtual_geometry_payload_source_reports_automatic_fallback() {
        let source = virtual_geometry_payload_source_for_extract(true, false, true);

        assert_eq!(
            source,
            RenderVirtualGeometryPayloadSource::AutomaticFallback
        );
    }

    #[test]
    fn virtual_geometry_payload_source_clears_when_feature_disabled_or_missing() {
        assert_eq!(
            virtual_geometry_payload_source_for_extract(false, true, true),
            RenderVirtualGeometryPayloadSource::None
        );
        assert_eq!(
            virtual_geometry_payload_source_for_extract(true, false, false),
            RenderVirtualGeometryPayloadSource::None
        );
    }

    #[test]
    fn hybrid_gi_payload_source_reports_authored_extract_only_when_enabled() {
        assert_eq!(
            hybrid_gi_payload_source_for_extract(true, true),
            RenderHybridGiPayloadSource::Authored
        );
        assert_eq!(
            hybrid_gi_payload_source_for_extract(false, true),
            RenderHybridGiPayloadSource::None
        );
        assert_eq!(
            hybrid_gi_payload_source_for_extract(true, false),
            RenderHybridGiPayloadSource::None
        );
    }
}

fn compute_ui_submission_stats(extract: &UiRenderExtract) -> UiSubmissionStats {
    let mut stats = UiSubmissionStats::default();
    for command in &extract.list.commands {
        stats.record_command();
        if matches!(command.kind, UiRenderCommandKind::Quad) {
            stats.record_quad();
        }
        if command.text.is_some() {
            stats.record_text_payload();
        }
        if command.image.is_some() {
            stats.record_image_payload();
        }
        if command.clip_frame.is_some() {
            stats.record_clipped_command();
        }
    }
    stats
}
