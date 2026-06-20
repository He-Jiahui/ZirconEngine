use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AssetUri, TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
};
use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, CapturedFrame, FallbackSkyboxKind,
    GraphicsDebuggerStatus, PreviewEnvironmentExtract, RenderCameraClear, RenderCameraTarget,
    RenderCameraTargetGraphImportStatus, RenderCameraTargetKind, RenderCameraTargetWritebackStatus,
    RenderCaptureSource, RenderFrameExtract, RenderFramework, RenderFrameworkError,
    RenderImageColorSpace, RenderImageFallbackKind, RenderImageUsage, RenderNativeSurfaceTarget,
    RenderPipelineHandle, RenderQualityProfile, RenderSamplerDescriptor,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderViewportHandle, RenderViewportRect, RenderViewportSurfaceDescriptor,
    RenderVirtualGeometryDebugSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::{RenderPipelineAsset, WgpuRenderFramework};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

const CAMERA_TEXTURE_TARGET_ASSET_CAPABILITY: &str = "camera texture render target asset";
const CAMERA_TEXTURE_SURFACE_PRESENT_CAPABILITY: &str = "camera texture surface present";
const CAMERA_TEXTURE_TARGET_FORMAT_CAPABILITY: &str = "camera texture render target format";
const CAMERA_TEXTURE_TARGET_USAGE_CAPABILITY: &str = "camera texture render target usage";
const HEADLESS_CAMERA_SURFACE_PRESENT_CAPABILITY: &str = "headless camera surface present";
const SURFACE_PRESENT_CAPABILITY: &str = "viewport surface present";

#[test]
fn graphics_surface_default_contract_reports_unsupported_present_and_noop_unbind() {
    let framework = UnsupportedSurfaceFramework;
    let viewport = RenderViewportHandle::new(7);

    assert_eq!(
        framework
            .bind_viewport_surface(viewport, win32_surface_descriptor())
            .unwrap_err(),
        unsupported_surface_present()
    );
    framework.unbind_viewport_surface(viewport).unwrap();
    assert_eq!(
        framework
            .present_frame_extract(viewport, empty_extract())
            .unwrap_err(),
        unsupported_surface_present()
    );
}

#[test]
fn graphics_surface_bind_rejects_unknown_viewport_before_native_surface_creation() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();

    let error = framework
        .bind_viewport_surface(RenderViewportHandle::new(404), win32_surface_descriptor())
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::UnknownViewport { viewport: 404 }
    );
}

#[test]
fn graphics_surface_unbind_rejects_unknown_viewport() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();

    let error = framework
        .unbind_viewport_surface(RenderViewportHandle::new(404))
        .unwrap_err();

    assert_eq!(
        error,
        RenderFrameworkError::UnknownViewport { viewport: 404 }
    );
}

#[test]
fn graphics_surface_present_without_bound_surface_reports_unsupported() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .present_frame_extract(viewport, empty_extract())
        .unwrap_err();

    assert_eq!(error, unsupported_surface_present());
}

#[test]
fn graphics_surface_missing_surface_clears_pending_graphics_debugger_capture() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();
    framework
        .request_graphics_debugger_capture(viewport)
        .unwrap();

    let error = framework
        .present_frame_extract(viewport, empty_extract())
        .unwrap_err();

    assert_eq!(error, unsupported_surface_present());
    let status = framework.query_graphics_debugger_status().unwrap();
    assert!(!status.active_capture);
    assert!(!status.capture_pending);
    assert_eq!(status.last_capture_frame, None);
    assert!(status
        .last_error
        .as_deref()
        .is_some_and(|message| message.contains(SURFACE_PRESENT_CAPABILITY)));
    assert_eq!(framework.query_stats().unwrap().captured_frames, 0);
}

