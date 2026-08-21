use crate::core::framework::render::{
    AntiAliasSettings, FrameHistoryInvalidationReason, PostProcessEffectKind, PostProcessPassGraph,
    PostProcessStackDescriptor, RenderBloomSettings, RenderCameraTargetResolutionReport,
    RenderColorGradingSettings, RenderDynamicResolutionSettings, RenderFrameExtract,
    RenderFrameworkError, RenderHybridGiExtract, RenderHybridGiPayloadSource, RenderPipelinePhase,
    RenderPostProcessEffectStackSettings, RenderResolutionPolicy, RenderUpscalerKind,
    RenderViewExtract, RenderViewFamilyPipeline, RenderViewportHandle, RenderViewportRect,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPayloadSource,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::FrameHistoryValidationKey;
use crate::graphics::{
    BuiltinRenderFeature, RenderPipelineCompileOptions, ViewportRenderOutputTarget,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::surface::{UiRenderCommandKind, UiRenderExtract};

use crate::graphics::{VirtualGeometryRuntimeExtractOutput, VisibilityContext};

use super::super::super::budget::BudgetDegradeSettings;
use super::super::super::compiled_feature_names::compiled_feature_names;
use super::super::super::wgpu_render_framework::WgpuRenderFrameworkAccess;
use super::super::frame_submission_context::{
    temporal_jitter_for_submission, FrameSubmissionContext, UiSubmissionStats,
};
use super::camera_history_key::camera_history_key_for_extract;
use super::compile_pipeline::compile_submission_pipeline_with_options;
use super::environment_ibl_compile_options::{
    compile_options_with_environment_ibl_bake_request, resolve_and_rehydrate_environment_ibl_cache,
    EnvironmentIblCacheResolution,
};
use super::material_feature_extract::resolve_advanced_pbr_material_usage;
use super::resolve_enabled_features::resolve_enabled_features;
use super::resolve_viewport_record_state::resolve_viewport_record_state;
use super::subsurface_profile_extract::resolve_subsurface_material_profiles;
use super::target_resolution::resolve_camera_target_descriptor;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn build_frame_submission_context_from_runtime_frame_extract(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    extract: &mut Arc<RenderFrameExtract>,
    ui_extract: Option<&UiRenderExtract>,
    source_payloads: Option<FrameSubmissionSourcePayloads<'_>>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    build_frame_submission_context_from_source(
        framework,
        viewport,
        extract,
        ui_extract,
        source_payloads,
    )
}

#[derive(Clone, Copy)]
pub(in crate::graphics::runtime::render_framework::submit_frame_extract) struct FrameSubmissionSourcePayloads<
    'a,
> {
    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) virtual_geometry:
        Option<&'a RenderVirtualGeometryExtract>,
    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) hybrid_global_illumination:
        Option<&'a RenderHybridGiExtract>,
}

impl<'a> FrameSubmissionSourcePayloads<'a> {
    pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn from_extract(
        extract: &'a RenderFrameExtract,
    ) -> Self {
        Self {
            virtual_geometry: extract.geometry.virtual_geometry.as_ref(),
            hybrid_global_illumination: extract.lighting.hybrid_global_illumination.as_ref(),
        }
    }
}

