use std::sync::Arc;

use super::*;
use crate::asset::{ProjectAssetManager, ProjectAssetManagerAccess};
use crate::core::framework::render::{
    EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::graphics::scene::scene_renderer::{SceneRenderer, SceneRendererStartupOptions};

#[test]
fn deferred_lighting_production_path_is_fail_closed() {
    let production = include_str!("../execute_lighting.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("deferred lighting test boundary");
    assert!(production.contains(") -> Result<(), String>"));
    assert!(!production.contains("expect("));
    assert!(!production.contains("unwrap("));
    assert!(!production.contains("panic!("));
}

fn empty_lit_snapshot() -> RenderSceneSnapshot {
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
        environment: EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

#[test]
fn deferred_lighting_color_attachments_use_fixed_stack_storage() {
    let source = include_str!("../execute_lighting.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("deferred lighting implementation");

    assert!(!implementation.contains("let mut color_attachments = vec!["));
    assert!(implementation.contains("let mut color_attachments = ["));
    assert!(implementation.contains("color_attachments[..execution_plan.color_attachment_count]"));
}

#[test]
fn deferred_lighting_selects_a_cached_pipeline_for_the_active_mrt_shape() {
    let source = include_str!("../execute_lighting.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("deferred lighting implementation");

    assert!(implementation.contains("self.lighting_pipelines.pipeline("));
    assert!(implementation.contains("execution_plan.subsurface_mrt,"));
}

#[test]
fn deferred_lighting_preallocates_the_fixed_bind_group_entry_count() {
    let source = include_str!("../execute_lighting.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("deferred lighting implementation");

    assert!(
        implementation.contains("const DEFERRED_LIGHTING_BIND_GROUP_ENTRY_CAPACITY: usize = 29")
    );
    assert!(implementation.contains("ENVIRONMENT_ONLY_PBR_BIND_GROUP_ENTRY_CAPACITY"));
    assert!(implementation.contains("Vec::with_capacity(execution_plan.bind_group_entry_capacity)"));
}

#[test]
fn full_scene_lighting_plan_builds_the_complete_bind_group_shape() {
    let plan = DeferredLightingExecutionPlan::new(
        SceneRendererDeferredLightingProfile::FullScene,
        true,
        true,
    );

    assert!(plan.full_lighting_bind_group);
    assert_eq!(
        plan.bind_group_entry_capacity,
        DEFERRED_LIGHTING_BIND_GROUP_ENTRY_CAPACITY
    );
    assert!(plan.subsurface_mrt);
    assert_eq!(plan.color_attachment_count, 3);
    assert!(plan.uses_gpu_scene);
}

#[test]
fn environment_only_lighting_plan_omits_direct_lighting_resources() {
    let plan = DeferredLightingExecutionPlan::new(
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        true,
        true,
    );

    assert!(!plan.full_lighting_bind_group);
    assert_eq!(
        plan.bind_group_entry_capacity, 10,
        "environment-only lighting must retain local reflection bindings for provider upgrades"
    );
    assert!(!plan.subsurface_mrt);
    assert_eq!(plan.color_attachment_count, 1);
    assert!(!plan.uses_gpu_scene);
}

#[test]
fn environment_only_lighting_retains_local_reflection_bind_group_entries() {
    let source = include_str!("../execute_lighting.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("deferred lighting implementation");

    assert!(implementation
        .contains("entries.extend(self.reflection_probe_bindings.bind_group_entries());"));
    assert!(!implementation.contains("uses_local_reflection_provider"));
}

#[test]
fn deferred_lighting_profiles_construct_bind_groups_during_real_render() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    for (profile_name, startup_options) in [
        ("full-scene", SceneRendererStartupOptions::default()),
        (
            "environment-only",
            SceneRendererStartupOptions::environment_only_pbr_preview(),
        ),
    ] {
        let (mut renderer, _) = SceneRenderer::new_with_startup_options_and_report(
            ProjectAssetManagerAccess::for_test(Arc::clone(&asset_manager)),
            startup_options,
        )
        .unwrap_or_else(|error| panic!("{profile_name} renderer startup failed: {error}"));

        renderer
            .render(empty_lit_snapshot(), UVec2::new(8, 8))
            .unwrap_or_else(|error| {
                panic!("{profile_name} deferred bind-group construction failed: {error}")
            });
    }
}
