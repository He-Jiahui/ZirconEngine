mod budget_degrade;
mod effective_view_state;
#[cfg(test)]
mod tests;
mod ui_submission_stats;

use crate::core::framework::render::{
    AntiAliasSettings, PostProcessPassGraph, PostProcessStackDescriptor, RenderBloomSettings,
    RenderCameraTargetResolutionReport, RenderColorGradingSettings, RenderFrameExtract,
    RenderFrameworkError, RenderHybridGiExtract, RenderHybridGiPayloadSource, RenderPipelinePhase,
    RenderUpscalerKind, RenderViewportHandle, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPayloadSource, UiRenderSubmission,
};
use crate::graphics::pipeline::AdvancedLightingCompileInputs;
use crate::graphics::runtime::FrameHistoryValidationKey;
use crate::graphics::{RendererPostProcessSnapshot, ViewportRenderOutputTarget};
use std::sync::Arc;

use crate::graphics::{VirtualGeometryRuntimeExtractOutput, VisibilityContext};

use super::super::super::budget::BudgetDegradeSettings;
use super::super::super::compiled_feature_names::compiled_feature_names;
use super::super::super::wgpu_render_framework::WgpuRenderFrameworkAccess;
use super::super::frame_submission_context::{
    temporal_jitter_for_submission, FrameSubmissionContext,
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
use budget_degrade::{
    apply_budget_render_scale, compile_options_for_budget_degrade, effect_stack_for_budget_degrade,
};
use effective_view_state::{
    apply_effective_view_settings, apply_renderer_owned_particle_previous_state,
    build_renderer_owned_post_process_snapshot, frame_history_invalidation_reason,
    resolve_view_family_pipeline_for_submission,
};
use ui_submission_stats::compute_ui_submission_stats;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn build_frame_submission_context_from_runtime_frame_extract(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    extract: &Arc<RenderFrameExtract>,
    ui_submission: Option<&UiRenderSubmission>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    build_frame_submission_context_from_source(framework, viewport, extract, ui_submission)
}

fn build_frame_submission_context_from_source(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    source_extract: &Arc<RenderFrameExtract>,
    ui_submission: Option<&UiRenderSubmission>,
) -> Result<FrameSubmissionContext, RenderFrameworkError> {
    // The generation-owned scene remains shared and immutable. Renderer policy is derived into
    // this compact per-submission view/timing overlay.
    let mut submission_extract = source_extract.as_ref().clone();
    let budget_degrade_settings = {
        let state = framework.lock_state();
        state.degrade_ladder.settings()
    };
    apply_budget_render_scale(&mut submission_extract, budget_degrade_settings);
    let mut viewport_state =
        resolve_viewport_record_state(framework, viewport, &submission_extract)?;
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
    let (subsurface_profiles, subsurface_material_profile_indices) =
        resolve_subsurface_material_profiles(asset_manager.as_ref(), &submission_extract);
    let material_features =
        resolve_advanced_pbr_material_usage(asset_manager.as_ref(), &submission_extract);
    let advanced_lighting_inputs = AdvancedLightingCompileInputs::new(
        material_features,
        subsurface_profiles,
        subsurface_material_profile_indices,
    );
    let environment_ibl_cache_resolution = resolve_and_rehydrate_environment_ibl_cache(
        asset_manager.as_ref(),
        &environment_ibl_hydration_cache,
        &submission_extract,
    )?;
    let resolved_camera_target = resolve_camera_target_descriptor(
        primary_target_size,
        submission_extract.view.selected_camera_target(),
        asset_manager.as_ref(),
    )?;
    let submission_size = resolved_camera_target.size();
    let compile_camera_target = resolved_camera_target.compile_fingerprint();
    submission_extract.apply_viewport_size(submission_size);
    let particle_previous_sprites_override =
        apply_renderer_owned_particle_previous_state(&submission_extract, &mut viewport_state);
    let sized_extract = &submission_extract;
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
    )
    .with_advanced_lighting_inputs(advanced_lighting_inputs);
    let compiled_pipeline = compile_submission_pipeline_with_options(
        framework,
        &viewport_state,
        sized_extract,
        compile_camera_target,
        &budget_compile_options,
    )?;
    let advanced_runtime_plan = viewport_state.take_advanced_runtime_plan()?;
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
    let effective_ambient_occlusion = resolved_post_process.ambient_occlusion;
    let effective_exposure = resolved_post_process.exposure;
    let effective_volumetric_fog = sized_extract
        .lighting
        .advanced_lighting
        .volumetric
        .unwrap_or(resolved_post_process.volumetric_fog);
    let effective_color_grading = color_grading_enabled
        .then_some(resolved_post_process.color_grading)
        .unwrap_or_else(RenderColorGradingSettings::default);
    let effective_effect_stack = effect_stack_for_budget_degrade(
        resolved_post_process.effect_stack,
        budget_degrade_settings,
    );
    let source_virtual_geometry = sized_extract.geometry.virtual_geometry.as_ref();
    let source_hybrid_gi = sized_extract.lighting.hybrid_global_illumination.as_ref();
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
    let effective_extract = &submission_extract;
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
    let history_validation_key = FrameHistoryValidationKey::from_extract(
        effective_extract,
        compiled_feature_names(&compiled_pipeline),
    );
    let previous_motion_vector_camera = viewport_state.take_previous_motion_vector_camera();
    let current_motion_vector_camera = effective_extract.view.selected_effective_camera();
    let temporal_reprojection_compatible =
        previous_motion_vector_camera
            .as_ref()
            .is_some_and(|previous| {
                current_motion_vector_camera.supports_temporal_reprojection_from(previous)
            });
    let history_invalidation_reason = frame_history_invalidation_reason(
        framework,
        viewport,
        submission_size,
        render_size,
        viewport_state.pipeline_handle(),
        &compiled_pipeline,
        &camera_history_key,
        &history_validation_key,
        temporal_reprojection_compatible,
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
    let primary_upscale_required = view_family_pipeline
        .phases()
        .contains(&RenderPipelinePhase::PrimarySpatialUpscale);
    let secondary_upscale_required = view_family_pipeline
        .phases()
        .contains(&RenderPipelinePhase::SecondarySpatialUpscale);
    let mut post_process_stack =
        PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale_phases(
            &effective_bloom,
            &effective_color_grading,
            effective_exposure,
            &effective_effect_stack,
            temporal_history_enabled,
            post_process_history_available,
            &anti_alias_report.effective_settings(),
            primary_upscale_required,
            secondary_upscale_required,
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
            .with_ambient_occlusion_source(effective_ambient_occlusion)
            .with_post_process_stack(post_process_stack.clone()),
        budget_degrade_settings,
    );
    let compile_options = compile_options_with_environment_ibl_bake_request(
        effective_extract,
        final_budget_compile_options,
        environment_ibl_cache_resolution.as_ref(),
    )?;
    submission_extract
        .view
        .apply_view_family_pipeline(view_family_pipeline);
    let effective_extract = &submission_extract;
    let compiled_pipeline = compile_submission_pipeline_with_options(
        framework,
        &viewport_state,
        effective_extract,
        compile_camera_target,
        &compile_options,
    )?;
    let temporal_jitter =
        temporal_jitter_for_submission(anti_alias_report, viewport_state.temporal_frame_index());
    apply_effective_view_settings(&mut submission_extract, anti_alias_report, temporal_jitter);
    let effective_extract = &submission_extract;
    let post_process = RendererPostProcessSnapshot::new(
        build_renderer_owned_post_process_snapshot(
            &effective_extract.post_process,
            effective_ambient_occlusion,
            effective_bloom,
            effective_exposure,
            effective_color_grading,
            effective_effect_stack,
            post_process_stack,
            post_process_graph,
        ),
        effective_volumetric_fog,
    );
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
    let particle_previous_state_sprite_count = particle_previous_sprites_override
        .as_deref()
        .map(|previous_sprites| {
            effective_extract
                .particles
                .previous_state_sprite_count_with(previous_sprites)
        })
        .unwrap_or_else(|| effective_extract.particles.previous_state_sprite_count());
    let particle_anonymous_stream_ambiguity_sprite_count = effective_extract
        .particles
        .anonymous_stream_ambiguity_sprite_count();
    let scene_camera_order_report = effective_extract.view.scene_camera_order_report.clone();
    let hybrid_gi_extract_for_context = effective_hybrid_gi_extract;
    let virtual_geometry_extract_for_context = virtual_geometry_enabled
        .then_some(effective_virtual_geometry_extract)
        .flatten();
    let submission_extract = Arc::new(submission_extract);
    let quality_profile_texture_mip_bias = viewport_state.quality_profile_texture_mip_bias();
    let quality_profile_texture_max_anisotropy =
        viewport_state.quality_profile_texture_max_anisotropy();
    let quality_profile = viewport_state.take_quality_profile();
    let capabilities = viewport_state.take_capabilities();
    let (environment_ibl_bake_reservation, environment_source_cubemap_override) =
        environment_ibl_cache_resolution
            .map(EnvironmentIblCacheResolution::into_submission_parts)
            .unwrap_or_default();

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
        view_family_pipeline,
        scene_camera_order_report,
        ui_submission
            .map(compute_ui_submission_stats)
            .unwrap_or_default(),
        post_process,
        anti_alias_report,
        advanced_runtime_plan,
        solari_runtime_report,
        hybrid_gi_enabled,
        virtual_geometry_enabled,
        hybrid_gi_extract_for_context,
        hybrid_gi_payload_source,
        hybrid_gi_update_plan,
        hybrid_gi_feedback,
        submission_extract,
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
    .with_environment_ibl_bake_reservation(environment_ibl_bake_reservation)
    .with_environment_source_cubemap_override(environment_source_cubemap_override)
    .with_particle_previous_sprites_override(particle_previous_sprites_override))
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
