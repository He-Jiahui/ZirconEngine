use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AssetUri, TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
};
use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, CapturedFrame, FallbackSkyboxKind,
    GraphicsDebuggerStatus, PreviewEnvironmentExtract, RenderCameraClear, RenderCameraTarget,
    RenderCameraTargetKind, RenderCaptureSource, RenderFrameExtract, RenderFramework,
    RenderFrameworkError, RenderImageColorSpace, RenderImageFallbackKind, RenderImageUsage,
    RenderNativeSurfaceTarget, RenderPipelineHandle, RenderQualityProfile, RenderSamplerDescriptor,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderViewportHandle, RenderViewportSurfaceDescriptor, RenderVirtualGeometryDebugSnapshot,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::{RenderPipelineAsset, WgpuRenderFramework};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

const CAMERA_TEXTURE_TARGET_ASSET_CAPABILITY: &str = "camera texture render target asset";
const CAMERA_TEXTURE_SURFACE_PRESENT_CAPABILITY: &str = "camera texture surface present";
const CAMERA_TEXTURE_TARGET_FORMAT_CAPABILITY: &str = "camera texture render target format";
const CAMERA_TEXTURE_TARGET_USAGE_CAPABILITY: &str = "camera texture render target usage";
const HEADLESS_CAMERA_SURFACE_PRESENT_CAPABILITY: &str = "headless camera surface present";
const SURFACE_PRESENT_CAPABILITY: &str = "viewport surface present";

mod texture_target;

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
