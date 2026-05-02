use std::sync::Arc;

use crate::core::framework::input::InputManager;
use crate::core::framework::render::{
    CapturedFrame, RenderFrameExtract, RenderFramework, RenderFrameworkError,
    RenderViewportDescriptor, RenderViewportHandle,
};
use crate::core::manager::resolve_render_framework;
use crate::core::math::UVec2;
use crate::core::{CoreError, CoreHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveViewport {
    handle: RenderViewportHandle,
    size: UVec2,
}

pub(super) struct RuntimeRenderBridge {
    render_framework: Arc<dyn RenderFramework>,
    viewport: Option<ActiveViewport>,
    last_generation: Option<u64>,
}

impl RuntimeRenderBridge {
    pub(super) fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        let render_framework = resolve_render_framework(core)?;
        Ok(Self {
            render_framework,
            viewport: None,
            last_generation: None,
        })
    }

    pub(super) fn submit_extract(
        &mut self,
        mut extract: RenderFrameExtract,
        size: UVec2,
    ) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
        let viewport = self.ensure_viewport(size)?;
        extract.apply_viewport_size(size);
        self.render_framework
            .submit_frame_extract(viewport, extract)?;
        let Some(frame) = self.render_framework.capture_frame(viewport)? else {
            return Ok(None);
        };
        if self.last_generation == Some(frame.generation) {
            return Ok(None);
        }
        self.last_generation = Some(frame.generation);
        Ok(Some(frame))
    }

    fn ensure_viewport(
        &mut self,
        size: UVec2,
    ) -> Result<RenderViewportHandle, RenderFrameworkError> {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        if let Some(viewport) = self.viewport {
            if viewport.size == size {
                return Ok(viewport.handle);
            }
            self.render_framework.destroy_viewport(viewport.handle)?;
            self.viewport = None;
            self.last_generation = None;
        }

        let descriptor = RenderViewportDescriptor::new(size).with_label("runtime.viewport");
        let handle = self.render_framework.create_viewport(descriptor)?;
        self.viewport = Some(ActiveViewport { handle, size });
        Ok(handle)
    }
}

impl Drop for RuntimeRenderBridge {
    fn drop(&mut self) {
        if let Some(viewport) = self.viewport {
            let _ = self.render_framework.destroy_viewport(viewport.handle);
        }
    }
}

pub(super) fn resolve_input(core: &CoreHandle) -> Result<Arc<dyn InputManager>, CoreError> {
    crate::core::manager::resolve_input_manager(core)
}