fn build_frame_submission_context_from_source(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    extract_source: &mut Arc<RenderFrameExtract>,
    ui_extract: Option<&UiRenderExtract>,
    source_payloads: Option<FrameSubmissionSourcePayloads<'_>>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    let budget_degrade_settings = {
        let state = framework.lock_state();
        state.degrade_ladder.settings()
    };
    apply_budget_render_scale(Arc::make_mut(extract_source), budget_degrade_settings);
    let mut viewport_state =
        resolve_viewport_record_state(framework, viewport, extract_source.as_ref())?;
    let primary_target_size = viewport_state.size();
    let (asset_manager, environment_ibl_hydration_cache) = {
        let state = framework.lock_state();
        (
            state.renderer.asset_manager_for_runtime_extract(),
            Arc::clone(&state.environment_ibl_hydration_cache),
        )
    };
    let asset_manager =
        asset_manager.map_err(|error| RenderFrameworkError::Backend(error.to_string()))?;
    resolve_subsurface_material_profiles(asset_manager.as_ref(), Arc::make_mut(extract_source));
    resolve_advanced_pbr_material_usage(asset_manager.as_ref(), Arc::make_mut(extract_source));
    let environment_ibl_cache_resolution = resolve_and_rehydrate_environment_ibl_cache(
        asset_manager.as_ref(),
        &environment_ibl_hydration_cache,
        Arc::make_mut(extract_source),
    )?;
    let resolved_camera_target = resolve_camera_target_descriptor(
        primary_target_size,
        extract_source.as_ref().view.selected_camera_target(),
        asset_manager.as_ref(),
    )?;
    let submission_size = resolved_camera_target.size();
    let compile_camera_target = resolved_camera_target.compile_fingerprint();
    {
        let sized_extract = Arc::make_mut(extract_source);
        sized_extract.apply_viewport_size(submission_size);
        apply_renderer_owned_particle_previous_state(sized_extract, &mut viewport_state);
    }
    let sized_extract = extract_source.as_ref();
    let camera_history_key = camera_history_key_for_extract(sized_extract);
    let effective_view_size = sized_extract.view.effective_view_size();
    let render_size = sized_extract.view.effective_render_size();
    let camera_target_resolution = RenderCameraTargetResolutionReport::new(
        sized_extract.view.selected_camera_target().kind(),
        primary_target_size,
        submission_size,
        effective_view_size,
        render_size,
    );
    let output_target = ViewportRenderOutputTarget::from_camera_target(
        sized_extract.view.selected_camera_target(),
        submission_size,
        resolved_camera_target.texture_format(),
    );
    let budget_compile_options = compile_options_for_budget_degrade(
        viewport_state.compile_options().clone(),
        budget_degrade_settings,
    );
    let compiled_pipeline = compile_submission_pipeline_with_options(
        framework,
        &viewport_state,
        sized_extract,
        compile_camera_target,
        &budget_compile_options,
    )?;
    let advanced_runtime_plan = viewport_state.take_advanced_runtime_plan();
    let solari_runtime_report = viewport_state.take_solari_runtime_report();
    let (hybrid_gi_enabled, virtual_geometry_enabled) =
        resolve_enabled_features(&compiled_pipeline, &advanced_runtime_plan);
    let runtime_features = compiled_pipeline.runtime_feature_flags();
    let bloom_enabled = runtime_features.bloom_enabled;
    let color_grading_enabled = runtime_features.color_grading_enabled;
    let temporal_history_enabled = runtime_features.temporal_history_enabled;
    let anti_alias_feature_enabled = runtime_features.anti_alias_enabled
        || runtime_features.screen_space_anti_alias_capability_enabled;
    let resolved_post_process = sized_extract
        .post_process
        .resolved_settings_for_camera(
            sized_extract.view.camera.transform.translation,
            sized_extract.view.selected_camera_volume_layers(),
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
    let effective_effect_stack = effect_stack_for_budget_degrade(
        resolved_post_process.effect_stack,
        budget_degrade_settings,
    );
    let source_payloads = source_payloads
        .unwrap_or_else(|| FrameSubmissionSourcePayloads::from_extract(sized_extract));
    let source_virtual_geometry = source_payloads.virtual_geometry;
    let source_hybrid_gi = source_payloads.hybrid_global_illumination;
    let authored_virtual_geometry_extract = apply_virtual_geometry_debug_override(
        source_virtual_geometry.cloned(),
        sized_extract.geometry.virtual_geometry_debug,
    );
    let authored_virtual_geometry_present = authored_virtual_geometry_extract.is_some();
    let automatic_virtual_geometry_output =
        if virtual_geometry_enabled && !authored_virtual_geometry_present {
            build_automatic_virtual_geometry_extract(framework, sized_extract)
        } else {
            None
        };
    let virtual_geometry_payload_source = virtual_geometry_payload_source_for_extract(
        virtual_geometry_enabled,
        authored_virtual_geometry_present,
        automatic_virtual_geometry_output.is_some(),
    );
    let (
        automatic_virtual_geometry_extract,
        virtual_geometry_cpu_reference_instances,
        virtual_geometry_bvh_visualization_instances,
        virtual_geometry_resident_page_payloads,
    ) = automatic_virtual_geometry_output
        .map(VirtualGeometryRuntimeExtractOutput::into_parts)
        .map(
            |(extract, cpu_references, bvh_instances, resident_payloads)| {
                (
                    Some(extract),
                    cpu_references,
                    bvh_instances,
                    resident_payloads,
                )
            },
        )
        .unwrap_or_default();
    let effective_virtual_geometry_extract =
        authored_virtual_geometry_extract.or(automatic_virtual_geometry_extract);
    let hybrid_gi_settings_present = sized_extract.lighting.hybrid_global_illumination.is_some();
    let hybrid_gi_settings_present = hybrid_gi_settings_present || source_hybrid_gi.is_some();
    let effective_hybrid_gi_extract = hybrid_gi_enabled
        .then(|| source_hybrid_gi.cloned())
        .flatten();
    let source_anti_alias = sized_extract.view.anti_alias;
    let source_msaa_samples = sized_extract.view.camera.msaa_samples;
    let effective_extract = Arc::make_mut(extract_source);
    apply_effective_post_process_settings(
        effective_extract,
        effective_bloom,
        effective_color_grading,
        effective_effect_stack,
    );
    let effective_extract = extract_source.as_ref();
    let visibility_context =
        VisibilityContext::from_extract_with_history_static_index_task_pool_and_feature_payloads(
            effective_extract,
            viewport_state.previous_visibility(),
            viewport_state.previous_static_index(),
            viewport_state.previous_dynamic_index(),
            Some(framework.compute_task_pool()),
            effective_hybrid_gi_extract.as_ref(),
            virtual_geometry_enabled
                .then_some(effective_virtual_geometry_extract.as_ref())
                .flatten(),
        );
    let history_validation_key = FrameHistoryValidationKey::from_extract_with_hybrid_gi(
        effective_extract,
        compiled_feature_names(&compiled_pipeline),
        effective_hybrid_gi_extract.as_ref(),
    );
    let history_invalidation_reason = frame_history_invalidation_reason(
        framework,
        viewport,
        submission_size,
        render_size,
        viewport_state.pipeline_handle(),
        &compiled_pipeline,
        &camera_history_key,
        &history_validation_key,
    );
    let history_available = temporal_history_enabled && history_invalidation_reason.is_none();
    let requested_anti_alias = if anti_alias_feature_enabled {
        source_anti_alias.with_taa_quality(
            viewport_state
                .quality_profile_taa_quality()
                .unwrap_or(source_anti_alias.taa_quality),
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
    let anti_alias_report = requested_anti_alias.resolve_with_requested_graph_sample_count(
        viewport_state.capabilities(),
        anti_alias_history_available,
        source_msaa_samples,
    );
    let post_process_history_available = history_available
        || (anti_alias_report.effective_mode == crate::core::framework::render::AntiAliasMode::Taa
            && taa_history_store_available);
    let view_family_pipeline = resolve_view_family_pipeline_for_submission(
        &effective_extract.view,
        submission_size,
        if anti_alias_report.effective_mode == crate::core::framework::render::AntiAliasMode::Taa {
            RenderUpscalerKind::Temporal
        } else {
            RenderUpscalerKind::Spatial
        },
    );
    let upscale_required = view_family_pipeline
        .phases()
        .contains(&RenderPipelinePhase::SpatialUpscale);
    let mut post_process_stack =
        PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
            &effective_bloom,
            &effective_color_grading,
            effective_exposure,
            &effective_effect_stack,
            temporal_history_enabled,
            post_process_history_available,
            &anti_alias_report.effective_settings(),
            upscale_required,
        );
    if hybrid_gi_enabled && effective_hybrid_gi_extract.is_some() {
        post_process_stack = post_process_stack.with_hybrid_gi_lighting_input();
    }
    let post_process_graph = PostProcessPassGraph::validate_stack_for_view_family(
        &post_process_stack,
        &view_family_pipeline,
    )
    .map_err(|error| {
        RenderFrameworkError::Backend(format!(
            "view-family post-process graph validation failed: {error}"
        ))
    })?;
    let final_budget_compile_options = compile_options_for_budget_degrade(
        budget_compile_options
            .clone()
            .with_graph_msaa_sample_count(anti_alias_report.effective_graph_sample_count())
            .with_post_process_stack(post_process_stack.clone()),
        budget_degrade_settings,
    );
    let compile_options = compile_options_with_environment_ibl_bake_request(
        effective_extract,
        final_budget_compile_options,
        environment_ibl_cache_resolution.as_ref(),
    )?;
    let compiled_pipeline = compile_submission_pipeline_with_options(
        framework,
        &viewport_state,
        effective_extract,
        compile_camera_target,
        &compile_options,
    )?;
    let temporal_jitter =
        temporal_jitter_for_submission(anti_alias_report, viewport_state.temporal_frame_index());
    {
        let effective_extract = Arc::make_mut(extract_source);
        apply_effective_view_and_graph_settings(
            effective_extract,
            anti_alias_report,
            temporal_jitter,
            post_process_stack.clone(),
            post_process_graph.clone(),
        );
    }
    let effective_extract = extract_source.as_ref();
    let hybrid_gi_update_plan =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_update_plan.clone());
    let hybrid_gi_feedback =
        hybrid_gi_enabled.then(|| visibility_context.hybrid_gi_feedback.clone());
    let hybrid_gi_payload_source =
        hybrid_gi_payload_source_for_frame(hybrid_gi_enabled, hybrid_gi_settings_present);
    let virtual_geometry_page_upload_plan = virtual_geometry_enabled
        .then(|| visibility_context.virtual_geometry_page_upload_plan.clone());
    let virtual_geometry_feedback =
        virtual_geometry_enabled.then(|| visibility_context.virtual_geometry_feedback.clone());
    let particle_sprite_count = effective_extract.particles.sprites.len();
    let particle_previous_state_sprite_count =
        effective_extract.particles.previous_state_sprite_count();
    let particle_anonymous_stream_ambiguity_sprite_count = effective_extract
        .particles
        .anonymous_stream_ambiguity_sprite_count();
    let scene_camera_order_report = effective_extract.view.scene_camera_order_report.clone();
    let hybrid_gi_extract_for_context = effective_hybrid_gi_extract;
    let virtual_geometry_extract_for_context = virtual_geometry_enabled
        .then_some(effective_virtual_geometry_extract)
        .flatten();
    let source_extract = Arc::clone(extract_source);
    let quality_profile_texture_mip_bias = viewport_state.quality_profile_texture_mip_bias();
    let quality_profile_texture_max_anisotropy =
        viewport_state.quality_profile_texture_max_anisotropy();
    let quality_profile = viewport_state.take_quality_profile();
    let capabilities = viewport_state.take_capabilities();
    let previous_motion_vector_camera = viewport_state.take_previous_motion_vector_camera();

    let environment_ibl_bake_reservation = environment_ibl_cache_resolution
        .and_then(EnvironmentIblCacheResolution::take_runtime_bake_reservation);

    Ok(FrameSubmissionContext::new(
        submission_size,
        render_size,
        viewport_state.pipeline_handle(),
        viewport_state.viewport_generation(),
        quality_profile,
        viewport_state.shader_quality(),
        compiled_pipeline,
        capabilities,
        visibility_context,
        previous_motion_vector_camera,
        camera_history_key,
        history_validation_key,
        temporal_history_enabled,
        history_invalidation_reason,
        output_target,
        camera_target_resolution,
        scene_camera_order_report,
        ui_extract
            .map(compute_ui_submission_stats)
            .unwrap_or_default(),
        effective_effect_stack,
        anti_alias_report,
        advanced_runtime_plan,
        solari_runtime_report,
        post_process_graph,
        hybrid_gi_enabled,
        virtual_geometry_enabled,
        hybrid_gi_extract_for_context,
        hybrid_gi_payload_source,
        hybrid_gi_update_plan,
        hybrid_gi_feedback,
        source_extract,
        particle_sprite_count,
        particle_previous_state_sprite_count,
        particle_anonymous_stream_ambiguity_sprite_count,
        virtual_geometry_extract_for_context,
        virtual_geometry_payload_source,
        virtual_geometry_cpu_reference_instances,
        virtual_geometry_bvh_visualization_instances,
        virtual_geometry_resident_page_payloads,
        virtual_geometry_page_upload_plan,
        virtual_geometry_feedback,
        viewport_state.predicted_generation(),
    )
    .with_global_material_mip_bias(
        f32::from(quality_profile_texture_mip_bias)
            + budget_degrade_settings.global_mip_bias as f32,
    )
    .with_texture_max_anisotropy(quality_profile_texture_max_anisotropy)
    .with_environment_ibl_bake_reservation(environment_ibl_bake_reservation))
}

