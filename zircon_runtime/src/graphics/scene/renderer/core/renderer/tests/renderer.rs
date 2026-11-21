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

#[test]
fn portable_scene_keeps_ui_and_output_transfer_without_eager_desktop_effect_resources() {
    let options = SceneRendererStartupOptions::portable_scene();
    let profile = options.deferred_lighting_profile();
    assert!(profile.uses_screen_space_ui());
    assert!(profile.uses_environment_only_lighting());
    assert!(profile.defers_local_reflection_provider_resources());
    assert!(!profile.uses_full_shadow_atlas_resources());
    assert!(!profile.uses_auxiliary_scene_effects());
    assert!(!profile.uses_full_post_process_resources());
    assert!(options.requires_environment_only_pbr_base_prewarm());
    assert_eq!(
        profile.post_process_startup_mode(),
        ScenePostProcessStartupMode::OutputTransferOnly
    );
    assert_eq!(
        SceneRendererStartupOptions::default().deferred_lighting_profile(),
        SceneRendererDeferredLightingProfile::FullScene
    );
}

#[test]
fn prepared_world_affine_reaches_direct_and_model_instance_uploads() {
    use crate::asset::{
        AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MESH_ATTRIBUTE_NORMAL,
        MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
    };
    use crate::core::framework::render::{
        render_mesh_stable_instance_key, CameraRenderDescriptor, ProjectionMode,
        RenderExtractContext, RenderMeshTopology, SceneViewportExtractRequest,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{Quat, Transform, UVec2, Vec3};
    use crate::core::resource::{ResourceHandle, ResourceId, ResourceKind, ResourceRecord};
    use crate::graphics::backend::{read_buffer_bytes, BufferByteReadback};
    use crate::graphics::scene::gpu_scene::GpuInstanceData;
    use crate::graphics::types::ViewportRenderFrame;
    use crate::scene::{components::MeshRenderer, NodeKind, World};

    let assets = Arc::new(ProjectAssetManager::default());
    let uri = AssetUri::parse("res://meshes/affine-draw.zmesh").unwrap();
    let mesh_id = ResourceId::from_locator(&uri);
    let mesh = MeshAsset::new(
        uri.clone(),
        RenderMeshTopology::TriangleList,
        std::collections::BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0; 3], [0.25, 0.0, 0.0], [0.0, 0.25, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0; 2], [1.0, 0.0], [0.0, 1.0]]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0, 1, 2])),
    )
    .unwrap();
    assets
        .resource_manager()
        .register_ready(ResourceRecord::new(mesh_id, ResourceKind::Mesh, uri), mesh)
        .expect("publish direct mesh fixture");
    let mut renderer = SceneRenderer::new_for_test(assets).expect("offscreen scene renderer");

    let mut world = World::empty();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let direct = world.spawn_node(NodeKind::Mesh).unwrap();
    world.get_mut::<MeshRenderer>(direct).unwrap().mesh = Some(ResourceHandle::new(mesh_id));
    let model = world.spawn_node(NodeKind::Cube).unwrap();
    let nodes = [direct, model];
    for (entity, x) in [(direct, -0.6), (model, 0.6)] {
        world
            .update_transform(
                entity,
                Transform::from_translation(Vec3::new(x, 0.0, -5.0))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            )
            .unwrap();
        world.set_parent_checked(entity, Some(parent)).unwrap();
    }
    let context = RenderExtractContext::new(
        world.render_world_snapshot_handle(),
        SceneViewportExtractRequest {
            camera: Some(CameraRenderDescriptor::from_camera_payload(
                None,
                ViewportCameraSnapshot {
                    projection_mode: ProjectionMode::Orthographic,
                    ortho_size: 4.0,
                    aspect_ratio: 1.0,
                    ..Default::default()
                },
            )),
            ..Default::default()
        },
    );
    let [(sheared, first), (orthogonal, second)] =
        [Vec3::new(2.0, 1.0, 1.0), Vec3::ONE].map(|scale| {
            world
                .update_transform(parent, Transform::default().with_scale(scale))
                .unwrap();
            let extract = world.build_prepared_render_frame_extract(&context);
            assert_eq!(extract.geometry.meshes.len(), 2);
            let expected =
                nodes.map(|entity| world.world_matrix(entity).unwrap().to_cols_array_2d());
            let frame = ViewportRenderFrame::from_extract(extract, UVec2::splat(64));
            renderer
                .render_frame(&frame)
                .expect("submit prepared frame");
            assert!(renderer.streamer.mesh(&mesh_id).is_some());
            let gpu_scene = &renderer.core.gpu_scene;
            let size = std::mem::size_of::<GpuInstanceData>() as u64;
            let uploaded = nodes.map(|entity| {
                let key = render_mesh_stable_instance_key(entity, 0);
                let entry = gpu_scene.entry(key).expect("ordinary mesh instance entry");
                let bytes = read_buffer_bytes(
                    &renderer.backend.device,
                    &renderer.backend.queue,
                    gpu_scene.instance_buffer(),
                    BufferByteReadback {
                        source_offset: u64::from(entry.first_instance_index) * size,
                        byte_len: size,
                        label: "prepared-world-affine-readback",
                    },
                )
                .expect("read actual uploaded instance without rolling history again");
                bytemuck::pod_read_unaligned::<GpuInstanceData>(&bytes)
            });
            (expected, uploaded)
        });
    assert_eq!(
        first.map(|row| row.world_from_local),
        sheared,
        "direct and model draws must upload the prepared World affine"
    );
    assert_eq!(first.map(|row| row.prev_world_from_local), sheared);
    assert_eq!(second.map(|row| row.world_from_local), orthogonal);
    assert_eq!(second.map(|row| row.prev_world_from_local), sheared);
}
