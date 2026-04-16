use std::sync::{Arc, Mutex};

use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use zircon_core::{CoreError, CoreHandle};
use zircon_math::UVec2;
use zircon_render_server::{
    resolve_render_server, CapturedFrame, RenderServer, RenderServerError,
    RenderViewportDescriptor, RenderViewportHandle,
};
#[cfg(test)]
use zircon_render_server::{RenderPipelineHandle, RenderQualityProfile, RenderStats};
use zircon_scene::RenderFrameExtract;

#[derive(Clone)]
pub(crate) struct SlintViewportController {
    shared: Arc<Mutex<ViewportState>>,
}

impl SlintViewportController {
    pub(crate) fn new(core: CoreHandle) -> Result<Self, CoreError> {
        let render_server = resolve_render_server(&core)?;
        Ok(Self::new_with_server(render_server))
    }

    #[cfg(test)]
    pub(crate) fn new_test_stub() -> Self {
        Self::new_with_server(Arc::new(TestRenderServer))
    }

    fn new_with_server(render_server: Arc<dyn RenderServer>) -> Self {
        Self {
            shared: Arc::new(Mutex::new(ViewportState {
                render_server,
                viewport: None,
                latest_generation: None,
                latest_image: None,
                last_error: None,
            })),
        }
    }