#[test]
fn graphics_surface_offscreen_submit_and_capture_survive_unbind_noop() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(64, 48);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework.unbind_viewport_surface(viewport).unwrap();
    framework
        .submit_frame_extract(viewport, empty_extract())
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();

    assert_eq!(frame.width, viewport_size.x);
    assert_eq!(frame.height, viewport_size.y);
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(frame.capture_report.output_size, viewport_size);
    assert_eq!(
        frame.generation,
        framework.query_stats().unwrap().last_generation.unwrap()
    );
    let target_resolution = framework
        .query_stats()
        .unwrap()
        .last_camera_target_resolution;
    assert_eq!(
        target_resolution.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(target_resolution.primary_target_size, viewport_size);
    assert_eq!(target_resolution.resolved_target_size, viewport_size);
    assert_eq!(target_resolution.effective_view_size, viewport_size);
    assert_eq!(target_resolution.effective_render_size, viewport_size);
}

#[test]
fn graphics_camera_target_headless_size_controls_offscreen_capture_size() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();
    let headless_size = UVec2::new(40, 24);

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Headless {
                size: headless_size,
            }),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();

    assert_eq!(frame.width, headless_size.x);
    assert_eq!(frame.height, headless_size.y);
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::Headless
    );
    assert_eq!(frame.capture_report.output_size, headless_size);
    let target_resolution = framework
        .query_stats()
        .unwrap()
        .last_camera_target_resolution;
    assert_eq!(
        target_resolution.target_kind,
        RenderCameraTargetKind::Headless
    );
    assert_eq!(target_resolution.primary_target_size, UVec2::new(64, 48));
    assert_eq!(target_resolution.resolved_target_size, headless_size);
    assert_eq!(target_resolution.effective_view_size, headless_size);
    assert_eq!(target_resolution.effective_render_size, headless_size);
}

#[test]
fn graphics_camera_target_texture_missing_asset_reports_unsupported_without_primary_fallback_capture(
) {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-target/texture",
    ));

    let error = framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(texture)),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_target());
    assert_eq!(framework.capture_frame(viewport).unwrap(), None);
    assert_eq!(framework.query_stats().unwrap().submitted_frames, 0);
}

#[test]
fn graphics_camera_target_texture_requires_render_target_usage() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = AssetUri::parse("res://tests/camera-target/sampled.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            TextureAsset::new_rgba8(texture_uri, 72, 40, vec![0; 72 * 40 * 4]),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_target_usage());
    assert_eq!(framework.capture_frame(viewport).unwrap(), None);
    assert_eq!(framework.query_stats().unwrap().submitted_frames, 0);
}

#[test]
fn graphics_camera_target_texture_requires_renderable_render_target_format() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri =
        AssetUri::parse("res://tests/camera-target/compressed-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            render_target_texture_asset(texture_uri, 72, 40).with_descriptor(
                TextureAssetDescriptor {
                    format: "dds/dxt1".to_string(),
                    ..render_target_texture_descriptor()
                },
            ),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_target_format());
    assert_eq!(framework.capture_frame(viewport).unwrap(), None);
    assert_eq!(framework.query_stats().unwrap().submitted_frames, 0);
}

#[test]
fn graphics_camera_target_texture_render_target_metadata_controls_offscreen_capture_size() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = AssetUri::parse("res://tests/camera-target/render-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            render_target_texture_asset(texture_uri, 72, 40),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();

    assert_eq!(frame.width, 72);
    assert_eq!(frame.height, 40);
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureWritebackConversion
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::Converted
    );
    let target_resolution = framework
        .query_stats()
        .unwrap()
        .last_camera_target_resolution;
    assert_eq!(
        target_resolution.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(target_resolution.primary_target_size, UVec2::new(64, 48));
    assert_eq!(target_resolution.resolved_target_size, UVec2::new(72, 40));
    assert_eq!(target_resolution.effective_view_size, UVec2::new(72, 40));
    assert_eq!(target_resolution.effective_render_size, UVec2::new(72, 40));
    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.submitted_frames, 1);
    assert_eq!(
        stats.last_capture_report.source,
        RenderCaptureSource::TextureWritebackConversion
    );
    assert_eq!(stats.last_capture_report.output_size, UVec2::new(72, 40));
    assert_eq!(
        stats.last_camera_target_writeback.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::Converted
    );
    assert_eq!(
        stats.last_camera_target_writeback.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_writeback.copied_count, 0);
    assert_eq!(stats.last_camera_target_writeback.converted_count, 1);
    assert!(!stats.last_camera_target_writeback.debug_marker_emitted);
    assert_eq!(
        stats.last_camera_target_graph_import.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        stats.last_camera_target_graph_import.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_graph_import.direct_import_count, 0);
    assert_eq!(
        stats
            .last_camera_target_graph_import
            .conversion_writeback_count,
        1
    );
}

