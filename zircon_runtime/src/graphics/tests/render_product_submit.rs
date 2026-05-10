use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    DisplayMode, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderOverlayExtract, RenderPipelineHandle, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
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
