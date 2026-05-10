use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderBloomSettings,
    RenderColorGradingSettings, RenderDirectionalLightSnapshot, RenderFrameExtract,
    RenderFramework, RenderFrameworkError, RenderMeshSnapshot, RenderOverlayExtract,
    RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderViewportDescriptor, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::backend::RenderBackendConfig;
use crate::graphics::runtime::{
    renderdoc_capture_next_from_value, FrameHistoryValidationKey, ViewportFrameHistory,
};
use crate::graphics::{debug_markers, RenderPassStage};
use crate::scene::components::{default_render_layer_mask, Mobility};
use crate::{
    FrameHistoryBinding, FrameHistoryHandle, FrameHistorySlot, VisibilityHistorySnapshot,
    WgpuRenderFramework,
};

#[test]
fn graphics_debugger_status_defaults_to_idle_for_wgpu() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();

    let status = framework.query_graphics_debugger_status().unwrap();

    assert!(status.available);
    assert!(status.backend_name.starts_with("wgpu("));
    if let Ok(requested_backend) = std::env::var("WGPU_BACKEND") {
        match requested_backend.to_ascii_lowercase().as_str() {
            "dx12" | "vulkan" => assert_eq!(
                status.backend_name,
                format!("wgpu({})", requested_backend.to_ascii_lowercase())
            ),
            _ => {}
        }
    }
    assert!(!status.capture_pending);
    assert!(!status.active_capture);
    assert_eq!(status.last_capture_frame, None);
    assert_eq!(status.last_error, None);
}

#[test]
fn render_backend_config_honors_renderdoc_wgpu_env_selection() {
    let dx12 = RenderBackendConfig::from_env_values(Some("dx12"), None, None);
    let vulkan = RenderBackendConfig::from_env_values(Some("vulkan"), None, None);
    let flags_disabled = RenderBackendConfig::from_env_values(None, Some("0"), Some("0"));
    let flags_enabled = RenderBackendConfig::from_env_values(None, Some("1"), Some("1"));

    assert_eq!(dx12.backends, wgpu::Backends::DX12);
    assert_eq!(vulkan.backends, wgpu::Backends::VULKAN);
    assert!(!flags_disabled
        .instance_flags
        .contains(wgpu::InstanceFlags::DEBUG));
    assert!(!flags_disabled
        .instance_flags
        .contains(wgpu::InstanceFlags::VALIDATION));
    assert!(flags_enabled
        .instance_flags
        .contains(wgpu::InstanceFlags::DEBUG));
    assert!(flags_enabled
        .instance_flags
        .contains(wgpu::InstanceFlags::VALIDATION));
}

#[test]
fn renderdoc_capture_next_environment_arms_first_created_viewport_only() {
    assert!(renderdoc_capture_next_from_value(Some("1")));
    assert!(!renderdoc_capture_next_from_value(Some("0")));
    assert!(!renderdoc_capture_next_from_value(None));

    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    framework.request_next_created_viewport_graphics_debugger_capture_for_tests();
    let first = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();
    let second = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    assert!(
        framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );
    framework
        .submit_frame_extract(second, empty_extract())
        .unwrap();
    assert!(
        framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );
    framework
        .submit_frame_extract(first, empty_extract())
        .unwrap();
    assert!(
        !framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );
}

#[test]
fn renderdoc_pending_capture_clears_when_armed_viewport_is_destroyed() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    framework.request_next_created_viewport_graphics_debugger_capture_for_tests();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();
    assert!(
        framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );

    framework.destroy_viewport(viewport).unwrap();

    let status = framework.query_graphics_debugger_status().unwrap();
    assert!(!status.capture_pending);
    assert!(!status.active_capture);
    assert!(status
        .last_error
        .as_deref()
        .is_some_and(|message| message.contains("destroyed")));
}

