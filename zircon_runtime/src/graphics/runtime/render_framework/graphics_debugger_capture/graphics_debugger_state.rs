use crate::core::framework::render::{GraphicsDebuggerStatus, RenderViewportHandle};

#[derive(Clone, Debug)]
pub(in crate::graphics::runtime::render_framework) struct GraphicsDebuggerState {
    backend_name: String,
    available: bool,
    capture_next_created_viewport: bool,
    pending_viewport: Option<RenderViewportHandle>,
    queued_viewport: Option<RenderViewportHandle>,
    active_capture: bool,
    last_capture_frame: Option<u64>,
    last_error: Option<String>,
}

impl GraphicsDebuggerState {
    pub(in crate::graphics::runtime::render_framework) fn available_with_capture_next_created_viewport(
        backend_name: impl Into<String>,
        capture_next_created_viewport: bool,
    ) -> Self {
        Self {
            backend_name: backend_name.into(),
            available: true,
            capture_next_created_viewport,
            pending_viewport: None,
            queued_viewport: None,
            active_capture: false,
            last_capture_frame: None,
            last_error: None,
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::runtime::render_framework) fn request_next_created_viewport_capture(
        &mut self,
    ) {
        self.capture_next_created_viewport = true;
    }

    pub(in crate::graphics::runtime::render_framework) fn request_capture_for_created_viewport_if_needed(
        &mut self,
        viewport: RenderViewportHandle,
    ) -> bool {
        if !self.capture_next_created_viewport {
            return false;
        }

        self.capture_next_created_viewport = false;
        if self.pending_viewport.is_none() {
            self.request_capture(viewport);
            return true;
        }

        false
    }

    pub(in crate::graphics::runtime::render_framework) fn request_capture(
        &mut self,
        viewport: RenderViewportHandle,
    ) {
        if self.active_capture {
            self.queued_viewport = Some(viewport);
            self.last_error = None;
            return;
        }

        self.pending_viewport = Some(viewport);
        self.last_error = None;
    }

    pub(in crate::graphics::runtime::render_framework) fn should_capture(
        &self,
        viewport: RenderViewportHandle,
    ) -> bool {
        self.pending_viewport == Some(viewport)
    }

    pub(in crate::graphics::runtime::render_framework) fn begin_capture(&mut self) {
        self.active_capture = true;
    }

    pub(in crate::graphics::runtime::render_framework) fn fail_pending_capture(
        &mut self,
        viewport: RenderViewportHandle,
        error: String,
    ) {
        if self.should_capture(viewport) {
            self.pending_viewport = None;
            self.active_capture = false;
            self.last_error = Some(error);
        }
    }

    pub(in crate::graphics::runtime::render_framework) fn forget_viewport(
        &mut self,
        viewport: RenderViewportHandle,
    ) {
        let mut removed_capture = false;
        if self.pending_viewport == Some(viewport) {
            self.pending_viewport = None;
            removed_capture = true;
        }
        if self.queued_viewport == Some(viewport) {
            self.queued_viewport = None;
            removed_capture = true;
        }
        if removed_capture {
            self.last_error = Some(format!(
                "graphics debugger capture viewport `{}` was destroyed",
                viewport.raw()
            ));
        }
    }

    pub(in crate::graphics::runtime::render_framework) fn finish_capture(
        &mut self,
        frame_generation: Option<u64>,
        error: Option<String>,
    ) {
        self.pending_viewport = self.queued_viewport.take();
        self.active_capture = false;
        if let Some(frame_generation) = frame_generation {
            self.last_capture_frame = Some(frame_generation);
        }
        self.last_error = error;
    }

    pub(in crate::graphics::runtime::render_framework) fn status(&self) -> GraphicsDebuggerStatus {
        GraphicsDebuggerStatus {
            available: self.available,
            backend_name: self.backend_name.clone(),
            capture_pending: self.pending_viewport.is_some() || self.queued_viewport.is_some(),
            active_capture: self.active_capture,
            last_capture_frame: self.last_capture_frame,
            last_error: self.last_error.clone(),
        }
    }
}
