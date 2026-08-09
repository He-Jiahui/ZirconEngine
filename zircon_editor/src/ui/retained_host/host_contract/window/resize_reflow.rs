use super::UiHostWindow;

impl UiHostWindow {
    pub(crate) fn defer_native_resize_reflow(&self) {
        self.state.borrow_mut().native_resize_reflow_pending = true;
    }

    pub(crate) fn commit_native_resize_reflow(&self) {
        self.state.borrow_mut().native_resize_reflow_pending = false;
    }

    pub(crate) fn native_resize_reflow_pending(&self) -> bool {
        self.state.borrow().native_resize_reflow_pending
    }
}
