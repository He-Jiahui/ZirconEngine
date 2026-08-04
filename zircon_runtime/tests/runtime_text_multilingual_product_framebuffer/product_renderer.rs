use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode,
    RenderFrameExtract, RenderFramework, RenderOverlayExtract, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec4};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::support;

const TEXT_RASTER_SETTLE_MAX_FRAMES: u64 = 120;
const TEXT_RASTER_SETTLE_FRAME_DELAY_MILLIS: u64 = 2;

pub(super) fn render_ui_extract_frame(
    ui: UiRenderExtract,
    viewport_size: UVec2,
    asset_manager: Arc<ProjectAssetManager>,
) -> (
    zircon_runtime::core::framework::render::CapturedFrame,
    zircon_runtime::core::framework::render::RenderStats,
) {
    let asset_runtime = support::ProjectAssetTestRuntime::new(asset_manager);
    let server = WgpuRenderFramework::new(asset_runtime.access()).expect("headless WGPU renderer");
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("headless viewport");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-multilingual-text")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false),
        )
        .expect("text proof quality profile");
    let contains_text = ui
        .list
        .commands
        .iter()
        .any(|command| command.text.is_some());
    let settle_frame_limit = if contains_text {
        TEXT_RASTER_SETTLE_MAX_FRAMES
    } else {
        2_u64
    };
    let mut final_stats = None;
    let mut raster_was_settled = false;
    let mut capture_is_stable = false;
    for frame_index in 0..settle_frame_limit {
        server
            .submit_frame_extract_with_ui(
                viewport,
                empty_extract(viewport_size, frame_index + 1),
                Some(ui.clone()),
            )
            .expect("submit multilingual text settle frame");
        let stats = server.query_stats().expect("text proof render stats");
        let raster_is_settled = text_raster_frame_is_settled(
            stats.last_ui_text_raster_worker_pending_count,
            stats.last_ui_text_raster_worker_failed_count,
            stats.last_ui_text_visible_missing_raster_image_count,
            stats.last_ui_text_visible_raster_placeholder_count,
            stats.last_ui_text_raster_renderer_upload_requeued_count,
            stats.last_ui_text_raster_renderer_upload_failure_count,
        );
        final_stats = Some(stats);
        capture_is_stable = text_raster_capture_is_stable(raster_was_settled, raster_is_settled);
        if contains_text && capture_is_stable {
            break;
        }
        raster_was_settled = contains_text && raster_is_settled;
        // Yield only between fresh statistics polls. Two consecutive successful raster frames
        // prevent capture while a newly submitted glyph is still pending.
        if frame_index + 1 < settle_frame_limit {
            std::thread::sleep(std::time::Duration::from_millis(
                TEXT_RASTER_SETTLE_FRAME_DELAY_MILLIS,
            ));
        }
    }

    let stats = final_stats.expect("the settle loop submits at least one frame");
    assert!(
        !contains_text || capture_is_stable,
        "multilingual text framebuffer must observe two consecutive successful raster frames before capture: {stats:#?}"
    );
    let capture = server
        .capture_frame(viewport)
        .expect("capture multilingual text frame")
        .expect("submitted frame must be capturable");
    (capture, stats)
}

fn text_raster_frame_is_settled(
    pending_count: usize,
    failed_count: usize,
    visible_missing_image_count: usize,
    visible_placeholder_count: usize,
    renderer_upload_requeued_count: usize,
    renderer_upload_failure_count: usize,
) -> bool {
    pending_count == 0
        && failed_count == 0
        && visible_missing_image_count == 0
        && visible_placeholder_count == 0
        && renderer_upload_requeued_count == 0
        && renderer_upload_failure_count == 0
}

fn text_raster_capture_is_stable(
    previous_frame_settled: bool,
    current_frame_settled: bool,
) -> bool {
    previous_frame_settled && current_frame_settled
}

fn empty_extract(viewport_size: UVec2, snapshot_id: u64) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: zircon_runtime::core::math::Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(snapshot_id),
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
            overlays: RenderOverlayExtract::default(),
            environment: EnvironmentExtract::default(),
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

#[test]
fn product_framebuffer_capture_requires_two_consecutive_successful_raster_frames() {
    assert!(text_raster_frame_is_settled(0, 0, 0, 0, 0, 0));
    assert!(!text_raster_frame_is_settled(1, 0, 0, 0, 0, 0));
    assert!(!text_raster_frame_is_settled(0, 1, 0, 0, 0, 0));
    assert!(!text_raster_frame_is_settled(0, 0, 1, 0, 0, 0));
    assert!(!text_raster_frame_is_settled(0, 0, 0, 1, 0, 0));
    assert!(!text_raster_frame_is_settled(0, 0, 0, 0, 1, 0));
    assert!(!text_raster_frame_is_settled(0, 0, 0, 0, 0, 1));
    assert!(!text_raster_capture_is_stable(false, true));
    assert!(!text_raster_capture_is_stable(true, false));
    assert!(text_raster_capture_is_stable(true, true));
}