fn apply_budget_render_scale(extract: &mut RenderFrameExtract, settings: BudgetDegradeSettings) {
    let authored_scale = extract.view.camera.dynamic_resolution.clamped_scale();
    let effective_scale = authored_scale.min(settings.render_scale);
    extract.view.camera.dynamic_resolution = if effective_scale < 1.0 {
        RenderDynamicResolutionSettings::fixed_scale(effective_scale)
    } else {
        RenderDynamicResolutionSettings::disabled()
    };
}

fn compile_options_for_budget_degrade(
    mut options: RenderPipelineCompileOptions,
    settings: BudgetDegradeSettings,
) -> RenderPipelineCompileOptions {
    if settings.disable_ssao {
        options = options
            .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
            .with_plugin_feature_disabled("screen_space_ambient_occlusion");
    }
    if settings.disable_contact_shadow {
        options = options.with_plugin_feature_disabled("contact_shadow");
    }
    if settings.disable_bloom_high {
        options = options
            .with_feature_disabled(BuiltinRenderFeature::Bloom)
            .with_post_process_effect_disabled(PostProcessEffectKind::Bloom);
    }
    options
}

fn effect_stack_for_budget_degrade(
    mut effect_stack: RenderPostProcessEffectStackSettings,
    settings: BudgetDegradeSettings,
) -> RenderPostProcessEffectStackSettings {
    effect_stack.screen_space_reflection.roughness_mip_bias += settings.global_mip_bias as f32;
    if settings.disable_ssr {
        effect_stack.screen_space_reflection.intensity = 0.0;
    }
    effect_stack
}

