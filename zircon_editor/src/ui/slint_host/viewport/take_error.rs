use super::slint_viewport_controller::SlintViewportController;

impl SlintViewportController {
    pub(crate) fn take_error(&self) -> Option<String> {
        self.shared.lock().unwrap().last_error.take()
    }
}
