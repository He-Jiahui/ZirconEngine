use crate::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderServerError, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_scene::RenderFrameExtract;

pub trait RenderServer: Send + Sync {
    fn create_viewport(
        &self,
        descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderServerError>;

    fn destroy_viewport(&self, viewport: RenderViewportHandle) -> Result<(), RenderServerError>;

    fn submit_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<(), RenderServerError>;

    fn set_pipeline_asset(
        &self,
        viewport: RenderViewportHandle,
        pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderServerError>;

    fn reload_pipeline(&self, pipeline: RenderPipelineHandle) -> Result<(), RenderServerError>;

    fn query_stats(&self) -> Result<RenderStats, RenderServerError>;

    fn capture_frame(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderServerError>;

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderServerError>;
}