#[test]
fn graphics_camera_target_texture_srgb_target_imports_direct_graph_final_target() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri =
        AssetUri::parse("res://tests/camera-target/srgb-render-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            srgb_render_target_texture_asset(texture_uri, 72, 40),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(frame.width, 72);
    assert_eq!(frame.height, 40);
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(stats.submitted_frames, 1);
    assert_eq!(
        stats.last_capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(stats.last_capture_report.output_size, UVec2::new(72, 40));
    assert_eq!(
        stats.last_camera_target_writeback.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(
        stats.last_camera_target_writeback.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_writeback.copied_count, 0);
    assert_eq!(stats.last_camera_target_writeback.converted_count, 0);
    assert!(!stats.last_camera_target_writeback.debug_marker_emitted);
    assert_eq!(
        stats.last_camera_target_graph_import.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        stats.last_camera_target_graph_import.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_graph_import.direct_import_count, 1);
    assert_eq!(
        stats
            .last_camera_target_graph_import
            .conversion_writeback_count,
        0
    );
}

#[test]
fn graphics_camera_target_texture_overlay_stack_preserves_base_composite() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri =
        AssetUri::parse("res://tests/camera-target/overlay-stack-composite.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            srgb_render_target_texture_asset(texture_uri, 64, 48),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_cameras(vec![
                texture_base_camera(texture_id, Vec4::new(1.0, 0.0, 0.0, 1.0)).with_stack([2]),
                texture_overlay_camera(texture_id),
            ]),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert!(
        dominant_red_pixels(&frame.rgba) > ((frame.width * frame.height) as usize * 9 / 10),
        "base camera clear must survive overlay stack load into the texture final target"
    );
}

#[test]
fn graphics_camera_target_texture_base_stacks_write_independent_texture_targets() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let red_texture_uri =
        AssetUri::parse("res://tests/camera-target/multi-target-red.texture").unwrap();
    let green_texture_uri =
        AssetUri::parse("res://tests/camera-target/multi-target-green.texture").unwrap();
    let red_texture_id = ResourceId::from_locator(&red_texture_uri);
    let green_texture_id = ResourceId::from_locator(&green_texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                red_texture_id,
                ResourceKind::Texture,
                red_texture_uri.clone(),
            ),
            srgb_render_target_texture_asset(red_texture_uri, 64, 48),
        )
        .expect("red texture insert");
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                green_texture_id,
                ResourceKind::Texture,
                green_texture_uri.clone(),
            ),
            srgb_render_target_texture_asset(green_texture_uri, 64, 48),
        )
        .expect("green texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_cameras(vec![
                texture_base_camera_with_entity(
                    1,
                    0,
                    red_texture_id,
                    Vec4::new(1.0, 0.0, 0.0, 1.0),
                ),
                texture_base_camera_with_entity(
                    2,
                    1,
                    green_texture_id,
                    Vec4::new(0.0, 1.0, 0.0, 1.0),
                ),
            ]),
        )
        .unwrap();

    let (red_size, red_rgba) = framework
        .read_output_target_texture_rgba_for_tests(red_texture_id)
        .unwrap()
        .expect("red target should stay prepared after submit");
    let (green_size, green_rgba) = framework
        .read_output_target_texture_rgba_for_tests(green_texture_id)
        .unwrap()
        .expect("green target should stay prepared after submit");
    let frame = framework.capture_frame(viewport).unwrap().unwrap();

    assert_eq!(red_size, UVec2::new(64, 48));
    assert_eq!(green_size, UVec2::new(64, 48));
    let target_pixels = (64 * 48) as usize;
    let red_target_red = dominant_red_pixels(&red_rgba);
    let red_target_green = dominant_green_pixels(&red_rgba);
    let green_target_red = dominant_red_pixels(&green_rgba);
    let green_target_green = dominant_green_pixels(&green_rgba);

    assert!(
        red_target_red > target_pixels * 9 / 10,
        "first texture Base stack should write red pixels into its own final target; red={red_target_red}, green={red_target_green}, total={target_pixels}"
    );
    assert!(
        red_target_green < target_pixels / 20,
        "second texture Base stack should not overwrite the first texture target; green={red_target_green}, total={target_pixels}"
    );
    assert!(
        green_target_green > target_pixels * 9 / 10,
        "second texture Base stack should write green pixels into its own final target; green={green_target_green}, red={green_target_red}, total={target_pixels}"
    );
    assert!(
        green_target_red < target_pixels / 20,
        "first texture Base stack should not leak into the second texture target; red={green_target_red}, total={target_pixels}"
    );
    assert!(
        dominant_green_pixels(&frame.rgba) > target_pixels * 9 / 10,
        "viewport capture remains owned by the last texture Base stack"
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
}