fn resolve_view_family_pipeline_for_submission(
    view: &RenderViewExtract,
    submission_size: UVec2,
    upscaler: RenderUpscalerKind,
) -> RenderViewFamilyPipeline {
    let view_camera = view.selected_effective_camera();
    let display_viewport = view
        .selected_camera_descriptor()
        .and_then(|descriptor| descriptor.viewport_rect)
        .map(|viewport| viewport.clamped_to_size(submission_size))
        .unwrap_or_else(|| RenderViewportRect::new(UVec2::ZERO, submission_size));
    RenderViewFamilyPipeline::resolve_for_viewport(
        submission_size,
        display_viewport,
        RenderResolutionPolicy::with_spatial_primary_fraction(
            view_camera.dynamic_resolution.clamped_scale(),
        ),
        upscaler,
    )
}

fn apply_renderer_owned_particle_previous_state(
    extract: &mut RenderFrameExtract,
    viewport_state: &mut super::viewport_record_state::ViewportRecordState,
) {
    if !extract.particles.previous_sprites.is_empty() {
        return;
    }
    extract.particles.previous_sprites = viewport_state.take_previous_particle_sprites();
}

fn apply_effective_post_process_settings(
    extract: &mut RenderFrameExtract,
    bloom: RenderBloomSettings,
    color_grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
) {
    extract.post_process.bloom = bloom;
    extract.post_process.color_grading = color_grading;
    extract.post_process.effect_stack = effect_stack;
    extract.post_process.volumes.clear();
    extract.post_process.rebuild_graph(false, false);
}

