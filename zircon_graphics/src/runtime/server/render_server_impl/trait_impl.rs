use zircon_render_server::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderServer, RenderServerError,
    RenderStats, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_scene::RenderFrameExtract;

use super::super::capture_frame::capture_frame;
use super::super::create_viewport::create_viewport;
use super::super::destroy_viewport::destroy_viewport;
use super::super::query_stats::query_stats;
use super::super::reload_pipeline::reload_pipeline;
use super::super::set_pipeline_asset::set_pipeline_asset;
use super::super::set_quality_profile::set_quality_profile;
use super::super::submit_frame_extract::submit_frame_extract;
use super::super::wgpu_render_server::WgpuRenderServer;

impl RenderServer for WgpuRenderServer {
    fn create_viewport(
        &self,
        descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderServerError> {
        create_viewport(self, descriptor)
    }

    fn destroy_viewport(&self, viewport: RenderViewportHandle) -> Result<(), RenderServerError> {
        destroy_viewport(self, viewport)
    }

    fn submit_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        extract: RenderFrameExtract,
    ) -> Result<(), RenderServerError> {
        submit_frame_extract(self, viewport, extract)
    }

    fn set_pipeline_asset(
        &self,
        viewport: RenderViewportHandle,
        pipeline: RenderPipelineHandle,
    ) -> Result<(), RenderServerError> {
        set_pipeline_asset(self, viewport, pipeline)
    }

    fn reload_pipeline(&self, pipeline: RenderPipelineHandle) -> Result<(), RenderServerError> {
        reload_pipeline(self, pipeline)
    }

    fn query_stats(&self) -> Result<RenderStats, RenderServerError> {
        query_stats(self)
    }

    fn capture_frame(
        &self,
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderServerError> {
        capture_frame(self, viewport)
    }

    fn set_quality_profile(
        &self,
        viewport: RenderViewportHandle,
        profile: RenderQualityProfile,
    ) -> Result<(), RenderServerError> {
        set_quality_profile(self, viewport, profile)
    }
}
