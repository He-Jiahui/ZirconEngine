use super::{
    CapturedFrame, GraphicsDebuggerStatus, RenderFrameExtract, RenderFrameworkError,
    RenderPipelineHandle, RenderQualityProfile, RenderStats, RenderSubmissionConfig,
    RenderViewportDescriptor, RenderViewportHandle, RenderViewportSurfaceDescriptor,
    RenderVirtualGeometryDebugSnapshot,
};
use zircon_runtime_interface::ui::surface::UiRenderExtract;

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

    fn capture_frame_if_newer(
        &self,
        viewport: RenderViewportHandle,
        last_generation: Option<u64>,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        Ok(self
            .capture_frame(viewport)?
            .filter(|frame| Some(frame.generation) != last_generation))
    }

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError>;
}