fn frame_history_invalidation_reason(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    target_size: crate::core::math::UVec2,
    render_size: crate::core::math::UVec2,
    pipeline_handle: crate::core::framework::render::RenderPipelineHandle,
    compiled_pipeline: &crate::graphics::CompiledRenderPipeline,
    camera_history_key: &super::super::super::viewport_record::ViewportCameraHistoryKey,
    history_validation_key: &FrameHistoryValidationKey,
) -> Option<FrameHistoryInvalidationReason> {
    let state = framework.lock_state();
    let Some(history) = state
        .viewports
        .get(&viewport)
        .and_then(|record| record.history(camera_history_key))
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
    framework: &dyn WgpuRenderFrameworkAccess,
    extract: &RenderFrameExtract,
) -> Option<VirtualGeometryRuntimeExtractOutput> {
    let (provider, asset_manager) = {
        let state = framework.lock_state();
        (
            state
                .virtual_geometry_runtime_provider
                .as_ref()?
                .provider_arc(),
            state.renderer.asset_manager_for_runtime_extract().ok()?,
        )
    };
    let mut load_model = |model_id| asset_manager.load_model_asset(model_id).ok();
    provider.build_extract_from_meshes(
        &extract.geometry.meshes,
        extract.geometry.virtual_geometry_debug,
        &mut load_model,
    )
}