#[test]
fn renderdoc_debug_marker_registry_covers_capture_timeline() {
    assert_eq!(
        debug_markers::REQUIRED_RENDERDOC_STAGE_MARKERS,
        &[
            "zircon::FrameExtract",
            "zircon::Clear",
            "zircon::Prepass",
            "zircon::MainScene",
            "zircon::Lighting",
            "zircon::DeferredLighting",
            "zircon::PostProcess",
            "zircon::HistoryCopy",
            "zircon::Overlay",
            "zircon::UI",
            "zircon::Readback",
        ]
    );
    assert_eq!(
        debug_markers::marker_for_render_pass_stage(RenderPassStage::Lighting),
        Some(debug_markers::RENDERDOC_MARKER_LIGHTING)
    );
    assert_eq!(
        debug_markers::marker_for_render_pass_stage(RenderPassStage::PostProcess),
        Some(debug_markers::RENDERDOC_MARKER_POST_PROCESS)
    );
    assert_eq!(
        debug_markers::marker_for_render_pass_stage(RenderPassStage::Ui),
        Some(debug_markers::RENDERDOC_MARKER_UI)
    );
    assert_eq!(
        debug_markers::marker_for_render_pass_stage(RenderPassStage::Overlay),
        Some(debug_markers::RENDERDOC_MARKER_OVERLAY)
    );
}

#[test]
fn graphics_debugger_capture_request_consumes_only_matching_viewport_submit() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let requested = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();
    let other = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .request_graphics_debugger_capture(requested)
        .unwrap();
    assert!(
        framework
            .query_graphics_debugger_status()
            .unwrap()
            .capture_pending
    );

    framework
        .submit_frame_extract(other, empty_extract())
        .unwrap();
    let status_after_other = framework.query_graphics_debugger_status().unwrap();
    assert!(status_after_other.capture_pending);
    assert_eq!(status_after_other.last_capture_frame, None);

    framework
        .submit_frame_extract(requested, empty_extract())
        .unwrap();
    let status_after_requested = framework.query_graphics_debugger_status().unwrap();
    let stats = framework.query_stats().unwrap();

    assert!(!status_after_requested.capture_pending);
    assert!(!status_after_requested.active_capture);
    assert_eq!(
        status_after_requested.last_capture_frame,
        stats.last_generation
    );
    assert_eq!(status_after_requested.last_error, None);
}

#[test]
fn graphics_debugger_capture_rejects_unknown_viewport() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();

    let error = framework
        .request_graphics_debugger_capture(
            crate::core::framework::render::RenderViewportHandle::new(404),
        )
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::UnknownViewport { viewport: 404 }
    );
}

#[test]
fn graphics_debugger_capture_request_is_consumed_when_matching_submit_fails_before_capture_start() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .request_graphics_debugger_capture(viewport)
        .unwrap();
    framework
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();

    let error = framework
        .submit_frame_extract(viewport, orthographic_extract())
        .unwrap_err();
    assert!(
        matches!(error, RenderFrameworkError::Backend(ref message) if message.contains("core pipeline mismatch"))
    );

    let status = framework.query_graphics_debugger_status().unwrap();
    assert!(!status.capture_pending);
    assert!(!status.active_capture);
    assert_eq!(status.last_capture_frame, None);
    assert!(status
        .last_error
        .as_deref()
        .is_some_and(|message| message.contains("core pipeline mismatch")));
}

#[test]
fn frame_history_validation_key_rejects_camera_or_mesh_motion() {
    let viewport_size = UVec2::new(320, 240);
    let pipeline = RenderPipelineHandle::new(1);
    let bindings = vec![FrameHistoryBinding::read_write(
        FrameHistorySlot::SceneColor,
    )];
    let base_extract = extract_with_camera_and_mesh(
        ViewportCameraSnapshot::default(),
        Transform::from_translation(Vec3::ZERO),
    );
    let base_key =
        FrameHistoryValidationKey::from_extract(&base_extract, vec!["history_resolve".to_string()]);
    let history = ViewportFrameHistory::new(
        FrameHistoryHandle::new(1),
        viewport_size,
        pipeline,
        1,
        bindings.clone(),
        VisibilityHistorySnapshot::default(),
        base_key,
    );

    assert!(history.is_compatible(
        viewport_size,
        pipeline,
        &bindings,
        &FrameHistoryValidationKey::from_extract(
            &base_extract,
            vec!["history_resolve".to_string()],
        ),
    ));

    let moved_camera = ViewportCameraSnapshot {
        transform: Transform::from_translation(Vec3::new(0.25, 0.0, 0.0)),
        ..ViewportCameraSnapshot::default()
    };
    let moved_camera_extract =
        extract_with_camera_and_mesh(moved_camera, Transform::from_translation(Vec3::ZERO));
    assert!(!history.is_compatible(
        viewport_size,
        pipeline,
        &bindings,
        &FrameHistoryValidationKey::from_extract(
            &moved_camera_extract,
            vec!["history_resolve".to_string()],
        ),
    ));

    let moved_mesh_extract = extract_with_camera_and_mesh(
        ViewportCameraSnapshot::default(),
        Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
    );
    assert!(!history.is_compatible(
        viewport_size,
        pipeline,
        &bindings,
        &FrameHistoryValidationKey::from_extract(
            &moved_mesh_extract,
            vec!["history_resolve".to_string()],
        ),
    ));
}

