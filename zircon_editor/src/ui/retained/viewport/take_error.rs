use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(crate) fn take_error(&self) -> Option<String> {
        self.lock_shared().last_error.take()
    }
}