#[test]
fn graphics_primary_surface_split_screen_base_cameras_clear_only_their_viewport_regions() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport_size = UVec2::new(64, 48);
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let left_half = RenderViewportRect::new(UVec2::ZERO, UVec2::new(32, 48));
    let right_half = RenderViewportRect::new(UVec2::new(32, 0), UVec2::new(32, 48));

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_cameras(vec![
                primary_base_camera(1, 0, left_half, Vec4::new(1.0, 0.0, 0.0, 1.0)),
                primary_base_camera(2, 1, right_half, Vec4::new(0.0, 1.0, 0.0, 1.0)),
            ]),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();

    assert_eq!(frame.width, viewport_size.x);
    assert_eq!(frame.height, viewport_size.y);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );

    let left_inset_origin = UVec2::new(4, 4);
    let right_inset_origin = UVec2::new(36, 4);
    let inset_size = UVec2::new(24, 40);
    let inset_pixels = (inset_size.x * inset_size.y) as usize;
    let left_red = dominant_red_pixels_in_region(&frame, left_inset_origin, inset_size);
    let left_green = dominant_green_pixels_in_region(&frame, left_inset_origin, inset_size);
    let right_red = dominant_red_pixels_in_region(&frame, right_inset_origin, inset_size);
    let right_green = dominant_green_pixels_in_region(&frame, right_inset_origin, inset_size);
    let left_center = pixel_at(&frame, UVec2::new(16, 24));
    let right_center = pixel_at(&frame, UVec2::new(48, 24));

    assert!(
        left_red > inset_pixels * 9 / 10,
        "left Base camera should clear only the left viewport red; left_red={left_red}, left_green={left_green}, right_red={right_red}, right_green={right_green}, left_center={left_center:?}, right_center={right_center:?}, total={inset_pixels}"
    );
    assert!(
        left_green < inset_pixels / 20,
        "right Base camera green clear should not leak into the left viewport; green={left_green}, total={inset_pixels}"
    );
    assert!(
        right_green > inset_pixels * 9 / 10,
        "right Base camera should clear only the right viewport green; green={right_green}, total={inset_pixels}"
    );
    assert!(
        right_red < inset_pixels / 20,
        "left Base camera red clear should not leak into the right viewport; red={right_red}, total={inset_pixels}"
    );
}

#[test]
fn graphics_camera_target_texture_present_reports_unsupported_surface_fallback() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = AssetUri::parse("res://tests/camera-target/present-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            render_target_texture_asset(texture_uri, 72, 40),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .present_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_surface_present());
}