#[test]
fn frame_history_validation_key_rejects_lighting_and_post_process_changes() {
    let viewport_size = UVec2::new(320, 240);
    let pipeline = RenderPipelineHandle::new(1);
    let bindings = vec![FrameHistoryBinding::read_write(
        FrameHistorySlot::SceneColor,
    )];
    let base_extract = extract_with_camera_and_mesh(
        ViewportCameraSnapshot::default(),
        Transform::from_translation(Vec3::ZERO),
    );
    let base_key = FrameHistoryValidationKey::from_extract(&base_extract, Vec::new());
    let history = ViewportFrameHistory::new(
        FrameHistoryHandle::new(1),
        viewport_size,
        pipeline,
        1,
        bindings.clone(),
        VisibilityHistorySnapshot::default(),
        base_key,
    );

    let mut relit_extract = base_extract.clone();
    relit_extract
        .lighting
        .directional_lights
        .push(RenderDirectionalLightSnapshot {
            node_id: 71,
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 3.0,
        });
    assert!(!history.is_compatible(
        viewport_size,
        pipeline,
        &bindings,
        &FrameHistoryValidationKey::from_extract(&relit_extract, Vec::new()),
    ));

    let mut graded_extract = base_extract.clone();
    graded_extract.post_process.bloom = RenderBloomSettings {
        threshold: 0.5,
        intensity: 2.0,
        radius: 0.25,
    };
    graded_extract.post_process.color_grading = RenderColorGradingSettings {
        exposure: 1.2,
        contrast: 1.1,
        saturation: 0.9,
        gamma: 1.0,
        tint: Vec3::new(1.0, 0.95, 0.9),
    };
    assert!(!history.is_compatible(
        viewport_size,
        pipeline,
        &bindings,
        &FrameHistoryValidationKey::from_extract(&graded_extract, Vec::new()),
    ));
}

#[test]
fn history_resolve_requires_explicit_compile_opt_in() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(viewport, empty_extract())
        .unwrap();
    assert!(!framework
        .query_stats()
        .unwrap()
        .last_effective_features
        .contains(&"history_resolve".to_string()));

    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("temporal-debug").with_history_resolve(true),
        )
        .unwrap();
    framework
        .submit_frame_extract(viewport, empty_extract())
        .unwrap();
    assert!(framework
        .query_stats()
        .unwrap()
        .last_effective_features
        .contains(&"history_resolve".to_string()));
}

fn empty_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}

fn orthographic_extract() -> RenderFrameExtract {
    let mut extract = empty_extract();
    extract.view.camera.projection_mode = ProjectionMode::Orthographic;
    extract.view.core_pipeline = extract.view.camera.core_pipeline_kind();
    extract
}

fn extract_with_camera_and_mesh(
    camera: ViewportCameraSnapshot,
    mesh_transform: Transform,
) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(7),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: vec![RenderMeshSnapshot {
                    node_id: 9,
                    transform: mesh_transform,
                    model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                        "history-test-model",
                    )),
                    material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                        "history-test-material",
                    )),
                    tint: Vec4::ONE,
                    mobility: Mobility::Dynamic,
                    render_layer_mask: default_render_layer_mask(),
                }],
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}