    pub(crate) fn submit_extract(
        &self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<(), RenderServerError> {
        let mut shared = self.shared.lock().unwrap();
        let viewport = shared.ensure_viewport(size)?;
        extract.apply_viewport_size(size);
        shared
            .render_server
            .submit_frame_extract(viewport, extract)?;
        shared.last_error = None;
        Ok(())
    }

    pub(crate) fn poll_image(&self) -> Option<Image> {
        let mut shared = self.shared.lock().unwrap();
        let Some(viewport) = shared.viewport.map(|viewport| viewport.handle) else {
            return shared.latest_image.clone();
        };
        match shared.render_server.capture_frame(viewport) {
            Ok(Some(frame)) => {
                if shared.latest_generation == Some(frame.generation) {
                    return shared.latest_image.clone();
                }
                match import_frame_image(&frame) {
                    Ok(image) => {
                        shared.latest_generation = Some(image.0);
                        shared.latest_image = Some(image.1.clone());
                        shared.last_error = None;
                        shared.latest_image.clone()
                    }
                    Err(error) => {
                        shared.last_error = Some(error);
                        shared.latest_image.clone()
                    }
                }
            }
            Ok(None) => shared.latest_image.clone(),
            Err(error) => {
                shared.last_error = Some(error.to_string());
                shared.latest_image.clone()
            }
        }
    }

    pub(crate) fn take_error(&self) -> Option<String> {
        self.shared.lock().unwrap().last_error.take()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveViewport {
    handle: RenderViewportHandle,
    size: UVec2,
}

struct ViewportState {
    render_server: Arc<dyn RenderServer>,
    viewport: Option<ActiveViewport>,
    latest_generation: Option<u64>,
    latest_image: Option<Image>,
    last_error: Option<String>,
}

impl ViewportState {
    fn ensure_viewport(&mut self, size: UVec2) -> Result<RenderViewportHandle, RenderServerError> {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        if let Some(viewport) = self.viewport {
            if viewport.size == size {
                return Ok(viewport.handle);
            }
            self.render_server.destroy_viewport(viewport.handle)?;
            self.viewport = None;
            self.latest_generation = None;
            self.latest_image = None;
        }

        let descriptor = RenderViewportDescriptor::new(size).with_label("editor.viewport");
        let handle = self.render_server.create_viewport(descriptor)?;
        self.viewport = Some(ActiveViewport { handle, size });
        Ok(handle)
    }
}

impl Drop for ViewportState {
    fn drop(&mut self) {
        if let Some(viewport) = self.viewport {
            let _ = self.render_server.destroy_viewport(viewport.handle);
        }
    }
}

fn import_frame_image(frame: &CapturedFrame) -> Result<(u64, Image), String> {
    if frame.width == 0 || frame.height == 0 {
        return Err("render server returned a zero-sized viewport frame".to_string());
    }

    let expected_len = frame.width as usize * frame.height as usize * 4;
    if frame.rgba.len() != expected_len {
        return Err(format!(
            "render server returned {} RGBA bytes for a {}x{} frame",
            frame.rgba.len(),
            frame.width,
            frame.height
        ));
    }

    let buffer =
        SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&frame.rgba, frame.width, frame.height);
    Ok((frame.generation, Image::from_rgba8(buffer)))
}

#[cfg(test)]
struct TestRenderServer;

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use zircon_math::UVec2;
    use zircon_render_server::{
        CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderServer, RenderServerError,
        RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    };
    use zircon_scene::{RenderFrameExtract, RenderWorldSnapshotHandle, World};

    use super::SlintViewportController;

    #[test]
    fn controller_creates_and_resizes_render_server_viewports() {
        let server = std::sync::Arc::new(FakeRenderServer::default());
        let controller = SlintViewportController::new_with_server(server.clone());
        let extract = test_extract();

        controller
            .submit_extract(extract.clone(), UVec2::new(320, 240))
            .unwrap();
        controller
            .submit_extract(extract, UVec2::new(640, 480))
            .unwrap();

        let state = server.state.lock().unwrap();
        assert_eq!(
            state.created_viewports,
            vec![
                RenderViewportDescriptor::new(UVec2::new(320, 240)).with_label("editor.viewport"),
                RenderViewportDescriptor::new(UVec2::new(640, 480)).with_label("editor.viewport"),
            ]
        );
        assert_eq!(
            state.destroyed_viewports,
            vec![RenderViewportHandle::new(1)]
        );
        assert_eq!(
            state.submitted_viewports,
            vec![RenderViewportHandle::new(1), RenderViewportHandle::new(2)]
        );
    }

    #[test]
    fn controller_polls_latest_captured_frame_from_render_server() {
        let server = std::sync::Arc::new(FakeRenderServer::default());
        let controller = SlintViewportController::new_with_server(server.clone());

        controller
            .submit_extract(test_extract(), UVec2::new(160, 90))
            .unwrap();

        let image = controller.poll_image();

        assert!(image.is_some());
        assert_eq!(server.state.lock().unwrap().capture_requests, 1);
    }

    #[test]
    fn controller_updates_extract_camera_aspect_ratio_to_match_viewport_size() {
        let server = std::sync::Arc::new(FakeRenderServer::default());
        let controller = SlintViewportController::new_with_server(server.clone());
        let extract = test_extract();

        controller
            .submit_extract(extract, UVec2::new(300, 150))
            .unwrap();

        assert_eq!(
            server.state.lock().unwrap().submitted_aspect_ratios,
            vec![2.0]
        );
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            World::new().to_render_snapshot(),
        )
    }

    #[derive(Default)]
    struct FakeRenderServer {
        state: Mutex<FakeRenderServerState>,
    }

    #[derive(Default)]
    struct FakeRenderServerState {
        next_viewport_id: u64,
        created_viewports: Vec<RenderViewportDescriptor>,
        viewport_sizes: HashMap<RenderViewportHandle, UVec2>,
        destroyed_viewports: Vec<RenderViewportHandle>,
        submitted_viewports: Vec<RenderViewportHandle>,
        submitted_aspect_ratios: Vec<f32>,
        capture_requests: usize,
        captures: HashMap<RenderViewportHandle, CapturedFrame>,
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

        fn destroy_viewport(
            &self,
            viewport: RenderViewportHandle,
        ) -> Result<(), RenderServerError> {
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

        fn reload_pipeline(
            &self,
            _pipeline: RenderPipelineHandle,
        ) -> Result<(), RenderServerError> {
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
}
