use std::sync::Arc;

use crate::asset::{ProjectAssetManager, ProjectAssetManagerAccess};

use super::{
    local_bounds_for_cpu_visibility, ScenePostProcessStartupMode, SceneRenderer,
    SceneRendererDeferredLightingProfile, SceneRendererGpuPassTiming, SceneRendererGpuTimingReport,
    SceneRendererStartupOptions,
};
use crate::core::framework::render::RenderMeshBounds;
use crate::core::framework::render::RenderMeshSubmissionProfile;
use crate::graphics::{
    RuntimePrepareMeshGeometrySeed, RuntimePrepareMeshSdfDeformationReason,
    RuntimePrepareMeshSdfSeed,
};

#[test]
fn gpu_timing_report_keeps_only_an_explicit_frame_matched_mesh_submission_snapshot() {
    let mesh_submission = RenderMeshSubmissionProfile {
        opaque_command_count: 2,
        advanced_pbr_opaque_command_count: 1,
        cached_command_hit_count: 4,
        command_rebuild_count: 0,
        dynamic_command_count: 1,
        ..RenderMeshSubmissionProfile::default()
    };
    let report = SceneRendererGpuTimingReport::new(
        7,
        1.0,
        [SceneRendererGpuPassTiming::new("direct_scene_content", 42)],
    )
    .with_mesh_submission_profile(mesh_submission.clone());

    assert_eq!(report.mesh_submission_profile(), Some(&mesh_submission));
}

#[test]
fn environment_only_pbr_profile_is_the_only_startup_profile_that_prewarm_base() {
    assert!(!SceneRendererStartupOptions::default().requires_environment_only_pbr_base_prewarm());
    assert!(!SceneRendererStartupOptions::standard_pbr_preview()
        .requires_environment_only_pbr_base_prewarm());
    assert!(SceneRendererStartupOptions::environment_only_pbr_preview()
        .requires_environment_only_pbr_base_prewarm());
}

#[test]
fn environment_only_async_pipeline_startup_queues_base_instead_of_waiting_for_it() {
    let options =
        SceneRendererStartupOptions::environment_only_pbr_preview().with_async_pipeline_compile();

    assert!(options.async_pipeline_compile_enabled());
    assert!(!options.requires_environment_only_pbr_base_prewarm());
    assert!(options.queues_environment_only_pbr_base_prewarm());
}

#[test]
fn explicit_generic_forward_owner_can_keep_lightweight_environment_startup_without_unused_prewarm()
{
    let options = SceneRendererStartupOptions::environment_only_pbr_preview()
        .without_environment_only_pbr_base_prewarm()
        .with_async_pipeline_compile();

    assert_eq!(
        options.deferred_lighting_profile(),
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
    );
    assert!(!options.requires_environment_only_pbr_base_prewarm());
    assert!(!options.queues_environment_only_pbr_base_prewarm());
}

#[test]
fn environment_only_pbr_preview_omits_auxiliary_scene_effects() {
    assert!(SceneRendererDeferredLightingProfile::FullScene.uses_auxiliary_scene_effects());
    assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_auxiliary_scene_effects());
    assert!(
        !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
            .uses_auxiliary_scene_effects()
    );
    assert!(SceneRendererDeferredLightingProfile::FullScene.uses_full_post_process_resources());
    assert!(
        SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_full_post_process_resources()
    );
    assert!(
        !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
            .uses_full_post_process_resources()
    );
    assert!(SceneRendererDeferredLightingProfile::FullScene.supports_compiled_scene_graph());
    assert!(
        SceneRendererDeferredLightingProfile::StandardPbrPreview.supports_compiled_scene_graph()
    );
    assert!(
        !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
            .supports_compiled_scene_graph()
    );
    assert!(!SceneRendererDeferredLightingProfile::FullScene
        .defers_local_reflection_provider_resources());
    assert!(!SceneRendererDeferredLightingProfile::StandardPbrPreview
        .defers_local_reflection_provider_resources());
    assert!(
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
            .defers_local_reflection_provider_resources()
    );
    assert!(SceneRendererDeferredLightingProfile::FullScene.uses_screen_space_ui());
    assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_screen_space_ui());
    assert!(
        !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview.uses_screen_space_ui()
    );
}

