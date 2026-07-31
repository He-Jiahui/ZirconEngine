use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetUri, RGBA8_UNORM_SRGB_FORMAT, TextureAsset, TextureAssetDescriptor};
use crate::core::framework::render::{
    CapturedFrame, OverlayLineSegment, ProjectionMode, RenderCameraTarget,
    RenderCameraTargetGraphImportStatus, RenderCameraTargetKind, RenderCameraTargetWritebackStatus,
    RenderCaptureSource, RenderDynamicResolutionSettings, RenderFrameExtract, RenderFramework,
    RenderImageColorSpace, RenderImageFallbackKind, RenderImageUsage, RenderPipelineHandle,
    RenderQualityProfile, RenderSamplerDescriptor, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, SceneGizmoKind, SceneGizmoOverlayExtract,
};
use crate::core::math::{UVec2, Vec3, Vec4};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::{
    CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset, WgpuRenderFramework,
};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap, UiVisualAssetRef,
};

#[test]
fn render_product_ui_compile_order_tracks_core2d_and_core3d_terminal_semantics() {
    let core2d = RenderPipelineAsset::default_core2d()
        .compile(&orthographic_extract())
        .unwrap();
    assert_ui_after_postprocess_before_overlay(&core2d.pass_stages);

    let forward = RenderPipelineAsset::default_forward_plus()
        .compile(&perspective_extract())
        .unwrap();
    assert_ui_after_overlay_for_default_3d(&forward);

    let deferred = RenderPipelineAsset::default_deferred()
        .compile(&perspective_extract())
        .unwrap();
    assert_ui_after_overlay_for_default_3d(&deferred);
}

#[test]
fn render_product_ui_submit_records_graph_pass_order_and_payload_stats() {
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-product")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    framework
        .submit_frame_extract_with_ui(
            viewport,
            perspective_extract(),
            Some(runtime_ui_extract_with_image_and_clip()),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_ui_command_count, 2);
    assert_eq!(stats.last_ui_quad_count, 1);
    assert_eq!(stats.last_ui_text_payload_count, 1);
    assert_eq!(stats.last_ui_image_payload_count, 1);
    assert_eq!(stats.last_ui_clipped_command_count, 1);
    assert_eq!(stats.last_ui_graph_executed_pass_count, 1);
    assert_eq!(stats.last_ui_target_size, Some(UVec2::new(320, 240)));
    assert_eq!(
        stats.last_ui_graph_pass_order.as_deref(),
        Some("postprocess-overlay-ui"),
        "executed passes: {:?}; executor ids: {:?}",
        stats.last_graph_executed_passes,
        stats.last_graph_executed_executor_ids
    );

    let post = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "uber")
        .expect("postprocess pass should stay before runtime UI");
    let ui = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "runtime-ui")
        .expect("runtime UI pass should be graph-executed");
    let overlay = stats
        .last_graph_executed_passes
        .iter()
        .position(|pass| pass == "overlay-gizmo")
        .expect("overlay pass should stay before runtime UI");

    assert!(post < overlay && overlay < ui);
    assert_eq!(
        stats.last_graph_executed_passes.last().map(String::as_str),
        Some("runtime-ui")
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"ui.screen-space".to_string())
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"overlay.gizmo".to_string())
    );
}

#[test]
fn render_product_ui_submit_keeps_presentation_target_under_dynamic_resolution() {
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-dynamic-resolution")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    let mut extract = perspective_extract();
    extract.view.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    extract.view.sync_selected_descriptor_camera_payload();
    framework
        .submit_frame_extract_with_ui(
            viewport,
            extract,
            Some(runtime_ui_extract_with_image_and_clip()),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_frame_target_size, Some(viewport_size));
    assert_eq!(
        stats.last_frame_render_size,
        Some(UVec2::new(160, 120)),
        "scene/postprocess graph resources should use the scaled internal render size"
    );
    assert_eq!(
        stats.last_ui_target_size,
        Some(viewport_size),
        "runtime UI must composite onto the unscaled presentation target"
    );
    assert_eq!(
        stats.last_ui_graph_pass_order.as_deref(),
        Some("postprocess-overlay-ui")
    );
}

