use super::{
    CapturedFrame, CapturedHdrFrame, GraphicsDebuggerStatus, RenderFrameExtract,
    RenderFrameworkError, RenderPipelineHandle, RenderQualityProfile, RenderStats,
    RenderSubmissionConfig, RenderViewportDescriptor, RenderViewportHandle, RenderViewportProduct,
    RenderViewportSurfaceDescriptor, RenderVirtualGeometryDebugSnapshot,
    RenderVisibleSpatialQuerySnapshot,
};
use zircon_runtime_interface::ui::surface::UiRenderExtract;
use zr_rhi::{UiSurfaceDescriptor, UiSurfacePresenter};

pub trait RenderFramework: Send + Sync {
    fn create_viewport(
        &self,
        descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderFrameworkError>;

    fn destroy_viewport(&self, viewport: RenderViewportHandle) -> Result<(), RenderFrameworkError>;

    fn submit_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError>;

    fn submit_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError>;

    fn set_submission_config(
        &self,
        _config: RenderSubmissionConfig,
    ) -> Result<(), RenderFrameworkError> {
        Err(RenderFrameworkError::UnsupportedCapability {
            capability: "render submission configuration".to_string(),
        })
    }

    fn submission_config(&self) -> RenderSubmissionConfig {
        RenderSubmissionConfig::default()
    }

    fn bind_viewport_surface(
        &self,
        _viewport: RenderViewportHandle,
        _descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<(), RenderFrameworkError> {
        Err(RenderFrameworkError::UnsupportedCapability {
            capability: "viewport surface present".to_string(),
        })
    }

    fn unbind_viewport_surface(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn present_frame_extract(
        &self,
        _viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
    ) -> Result<(), RenderFrameworkError> {
        Err(RenderFrameworkError::UnsupportedCapability {
            capability: "viewport surface present".to_string(),
        })
    }

    fn present_frame_extract_with_ui(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
        _ui: Option<UiRenderExtract>,
    ) -> Result<(), RenderFrameworkError> {
        self.present_frame_extract(viewport, extract)
    }

    fn set_pipeline_asset(
        &self,
        viewport: RenderViewportHandle,
        pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError>;

    fn reload_pipeline(&self, pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError>;

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError>;

    fn query_visible_spatial_snapshot(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<Option<RenderVisibleSpatialQuerySnapshot>, RenderFrameworkError> {
        Err(RenderFrameworkError::UnsupportedCapability {
            capability: "renderer-visible spatial query".to_string(),
        })
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<Option<RenderVirtualGeometryDebugSnapshot>, RenderFrameworkError>;

    fn request_graphics_debugger_capture(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn query_graphics_debugger_status(
        &self,
    ) -> Result<GraphicsDebuggerStatus, RenderFrameworkError> {
        Ok(GraphicsDebuggerStatus::unavailable("unimplemented"))
    }

    fn capture_frame(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError>;

    /// Reads the retained, linear HDR scene-color product of a completed frame.
    fn capture_scene_color_hdr(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedHdrFrame>, RenderFrameworkError> {
        Err(RenderFrameworkError::UnsupportedCapability {
            capability: "linear HDR scene-color capture".to_string(),
        })
    }

    fn capture_frame_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        Ok(self
            .capture_frame(viewport)?
            .filter(|frame| Some(frame.generation) != last_generation))
    }

    /// Returns only a capture that has already completed without waiting for GPU work.
    fn poll_captured_frame_if_newer(
        &self,
        _viewport: RenderViewportHandle,
        _last_generation: Option<u64>,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        Ok(None)
    }

    /// Returns an already-produced GPU presentation identity without requesting readback.
    fn poll_viewport_product_if_newer(
        &self,
        _viewport: RenderViewportHandle,
        _last_generation: Option<u64>,
    ) -> Result<Option<RenderViewportProduct>, RenderFrameworkError> {
        Ok(None)
    }

    /// Creates a native UI presenter that can directly sample products from this backend.
    fn create_ui_surface_presenter(
        &self,
        _descriptor: UiSurfaceDescriptor,
    ) -> Result<Box<dyn UiSurfacePresenter>, RenderFrameworkError> {
        Err(RenderFrameworkError::UnsupportedCapability {
            capability: "shared GPU UI surface presentation".to_string(),
        })
    }

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_capture_poll_is_explicitly_nonblocking() {
        let source = include_str!("framework.rs");
        let poll_source = source
            .split("fn poll_captured_frame_if_newer")
            .nth(1)
            .expect("render framework declares a capture polling contract");

        assert!(poll_source.contains("Ok(None)"));
        assert!(!poll_source.contains("capture_frame_if_newer"));
    }
}
