use crate::core::framework::render::{GraphicsDebuggerStatus, RenderViewportHandle};

#[derive(Clone, Debug)]
pub(in crate::graphics::runtime::render_framework) struct GraphicsDebuggerState {
    backend_name: String,
    available: bool,
    capture_count_on_next_created_viewport: u32,
    capture_sequence_viewport: Option<RenderViewportHandle>,
    remaining_sequence_capture_count: u32,
    pending_viewport: Option<RenderViewportHandle>,
    queued_viewport: Option<RenderViewportHandle>,
    active_capture: bool,
    last_capture_frame: Option<u64>,
    last_error: Option<String>,
}

impl GraphicsDebuggerState {
    pub(in crate::graphics::runtime::render_framework) fn available_with_capture_frame_count(
        backend_name: impl Into<String>,
        capture_frame_count: u32,
    ) -> Self {
        Self {
            backend_name: backend_name.into(),
            available: true,
            capture_count_on_next_created_viewport: capture_frame_count,
            capture_sequence_viewport: None,
            remaining_sequence_capture_count: 0,
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
        self.capture_count_on_next_created_viewport = 1;
    }

    pub(in crate::graphics::runtime::render_framework) fn request_capture_for_created_viewport_if_needed(
        &mut self,
        viewport: RenderViewportHandle,
    ) -> bool {
        let capture_count = self.capture_count_on_next_created_viewport;
        if capture_count == 0 {
            return false;
        }

        self.capture_count_on_next_created_viewport = 0;
        if self.pending_viewport.is_none() {
            self.capture_sequence_viewport = Some(viewport);
            self.remaining_sequence_capture_count = capture_count.saturating_sub(1);
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
            if self.capture_sequence_viewport == Some(viewport) {
                self.capture_sequence_viewport = None;
                self.remaining_sequence_capture_count = 0;
            }
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
        if self.capture_sequence_viewport == Some(viewport) {
            self.capture_sequence_viewport = None;
            self.remaining_sequence_capture_count = 0;
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
        if error.is_some() {
            self.capture_sequence_viewport = None;
            self.remaining_sequence_capture_count = 0;
        }
        self.pending_viewport = if let Some(queued_viewport) = self.queued_viewport.take() {
            Some(queued_viewport)
        } else {
            self.take_next_sequence_capture()
        };
        self.active_capture = false;
        if let Some(frame_generation) = frame_generation {
            self.last_capture_frame = Some(frame_generation);
        }
        self.last_error = error;
    }

    fn take_next_sequence_capture(&mut self) -> Option<RenderViewportHandle> {
        if self.remaining_sequence_capture_count == 0 {
            self.capture_sequence_viewport = None;
            return None;
        }
        let Some(viewport) = self.capture_sequence_viewport else {
            self.remaining_sequence_capture_count = 0;
            return None;
        };
        self.remaining_sequence_capture_count =
            self.remaining_sequence_capture_count.saturating_sub(1);
        Some(viewport)
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

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderViewportHandle;

    use super::GraphicsDebuggerState;

    #[test]
    fn capture_frame_count_queues_consecutive_frames_for_one_viewport() {
        let viewport = RenderViewportHandle::new(7);
        let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 2);

        assert!(state.request_capture_for_created_viewport_if_needed(viewport));
        assert!(state.should_capture(viewport));
        state.begin_capture();
        state.finish_capture(Some(10), None);

        assert!(state.should_capture(viewport));
        assert!(state.status().capture_pending);
        state.begin_capture();
        state.finish_capture(Some(11), None);

        let status = state.status();
        assert!(!status.capture_pending);
        assert!(!status.active_capture);
        assert_eq!(status.last_capture_frame, Some(11));
    }

    #[test]
    fn zero_capture_frame_count_does_not_arm_the_created_viewport() {
        let viewport = RenderViewportHandle::new(8);
        let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 0);

        assert!(!state.request_capture_for_created_viewport_if_needed(viewport));
        assert!(!state.should_capture(viewport));
    }

    #[test]
    fn failed_capture_cancels_the_remaining_sequence() {
        let viewport = RenderViewportHandle::new(9);
        let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 3);
        state.request_capture_for_created_viewport_if_needed(viewport);

        state.begin_capture();
        state.fail_pending_capture(viewport, "capture failed".to_owned());

        assert!(!state.should_capture(viewport));
        assert!(!state.status().capture_pending);
        assert_eq!(state.status().last_error.as_deref(), Some("capture failed"));
    }

    #[test]
    fn capture_finish_error_does_not_schedule_the_next_sequence_frame() {
        let viewport = RenderViewportHandle::new(10);
        let mut state = GraphicsDebuggerState::available_with_capture_frame_count("renderdoc", 2);
        state.request_capture_for_created_viewport_if_needed(viewport);

        state.begin_capture();
        state.finish_capture(Some(20), Some("stop failed".to_owned()));

        let status = state.status();
        assert!(!status.capture_pending);
        assert!(!status.active_capture);
        assert_eq!(status.last_capture_frame, Some(20));
        assert_eq!(status.last_error.as_deref(), Some("stop failed"));
    }
}