#[test]
fn render_product_ui_submit_targets_direct_import_texture_under_dynamic_resolution() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri =
        AssetUri::parse("res://tests/runtime-ui/direct-import-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    let texture_size = UVec2::new(96, 54);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            srgb_render_target_texture_asset(texture_uri, texture_size),
        )
        .expect("texture target insert");

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport_size = UVec2::new(320, 240);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-direct-import-target")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false),
        )
        .unwrap();

    let mut extract = perspective_extract();
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("test extract should carry a selected camera descriptor")
        .target = RenderCameraTarget::Texture(ResourceHandle::<TextureMarker>::new(texture_id));
    extract.view.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    extract.view.sync_selected_descriptor_camera_payload();
    framework
        .submit_frame_extract_with_ui(
            viewport,
            extract,
            Some(runtime_ui_extract_with_image_and_clip()),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_frame_target_size, Some(texture_size));
    assert_eq!(
        stats.last_frame_render_size,
        Some(UVec2::new(48, 27)),
        "scene/postprocess resources should scale from the resolved texture target extent"
    );
    assert_eq!(
        stats.last_ui_target_size,
        Some(texture_size),
        "runtime UI must composite onto the imported texture presentation target"
    );
    assert_eq!(
        stats.last_ui_graph_pass_order.as_deref(),
        Some("postprocess-overlay-ui")
    );
    assert_eq!(
        stats.last_camera_target_resolution.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_resolution.primary_target_size,
        viewport_size
    );
    assert_eq!(
        stats.last_camera_target_resolution.resolved_target_size,
        texture_size
    );
    assert_eq!(
        stats.last_camera_target_resolution.effective_view_size,
        texture_size
    );
    assert_eq!(
        stats.last_camera_target_resolution.effective_render_size,
        UVec2::new(48, 27)
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        stats.last_camera_target_graph_import.target_size,
        texture_size
    );
    assert_eq!(stats.last_camera_target_graph_import.direct_import_count, 1);
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(stats.last_camera_target_writeback.target_size, texture_size);
    assert_eq!(
        stats.last_capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(stats.last_capture_report.output_size, texture_size);
}

#[test]
fn render_product_ui_submit_keeps_ui_pixels_over_scene_overlay_product() {
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("runtime-ui-scene-overlay-product")
                .with_pipeline_asset(RenderPipelineHandle::new(1))
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    framework
        .submit_frame_extract_with_ui(
            viewport,
            perspective_extract_with_overlay_lattice(),
            Some(runtime_ui_extract_with_center_quad()),
        )
        .unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("runtime UI overlay product frame should be capturable");
    let stats = framework.query_stats().unwrap();

    assert_eq!(
        stats.last_ui_graph_pass_order.as_deref(),
        Some("postprocess-overlay-ui")
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"overlay.gizmo".to_string())
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"ui.screen-space".to_string())
    );

    let inner_ui_origin = UVec2::new(104, 80);
    let inner_ui_size = UVec2::new(112, 80);
    let inner_ui_pixels = (inner_ui_size.x * inner_ui_size.y) as usize;
    let cyan_pixels = dominant_cyan_pixels_in_region(&frame, inner_ui_origin, inner_ui_size);
    let green_pixels = dominant_green_pixels_in_region(&frame, inner_ui_origin, inner_ui_size);

    assert!(
        cyan_pixels > inner_ui_pixels * 9 / 10,
        "runtime UI should remain dominant over dense scene overlay lines; cyan={cyan_pixels}, green={green_pixels}, total={inner_ui_pixels}"
    );
    assert!(
        green_pixels < inner_ui_pixels / 20,
        "scene overlay lines should not overwrite the terminal runtime UI product; cyan={cyan_pixels}, green={green_pixels}, total={inner_ui_pixels}"
    );
}

fn assert_ui_after_postprocess_before_overlay(
    pass_stages: &[crate::graphics::CompiledRenderPipelinePassStage],
) {
    let postprocess = pass_stages
        .iter()
        .position(|entry| entry.stage == RenderPassStage::PostProcess)
        .expect("pipeline should compile at least one postprocess graph pass");
    let ui = pass_stages
        .iter()
        .position(|entry| entry.stage == RenderPassStage::Ui)
        .expect("pipeline should compile a runtime UI graph pass");
    let overlay = pass_stages
        .iter()
        .position(|entry| entry.stage == RenderPassStage::Debug)
        .expect("pipeline should compile an overlay/debug graph pass");

    assert!(postprocess < ui && ui < overlay);
}

fn assert_ui_after_overlay_for_default_3d(compiled: &CompiledRenderPipeline) {
    let postprocess = compiled
        .stages
        .iter()
        .position(|stage| *stage == RenderPassStage::PostProcess)
        .expect("pipeline should compile at least one postprocess graph pass");
    let ui = compiled
        .stages
        .iter()
        .position(|stage| *stage == RenderPassStage::Ui)
        .expect("pipeline should compile a runtime UI graph pass");
    let overlay = compiled
        .stages
        .iter()
        .position(|stage| *stage == RenderPassStage::Overlay)
        .expect("pipeline should compile an overlay graph stage");
    let debug = compiled
        .stages
        .iter()
        .position(|stage| *stage == RenderPassStage::Debug)
        .expect("pipeline should compile a debug overlay graph stage");

    assert!(postprocess < overlay && overlay < ui);
    assert!(postprocess < debug && debug < ui);
    assert_pass_before(compiled, "uber", "overlay-gizmo");
    assert_pass_before(compiled, "overlay-gizmo", "runtime-ui");
    assert_eq!(
        compiled
            .graph()
            .passes()
            .last()
            .map(|pass| pass.name.as_str()),
        Some("runtime-ui")
    );
}

