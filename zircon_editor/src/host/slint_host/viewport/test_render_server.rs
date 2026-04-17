use zircon_render_server::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderServer, RenderServerError,
    RenderStats, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_scene::RenderFrameExtract;

pub(super) struct TestRenderServer;

impl RenderServer for TestRenderServer {
    fn create_viewport(
        &self,
        _descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderServerError> {
        Ok(RenderViewportHandle::new(1))
    }

    fn destroy_viewport(&self, _viewport: RenderViewportHandle) -> Result<(), RenderServerError> {
        Ok(())
    }

    fn submit_frame_extract(
        &self,
        _viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
    ) -> Result<(), RenderServerError> {
        Ok(())
    }

    fn set_pipeline_asset(
        &self,
        _viewport: RenderViewportHandle,
        _pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderServerError> {
        Ok(())
    }

    fn reload_pipeline(&self, _pipeline: RenderPipelineHandle) -> Result<(), RenderServerError> {
        Ok(())
    }

    fn query_stats(&self) -> Result<RenderStats, RenderServerError> {
        Ok(RenderStats::default())
    }

    fn capture_frame(
        &self,
        _viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderServerError> {
        Ok(None)
    }

    fn set_quality_profile(
        &self,
        _viewport: RenderViewportHandle,
        _profile: RenderQualityProfile,
    ) -> Result<(), RenderServerError> {
        Ok(())
    }
}
