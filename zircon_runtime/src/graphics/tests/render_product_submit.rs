use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode,
    RenderAmbientLightSnapshot, RenderFrameExtract, RenderFramework, RenderMeshSnapshot,
    RenderOverlayExtract, RenderPipelineHandle, RenderQualityProfile, RenderRectLightSnapshot,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::{ViewportRenderFrame, WgpuRenderFramework};

#[test]
fn render_product_submit_direct_extract_frame_does_not_use_legacy_scene_snapshot_authority() {
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(88),
        snapshot_with_projection(ProjectionMode::Orthographic),
    );

    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(320, 240));

    assert_eq!(frame.camera().projection_mode, ProjectionMode::Orthographic);
    assert_eq!(
        frame.scene.scene.camera.projection_mode,
        ProjectionMode::Perspective,
        "legacy scene snapshot must not be the product submit draw authority"
    );
    assert_eq!(frame.overlays().display_mode, DisplayMode::WireOnly);
    assert_eq!(frame.scene.overlays.display_mode, DisplayMode::Shaded);
}

#[test]
fn render_product_submit_unknown_viewport_returns_error_without_panic() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework.destroy_viewport(viewport).unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(89),
                snapshot_with_projection(ProjectionMode::Perspective),
            ),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        crate::core::framework::render::RenderFrameworkError::UnknownViewport { viewport: 1 }
    ));
}

#[test]
fn render_product_submit_selects_default_pipeline_from_extract_core_pipeline() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(90),
                snapshot_with_projection(ProjectionMode::Orthographic),
            ),
        )
        .unwrap();
    assert_eq!(
        framework.query_stats().unwrap().last_pipeline,
        Some(RenderPipelineHandle::new(3))
    );

    framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(91),
                snapshot_with_projection(ProjectionMode::Perspective),
            ),
        )
        .unwrap();
    assert_eq!(
        framework.query_stats().unwrap().last_pipeline,
        Some(RenderPipelineHandle::new(1))
    );
}

#[test]
fn render_product_submit_preserves_quality_profile_pipeline_override() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("explicit-forward")
                .with_pipeline_asset(RenderPipelineHandle::new(1)),
        )
        .unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(92),
                snapshot_with_projection(ProjectionMode::Orthographic),
            ),
        )
        .unwrap_err();

    assert!(
        matches!(error, crate::core::framework::render::RenderFrameworkError::Backend(ref message) if message.contains("core pipeline mismatch")),
        "unexpected error: {error:?}"
    );
}

#[test]
fn render_product_pbr_submit_reports_material_fallback_and_light_stats() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("pbr-material-light-stats")
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(93),
        snapshot_with_projection(ProjectionMode::Perspective),
    );
    extract.geometry = crate::core::framework::render::GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![pbr_mesh_with_missing_material()],
    );
    extract
        .lighting
        .ambient_lights
        .push(RenderAmbientLightSnapshot {
            color: Vec3::new(0.04, 0.05, 0.06),
            intensity: 0.25,
            renderer_degraded: true,
            degradation_reason: Some(
                "ambient light renderer path is deferred after M5A".to_string(),
            ),
        });
    extract.lighting.rect_lights.push(RenderRectLightSnapshot {
        node_id: 700,
        position: Vec3::new(1.0, 2.0, 3.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 4.0,
        size: Vec2::new(2.0, 0.5),
        renderer_degraded: true,
        degradation_reason: Some("rect light renderer path is deferred after M5A".to_string()),
    });

    framework.submit_frame_extract(viewport, extract).unwrap();

    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.last_material_count, 1);
    assert_eq!(stats.last_material_ready_count, 0);
    assert_eq!(stats.last_material_fallback_count, 1);
    assert_eq!(stats.last_material_validation_error_count, 1);
    assert_eq!(stats.last_ambient_light_count, 1);
    assert_eq!(stats.last_rect_light_count, 1);
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 0);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 0);
}

pub(super) fn snapshot_with_projection_for_sprite_tests(
    projection_mode: ProjectionMode,
) -> RenderSceneSnapshot {
    snapshot_with_projection(projection_mode)
}

fn snapshot_with_projection(projection_mode: ProjectionMode) -> RenderSceneSnapshot {
    let camera = ViewportCameraSnapshot {
        projection_mode,
        ..ViewportCameraSnapshot::default()
    };
    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::WireOnly,
            ..RenderOverlayExtract::default()
        },
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

fn pbr_mesh_with_missing_material() -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 600,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/not-registered",
        )),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        render_layer_mask: u32::MAX,
    }
}