fn hybrid_gi_payload_source_for_frame(
    hybrid_gi_enabled: bool,
    hybrid_gi_settings_present: bool,
) -> RenderHybridGiPayloadSource {
    if hybrid_gi_enabled && hybrid_gi_settings_present {
        RenderHybridGiPayloadSource::SceneRepresentation
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

fn apply_effective_view_and_graph_settings(
    extract: &mut RenderFrameExtract,
    anti_alias_report: crate::core::framework::render::AntiAliasFallbackReport,
    temporal_jitter: crate::core::framework::render::TemporalJitterSample,
    post_process_stack: PostProcessStackDescriptor,
    post_process_graph: crate::core::framework::render::PostProcessPassGraph,
) {
    extract.view.anti_alias = anti_alias_report.effective_settings();
    extract.view.camera.temporal_jitter = temporal_jitter;
    extract.view.sync_selected_descriptor_camera_payload();
    extract.post_process.stack = post_process_stack;
    extract.post_process.graph = post_process_graph;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        CameraRenderDescriptor, PostProcessGraphResourceNames, RenderTonemapOperator,
        RenderTonemapSettings, ViewportCameraSnapshot,
    };

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
    fn hybrid_gi_payload_source_reports_scene_representation_only_when_enabled() {
        assert_eq!(
            hybrid_gi_payload_source_for_frame(true, true),
            RenderHybridGiPayloadSource::SceneRepresentation
        );
        assert_eq!(
            hybrid_gi_payload_source_for_frame(false, true),
            RenderHybridGiPayloadSource::None
        );
        assert_eq!(
            hybrid_gi_payload_source_for_frame(true, false),
            RenderHybridGiPayloadSource::None
        );
    }

    #[test]
    fn bloom_budget_degrade_synchronizes_the_feature_gate_and_post_process_stack() {
        let rebuilt_stack =
            PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
                &RenderBloomSettings {
                    intensity: 0.6,
                    ..Default::default()
                },
                &RenderColorGradingSettings::default(),
                &RenderPostProcessEffectStackSettings {
                    tonemap: RenderTonemapSettings {
                        operator: RenderTonemapOperator::Aces,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                false,
                false,
                &AntiAliasSettings::off(),
            );
        let settings = BudgetDegradeSettings {
            disable_bloom_high: true,
            ..Default::default()
        };
        let initial_options =
            compile_options_for_budget_degrade(RenderPipelineCompileOptions::default(), settings);
        let options = compile_options_for_budget_degrade(
            initial_options.with_post_process_stack(rebuilt_stack),
            settings,
        );

        assert!(options
            .disabled_features
            .contains(&BuiltinRenderFeature::Bloom));
        let stack = options
            .post_process_stack
            .expect("budget degradation should preserve the remaining post-process stack");
        assert!(stack
            .effects
            .iter()
            .any(|effect| { effect.kind == PostProcessEffectKind::Bloom && !effect.enabled }));
        assert!(stack.effects.iter().all(|effect| {
            !effect
                .required_inputs
                .iter()
                .any(|resource| resource == PostProcessGraphResourceNames::BLOOM)
                && !effect.after.contains(&PostProcessEffectKind::Bloom)
        }));
    }

    #[test]
    fn context_build_moves_owned_viewport_and_virtual_geometry_payloads() {
        let source = include_str!("build.rs");

        assert!(source.contains("take_previous_particle_sprites()"));
        assert!(source.contains("VirtualGeometryRuntimeExtractOutput::into_parts"));
        assert!(!source.contains(concat!("previous_particle_sprites()", ".to_vec()")));
        assert!(!source.contains(concat!("virtual_geometry_runtime_provider", ".clone()")));
    }

    #[test]
    fn post_process_mutation_reborrows_before_building_the_view_family() {
        let source = include_str!("build.rs");
        let mutation_start = source
            .find("let effective_extract = Arc::make_mut(extract_source);")
            .expect("post-process settings must mutate the frame extract once");
        let view_family_start = source[mutation_start..]
            .find("let view_family_pipeline =")
            .map(|offset| mutation_start + offset)
            .expect("view-family resolution must follow post-process mutation");
        let view_family_end = source[view_family_start..]
            .find("\n    );")
            .map(|offset| view_family_start + offset)
            .expect("view-family resolution must be a bounded call");
        let view_family = &source[view_family_start..view_family_end];

        assert!(view_family.contains("&effective_extract.view,"));
        assert!(!view_family.contains("sized_extract"));
    }

    #[test]
    fn submission_view_family_preserves_the_selected_camera_viewport_rect() {
        let camera = ViewportCameraSnapshot::default();
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(None, camera.clone());
        descriptor.viewport_rect = Some(RenderViewportRect::new(
            UVec2::new(960, 0),
            UVec2::new(960, 1080),
        ));
        let view =
            RenderViewExtract::from_camera(camera).with_selected_camera_descriptor(descriptor);

        let pipeline = resolve_view_family_pipeline_for_submission(
            &view,
            UVec2::new(1920, 1080),
            RenderUpscalerKind::Spatial,
        );

        assert_eq!(
            pipeline.resolution().display_viewport(),
            RenderViewportRect::new(UVec2::new(960, 0), UVec2::new(960, 1080))
        );
        assert_eq!(
            pipeline.resolution().primary_viewport(),
            RenderViewportRect::new(UVec2::new(960, 0), UVec2::new(960, 1080))
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
