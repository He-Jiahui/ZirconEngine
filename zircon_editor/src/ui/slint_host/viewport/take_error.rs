use super::slint_viewport_controller::SlintViewportController;

impl SlintViewportController {
    pub(crate) fn take_error(&self) -> Option<String> {
        self.lock_shared().last_error.take()
    }
}