fn assert_pass_before(compiled: &CompiledRenderPipeline, earlier: &str, later: &str) {
    let earlier_index = compiled
        .graph()
        .passes()
        .iter()
        .position(|pass| pass.name == earlier)
        .unwrap_or_else(|| panic!("compiled pipeline should include {earlier}"));
    let later_index = compiled
        .graph()
        .passes()
        .iter()
        .position(|pass| pass.name == later)
        .unwrap_or_else(|| panic!("compiled pipeline should include {later}"));
    assert!(earlier_index < later_index);
}

fn perspective_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(700),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Perspective,
        ),
    )
}

fn perspective_extract_with_overlay_lattice() -> RenderFrameExtract {
    let mut extract = perspective_extract();
    extract.debug.overlays.scene_gizmos = vec![SceneGizmoOverlayExtract::new(
        9000,
        SceneGizmoKind::Camera,
        true,
        dense_green_overlay_lines(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )];
    extract
}

fn orthographic_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(701),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Orthographic,
        ),
    )
}

fn runtime_ui_extract_with_image_and_clip() -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.product"),
        list: UiRenderList {
            commands: vec![
                UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(8.0, 8.0, 180.0, 28.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#1b2330cc".to_string()),
                        foreground_color: Some("#f5f7fb".to_string()),
                        font_size: 14.0,
                        line_height: 18.0,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::None,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Runtime HUD".to_string()),
                    image: None,
                    opacity: 1.0,
                },
                UiRenderCommand {
                    node_id: UiNodeId::new(2),
                    kind: UiRenderCommandKind::Image,
                    frame: UiFrame::new(20.0, 48.0, 32.0, 32.0),
                    clip_frame: Some(UiFrame::new(16.0, 44.0, 40.0, 40.0)),
                    z_index: 1,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: Some(UiVisualAssetRef::Image(
                        "res://ui/runtime/hud-icon.png".to_string(),
                    )),
                    opacity: 1.0,
                },
            ],
        },
    }
}

fn runtime_ui_extract_with_center_quad() -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("runtime.ui.overlay.product"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(11),
                kind: UiRenderCommandKind::Quad,
                frame: UiFrame::new(96.0, 72.0, 128.0, 96.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    background_color: Some("#08d8ff".to_string()),
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: None,
                image: None,
                opacity: 1.0,
            }],
        },
    }
}

fn dense_green_overlay_lines() -> Vec<OverlayLineSegment> {
    (0..=120)
        .map(|index| {
            let y = -1.2 + index as f32 * 0.02;
            OverlayLineSegment {
                start: Vec3::new(-2.2, y, -2.0),
                end: Vec3::new(2.2, y, -2.0),
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
            }
        })
        .collect()
}

fn srgb_render_target_texture_asset(uri: AssetUri, size: UVec2) -> TextureAsset {
    TextureAsset::new_rgba8(uri, size.x, size.y, vec![0; (size.x * size.y * 4) as usize])
        .with_descriptor(srgb_render_target_texture_descriptor())
}

fn srgb_render_target_texture_descriptor() -> TextureAssetDescriptor {
    TextureAssetDescriptor {
        format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Srgb,
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![
            RenderImageUsage::RenderTarget,
            RenderImageUsage::Sampled,
            RenderImageUsage::CopySrc,
        ],
        fallback: RenderImageFallbackKind::MissingImage,
        ..TextureAssetDescriptor::default()
    }
}

fn dominant_cyan_pixels_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> usize {
    dominant_pixels_in_region(frame, origin, size, is_dominant_cyan)
}

fn dominant_green_pixels_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> usize {
    dominant_pixels_in_region(frame, origin, size, is_dominant_green)
}

fn dominant_pixels_in_region(
    frame: &CapturedFrame,
    origin: UVec2,
    size: UVec2,
    predicate: impl Fn(&[u8]) -> bool,
) -> usize {
    let width = frame.width as usize;
    let x_end = origin.x.saturating_add(size.x).min(frame.width) as usize;
    let y_end = origin.y.saturating_add(size.y).min(frame.height) as usize;
    let mut count = 0;
    for y in origin.y as usize..y_end {
        for x in origin.x as usize..x_end {
            let index = (y * width + x) * 4;
            if predicate(&frame.rgba[index..index + 4]) {
                count += 1;
            }
        }
    }
    count
}

fn is_dominant_cyan(pixel: &[u8]) -> bool {
    pixel[3] == 255 && pixel[0] < 96 && pixel[1] > 120 && pixel[2] > 160
}

fn is_dominant_green(pixel: &[u8]) -> bool {
    let red = u16::from(pixel[0]);
    let green = u16::from(pixel[1]);
    let blue = u16::from(pixel[2]);
    pixel[3] == 255 && green > 120 && green > red + 40 && green > blue + 40
}
