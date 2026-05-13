use super::{
    CapturedFrame, GraphicsDebuggerStatus, RenderFrameExtract, RenderFrameworkError,
    RenderPipelineHandle, RenderQualityProfile, RenderStats, RenderViewportDescriptor,
    RenderViewportHandle, RenderViewportSurfaceDescriptor, RenderVirtualGeometryDebugSnapshot,
};
use crate::graphics::RenderPipelineAsset;
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

    fn set_pipeline_asset(
        &self,
        viewport: RenderViewportHandle,
        pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderFrameworkError>;

    fn register_pipeline_asset(
        &self,
        pipeline: RenderPipelineAsset,
    ) -> Result<RenderPipelineHandle, RenderFrameworkError>;

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

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderFrameworkError>;
}