#[test]
fn graphics_camera_target_headless_present_reports_unsupported_surface_fallback() {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .present_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Headless {
                size: UVec2::new(40, 24),
            }),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_headless_camera_surface_present());
}

#[test]
fn graphics_surface_present_path_source_uses_swapchain_present_without_readback_fallback() {
    let framework_present_source = include_str!(
        "../runtime/render_framework/submit_frame_extract/submit/present_frame_extract.rs"
    );
    let backend_surface_source = include_str!("../backend/render_backend/viewport_surface.rs");

    assert!(framework_present_source.contains("record_present_submission"));
    assert!(backend_surface_source.contains("surface_texture.present()"));
    assert!(!framework_present_source.contains("capture_frame"));
    assert!(!framework_present_source.contains("read_texture_rgba"));
    assert!(!backend_surface_source.contains("read_texture_rgba"));
}

fn win32_surface_descriptor() -> RenderViewportSurfaceDescriptor {
    RenderViewportSurfaceDescriptor::new(
        UVec2::new(64, 48),
        RenderNativeSurfaceTarget::Win32 {
            hwnd: 1,
            hinstance: Some(2),
        },
    )
}

fn unsupported_surface_present() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: SURFACE_PRESENT_CAPABILITY.to_string(),
    }
}

fn unsupported_camera_texture_target() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: CAMERA_TEXTURE_TARGET_ASSET_CAPABILITY.to_string(),
    }
}

fn unsupported_camera_texture_surface_present() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: CAMERA_TEXTURE_SURFACE_PRESENT_CAPABILITY.to_string(),
    }
}

fn unsupported_camera_texture_target_format() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: CAMERA_TEXTURE_TARGET_FORMAT_CAPABILITY.to_string(),
    }
}

fn unsupported_camera_texture_target_usage() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: CAMERA_TEXTURE_TARGET_USAGE_CAPABILITY.to_string(),
    }
}

fn unsupported_headless_camera_surface_present() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: HEADLESS_CAMERA_SURFACE_PRESENT_CAPABILITY.to_string(),
    }
}

fn empty_extract_with_target(target: RenderCameraTarget) -> RenderFrameExtract {
    let mut extract = empty_extract();
    extract
        .view
        .selected_camera_descriptor_mut()
        .expect("test extract should carry a selected camera descriptor")
        .target = target;
    extract
}

fn empty_extract_with_cameras(cameras: Vec<CameraRenderDescriptor>) -> RenderFrameExtract {
    let mut extract = empty_extract();
    extract.view = extract.view.with_cameras(cameras);
    extract
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
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
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

fn render_target_texture_asset(uri: AssetUri, width: u32, height: u32) -> TextureAsset {
    TextureAsset::new_rgba8(uri, width, height, vec![0; (width * height * 4) as usize])
        .with_descriptor(render_target_texture_descriptor())
}

fn srgb_render_target_texture_asset(uri: AssetUri, width: u32, height: u32) -> TextureAsset {
    TextureAsset::new_rgba8(uri, width, height, vec![0; (width * height * 4) as usize])
        .with_descriptor(srgb_render_target_texture_descriptor())
}

fn render_target_texture_descriptor() -> TextureAssetDescriptor {
    TextureAssetDescriptor {
        format: RGBA8_UNORM_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Linear,
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

fn srgb_render_target_texture_descriptor() -> TextureAssetDescriptor {
    TextureAssetDescriptor {
        format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Srgb,
        ..render_target_texture_descriptor()
    }
}

fn texture_base_camera(texture_id: ResourceId, clear_color: Vec4) -> CameraRenderDescriptor {
    texture_base_camera_with_entity(1, 0, texture_id, clear_color)
}

fn texture_base_camera_with_entity(
    entity: u64,
    render_order: i32,
    texture_id: ResourceId,
    clear_color: Vec4,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_order,
        target: RenderCameraTarget::Texture(ResourceHandle::<TextureMarker>::new(texture_id)),
        clear: RenderCameraClear::Color(clear_color),
        ..CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        )
    }
}

fn texture_overlay_camera(texture_id: ResourceId) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(2),
        render_type: CameraRenderType::Overlay,
        target: RenderCameraTarget::Texture(ResourceHandle::<TextureMarker>::new(texture_id)),
        clear: RenderCameraClear::None,
        clear_depth: false,
        ..CameraRenderDescriptor::from_camera_payload(Some(2), ViewportCameraSnapshot::default())
    }
}

