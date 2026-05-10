use crate::scene::viewport::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError, RenderPipelineHandle,
    RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_runtime::graphics::RenderPipelineAsset;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

pub(super) struct TestRenderFramework;

impl RenderFramework for TestRenderFramework {
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
        pipeline: RenderPipelineAsset,
    ) -> Result<RenderPipelineHandle, RenderFrameworkError> {
        Ok(pipeline.handle)
    }

    fn reload_pipeline(&self, _pipeline: RenderPipelineHandle) -> Result<(), RenderFrameworkError> {
        Ok(())
    }

    fn query_stats(&self) -> Result<RenderStats, RenderFrameworkError> {
        Ok(RenderStats::default())
    }

    fn query_virtual_geometry_debug_snapshot(
        &self,
    ) -> Result<
        Option<zircon_runtime::core::framework::render::RenderVirtualGeometryDebugSnapshot>,
        RenderFrameworkError,
    > {
        Ok(None)
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