#[test]
fn environment_only_pbr_preview_uses_a_shadow_atlas_placeholder() {
    assert!(SceneRendererDeferredLightingProfile::FullScene.uses_full_shadow_atlas_resources());
    assert!(
        SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_full_shadow_atlas_resources()
    );
    assert!(
        !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
            .uses_full_shadow_atlas_resources()
    );
}

#[test]
fn environment_only_pbr_preview_omits_direct_light_buffer_work() {
    assert!(SceneRendererDeferredLightingProfile::FullScene.uses_direct_lights());
    assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_direct_lights());
    assert!(!SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview.uses_direct_lights());
}

#[test]
fn environment_only_pbr_shader_has_no_direct_light_buffer_lookup() {
    let source = include_str!("../../../../../shader/wgsl/zr_shading_environment_only_pbr.wgsl");

    assert!(
        !source.contains("zr_gpu_scene_light"),
        "the environment-only shader must not consume the direct-light buffer"
    );
}

#[test]
fn environment_only_pbr_preview_omits_compiled_scene_shadow_renderer() {
    assert!(SceneRendererDeferredLightingProfile::FullScene.uses_shadow_map_renderer());
    assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_shadow_map_renderer());
    assert!(
        !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview.uses_shadow_map_renderer()
    );
}

#[test]
fn shadow_map_renderer_construction_tracks_deferred_lighting_profile() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    for (profile_name, startup_options, expects_shadow_map_renderer) in [
        ("full-scene", SceneRendererStartupOptions::default(), true),
        (
            "standard-pbr-preview",
            SceneRendererStartupOptions::standard_pbr_preview(),
            true,
        ),
        (
            "environment-only-pbr-preview",
            SceneRendererStartupOptions::environment_only_pbr_preview(),
            false,
        ),
    ] {
        let (renderer, _) = SceneRenderer::new_with_startup_options_and_report(
            ProjectAssetManagerAccess::for_test(Arc::clone(&asset_manager)),
            startup_options,
        )
        .unwrap_or_else(|error| panic!("{profile_name} renderer startup failed: {error}"));

        assert_eq!(
            renderer.core.shadow_map_renderer.is_some(),
            expects_shadow_map_renderer,
            "{profile_name} shadow renderer construction did not match its profile"
        );
    }
}

#[test]
fn environment_only_pbr_preview_excludes_editor_interaction_overlays() {
    assert!(SceneRendererDeferredLightingProfile::FullScene.uses_interaction_overlays());
    assert!(SceneRendererDeferredLightingProfile::StandardPbrPreview.uses_interaction_overlays());
    assert!(
        !SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
            .uses_interaction_overlays()
    );
}

#[test]
fn post_process_startup_mode_preserves_full_graphs_and_selects_viewer_transfer_only() {
    assert_eq!(
        SceneRendererDeferredLightingProfile::FullScene.post_process_startup_mode(),
        ScenePostProcessStartupMode::Full
    );
    assert_eq!(
        SceneRendererDeferredLightingProfile::StandardPbrPreview.post_process_startup_mode(),
        ScenePostProcessStartupMode::Full
    );
    assert_eq!(
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview.post_process_startup_mode(),
        ScenePostProcessStartupMode::OutputTransferOnly
    );
}

#[test]
fn gpu_timing_resources_require_an_explicit_startup_option() {
    assert!(!SceneRendererStartupOptions::default().allow_gpu_timing());
    assert!(SceneRendererStartupOptions::default()
        .with_gpu_timing()
        .allow_gpu_timing());
}

#[test]
fn skinned_geometry_omits_unposed_bounds_from_cpu_visibility() {
    let local_bounds = RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]);
    let skinned = RuntimePrepareMeshGeometrySeed {
        local_bounds,
        resource_revision: 7,
        shape_revision: 11,
        mesh_sdf: RuntimePrepareMeshSdfSeed::Deforming(
            RuntimePrepareMeshSdfDeformationReason::Skinning,
        ),
    };
    assert_eq!(local_bounds_for_cpu_visibility(skinned), None);

    let morphed = RuntimePrepareMeshGeometrySeed {
        local_bounds,
        resource_revision: 7,
        shape_revision: 12,
        mesh_sdf: RuntimePrepareMeshSdfSeed::Deforming(
            RuntimePrepareMeshSdfDeformationReason::ActiveMorphTargets,
        ),
    };
    assert_eq!(local_bounds_for_cpu_visibility(morphed), Some(local_bounds));
}
