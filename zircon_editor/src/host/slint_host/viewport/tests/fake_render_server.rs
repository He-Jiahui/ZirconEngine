use std::collections::HashMap;
use std::sync::Mutex;

use zircon_math::UVec2;
use zircon_render_server::{
    CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderServer, RenderServerError,
    RenderStats, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_scene::RenderFrameExtract;

#[derive(Default)]
pub(super) struct FakeRenderServer {
    pub(super) state: Mutex<FakeRenderServerState>,
}

#[derive(Default)]
pub(super) struct FakeRenderServerState {
    pub(super) next_viewport_id: u64,
    pub(super) created_viewports: Vec<RenderViewportDescriptor>,
    pub(super) viewport_sizes: HashMap<RenderViewportHandle, UVec2>,
    pub(super) destroyed_viewports: Vec<RenderViewportHandle>,
    pub(super) submitted_viewports: Vec<RenderViewportHandle>,
    pub(super) submitted_aspect_ratios: Vec<f32>,
    pub(super) capture_requests: usize,
    pub(super) captures: HashMap<RenderViewportHandle, CapturedFrame>,
}

impl RenderServer for FakeRenderServer {
    fn create_viewport(
        &self,
        descriptor: RenderViewportDescriptor,
    ) -> Result<RenderViewportHandle, RenderServerError> {
        let mut state = self.state.lock().unwrap();
        state.next_viewport_id += 1;
        let handle = RenderViewportHandle::new(state.next_viewport_id);
        state.viewport_sizes.insert(handle, descriptor.size);
        state.created_viewports.push(descriptor);
        Ok(handle)
    }

    fn destroy_viewport(&self, viewport: RenderViewportHandle) -> Result<(), RenderServerError> {
        self.state
            .lock()
            .unwrap()
            .destroyed_viewports
            .push(viewport);
        Ok(())
    }

    fn submit_frame_extract(
        &self,
        viewport: RenderViewportHandle,
        _extract: RenderFrameExtract,
    ) -> Result<(), RenderServerError> {
        let mut state = self.state.lock().unwrap();
        state.submitted_viewports.push(viewport);
        let size = state
            .viewport_sizes
            .get(&viewport)
            .copied()
            .unwrap_or(UVec2::new(1, 1));
        state
            .submitted_aspect_ratios
            .push(size.x as f32 / size.y as f32);
        state.captures.insert(
            viewport,
            CapturedFrame::new(1, 1, vec![viewport.raw() as u8, 0, 0, 255], viewport.raw()),
        );
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
        viewport: RenderViewportHandle,
    ) -> Result<Option<CapturedFrame>, RenderServerError> {
        let mut state = self.state.lock().unwrap();
        state.capture_requests += 1;
        Ok(state.captures.get(&viewport).cloned())
    }

    fn set_quality_profile(
        &self,
        _viewport: RenderViewportHandle,
        _profile: RenderQualityProfile,
    ) -> Result<(), RenderServerError> {
        Ok(())
    }
}