fn primary_base_camera(
    entity: u64,
    render_order: i32,
    viewport_rect: RenderViewportRect,
    clear_color: Vec4,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_order,
        target: RenderCameraTarget::PrimarySurface,
        viewport_rect: Some(viewport_rect),
        clear: RenderCameraClear::Color(clear_color),
        ..CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        )
    }
}

fn dominant_red_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| is_dominant_red(pixel))
        .count()
}

fn dominant_green_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| is_dominant_green(pixel))
        .count()
}

fn dominant_red_pixels_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> usize {
    dominant_pixels_in_region(frame, origin, size, is_dominant_red)
}

fn dominant_green_pixels_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> usize {
    dominant_pixels_in_region(frame, origin, size, is_dominant_green)
}

fn pixel_at(frame: &CapturedFrame, position: UVec2) -> [u8; 4] {
    let x = position.x.min(frame.width.saturating_sub(1)) as usize;
    let y = position.y.min(frame.height.saturating_sub(1)) as usize;
    let index = (y * frame.width as usize + x) * 4;
    [
        frame.rgba[index],
        frame.rgba[index + 1],
        frame.rgba[index + 2],
        frame.rgba[index + 3],
    ]
}

fn dominant_pixels_in_region(
    frame: &CapturedFrame,
    origin: UVec2,
    size: UVec2,
    predicate: impl Fn(&[u8]) -> bool,
) -> usize {
    let x_end = (origin.x + size.x).min(frame.width) as usize;
    let y_end = (origin.y + size.y).min(frame.height) as usize;
    let width = frame.width as usize;
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

fn is_dominant_red(pixel: &[u8]) -> bool {
    pixel[3] == 255 && pixel[0] > 80 && pixel[0] > pixel[1] + 40 && pixel[0] > pixel[2] + 40
}

fn is_dominant_green(pixel: &[u8]) -> bool {
    pixel[3] == 255 && pixel[1] > 80 && pixel[1] > pixel[0] + 40 && pixel[1] > pixel[2] + 40
}

trait CameraDescriptorTestExt {
    fn with_stack(self, stack: impl IntoIterator<Item = u64>) -> Self;
}

impl CameraDescriptorTestExt for CameraRenderDescriptor {
    fn with_stack(mut self, stack: impl IntoIterator<Item = u64>) -> Self {
        self.stack = stack.into_iter().collect();
        self
    }
}

struct UnsupportedSurfaceFramework;

impl RenderFramework for UnsupportedSurfaceFramework {
    fn create_viewport(
        &self,
        _descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        Ok(RenderViewportHandle::new(1))
    }

    fn destroy_viewport(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn submit_frame_extract(
        &self,
        _viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn submit_frame_extract_with_ui(
        &self,
        _viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
        _ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn set_pipeline_asset(
        &self,
        _viewport: RenderViewportHandle,
        _pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn register_pipeline_asset(
        &self,
        _pipeline: RenderPipelineAsset,
    ) -> Result<RenderPipelineHandle, RenderFrameworkError> {
        Ok(RenderPipelineHandle::new(1))
    }

    fn reload_pipeline(&self, _pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        Ok(RenderStats::default())
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError> {
        Ok(None)
    }

    fn query_graphics_debugger_status(
        &self,
    ) -> Result<GraphicsDebuggerStatus, RenderFrameworkError> {
        Ok(GraphicsDebuggerStatus::unavailable("test"))
    }

    fn capture_frame(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        Ok(None)
    }

    fn set_quality_profile(
        &self,
        _viewport: RenderViewportHandle,
        _profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }
}
