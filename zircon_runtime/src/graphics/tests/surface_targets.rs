use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, GraphicsDebuggerStatus, PreviewEnvironmentExtract,
    RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderNativeSurfaceTarget,
    RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderStats, RenderViewportDescriptor, RenderViewportHandle, RenderViewportSurfaceDescriptor,
    RenderVirtualGeometryDebugSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::graphics::WgpuRenderFramework;
use crate::RenderPipelineAsset;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

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
        frame.generation,
        framework.query_stats().unwrap().last_generation.unwrap()
    );
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
