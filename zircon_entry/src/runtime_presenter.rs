use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::window::Window;
use zircon_core::{CoreError, CoreHandle};
use zircon_math::UVec2;
use zircon_render_server::{
    resolve_render_server, CapturedFrame, RenderServer, RenderServerError,
    RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_scene::RenderFrameExtract;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveViewport {
    handle: RenderViewportHandle,
    size: UVec2,
}

pub(crate) struct RenderServerRuntimeBridge {
    render_server: Arc<dyn RenderServer>,
    viewport: Option<ActiveViewport>,
    last_generation: Option<u64>,
}

impl RenderServerRuntimeBridge {
    pub(crate) fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        let render_server = resolve_render_server(core)?;
        Ok(Self::new_with_server(render_server))
    }

    fn new_with_server(render_server: Arc<dyn RenderServer>) -> Self {
        Self {
            render_server,
            viewport: None,
            last_generation: None,
        }
    }

    pub(crate) fn submit_extract(
        &mut self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<Option<CapturedFrame>, RenderServerError> {
        let viewport = self.ensure_viewport(size)?;
        extract.apply_viewport_size(size);
        self.render_server.submit_frame_extract(viewport, extract)?;
        let Some(frame) = self.render_server.capture_frame(viewport)? else {
            return Ok(None);
        };
        if self.last_generation == Some(frame.generation) {
            return Ok(None);
        }
        self.last_generation = Some(frame.generation);
        Ok(Some(frame))
    }

    fn ensure_viewport(&mut self, size: UVec2) -> Result<RenderViewportHandle, RenderServerError> {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        if let Some(viewport) = self.viewport {
            if viewport.size == size {
                return Ok(viewport.handle);
            }
            self.render_server.destroy_viewport(viewport.handle)?;
            self.viewport = None;
            self.last_generation = None;
        }

        let descriptor = RenderViewportDescriptor::new(size).with_label("runtime.viewport");
        let handle = self.render_server.create_viewport(descriptor)?;
        self.viewport = Some(ActiveViewport { handle, size });
        Ok(handle)
    }
}

impl Drop for RenderServerRuntimeBridge {
    fn drop(&mut self) {
        if let Some(viewport) = self.viewport {
            let _ = self.render_server.destroy_viewport(viewport.handle);
        }
    }
}

pub(crate) struct SoftbufferRuntimePresenter {
    #[allow(dead_code)]
    context: Context<Arc<Window>>,
    surface: Surface<Arc<Window>, Arc<Window>>,
    size: UVec2,
}

impl SoftbufferRuntimePresenter {
    pub(crate) fn new(window: Arc<Window>) -> Result<Self, softbuffer::SoftBufferError> {
        let context = Context::new(window.clone())?;
        let mut surface = Surface::new(&context, window.clone())?;
        let size = current_window_size(&window);
        resize_surface(&mut surface, size)?;
        Ok(Self {
            context,
            surface,
            size,
        })
    }

    pub(crate) fn resize(&mut self, size: UVec2) -> Result<(), softbuffer::SoftBufferError> {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        resize_surface(&mut self.surface, size)?;
        self.size = size;
        Ok(())
    }

    pub(crate) fn present(
        &mut self,
        frame: &CapturedFrame,
    ) -> Result<(), softbuffer::SoftBufferError> {
        let frame_size = UVec2::new(frame.width.max(1), frame.height.max(1));
        if self.size != frame_size {
            self.resize(frame_size)?;
        }

        let mut buffer = self.surface.buffer_mut()?;
        buffer.fill(0);

        for (pixel, rgba) in buffer.iter_mut().zip(frame.rgba.chunks_exact(4)) {
            let red = rgba[0] as u32;
            let green = rgba[1] as u32;
            let blue = rgba[2] as u32;
            *pixel = (red << 16) | (green << 8) | blue;
        }

        buffer.present()
    }
}

fn current_window_size(window: &Window) -> UVec2 {
    let size = window.inner_size();
    UVec2::new(size.width.max(1), size.height.max(1))
}

fn resize_surface(
    surface: &mut Surface<Arc<Window>, Arc<Window>>,
    size: UVec2,
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.x), non_zero(size.y))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to non-zero")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use zircon_math::UVec2;
    use zircon_render_server::{
        CapturedFrame, RenderPipelineHandle, RenderQualityProfile, RenderServer, RenderServerError,
        RenderStats, RenderViewportDescriptor, RenderViewportHandle,
    };
    use zircon_scene::{RenderFrameExtract, RenderWorldSnapshotHandle, World};

    use super::RenderServerRuntimeBridge;

    #[test]
    fn runtime_bridge_creates_and_resizes_render_viewports() {
        let server = Arc::new(FakeRenderServer::default());
        let mut bridge = RenderServerRuntimeBridge::new_with_server(server.clone());
        let extract = test_extract();

        bridge
            .submit_extract(extract.clone(), UVec2::new(320, 180))
            .unwrap();
        bridge
            .submit_extract(extract, UVec2::new(640, 360))
            .unwrap();

        let state = server.state.lock().unwrap();
        assert_eq!(
            state.created_viewports,
            vec![
                RenderViewportDescriptor::new(UVec2::new(320, 180)).with_label("runtime.viewport"),
                RenderViewportDescriptor::new(UVec2::new(640, 360)).with_label("runtime.viewport"),
            ]
        );
        assert_eq!(
            state.destroyed_viewports,
            vec![RenderViewportHandle::new(1)]
        );
    }

    #[test]
    fn runtime_bridge_returns_latest_captured_frame() {
        let server = Arc::new(FakeRenderServer::default());
        let mut bridge = RenderServerRuntimeBridge::new_with_server(server);

        let frame = bridge
            .submit_extract(test_extract(), UVec2::new(160, 90))
            .unwrap()
            .expect("captured frame");

        assert_eq!(frame.width, 1);
        assert_eq!(frame.height, 1);
        assert_eq!(frame.generation, 1);
    }

    #[test]
    fn runtime_bridge_updates_extract_camera_aspect_ratio_to_match_viewport_size() {
        let server = Arc::new(FakeRenderServer::default());
        let mut bridge = RenderServerRuntimeBridge::new_with_server(server.clone());
        let mut extract = test_extract();
        extract.view.camera.aspect_ratio = 1.0;

        bridge
            .submit_extract(extract, UVec2::new(400, 200))
            .unwrap()
            .expect("captured frame");

        let state = server.state.lock().unwrap();
        assert_eq!(state.submitted_aspect_ratios, vec![2.0]);
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(5),
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
        destroyed_viewports: Vec<RenderViewportHandle>,
        submitted_aspect_ratios: Vec<f32>,
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
            extract: RenderFrameExtract,
        ) -> Result<(), RenderServerError> {
            let mut state = self.state.lock().unwrap();
            state
                .submitted_aspect_ratios
                .push(extract.view.camera.aspect_ratio);
            state.captures.insert(
                viewport,
                CapturedFrame::new(1, 1, vec![255, 255, 255, 255], viewport.raw()),
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
            Ok(self.state.lock().unwrap().captures.get(&viewport).cloned())
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
