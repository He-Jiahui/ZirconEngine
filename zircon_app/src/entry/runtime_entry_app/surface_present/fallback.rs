use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;
use crate::runtime_presenter::SoftbufferRuntimePresenter;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn ensure_fallback_presenter(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        if self.presenter.is_some() {
            return true;
        }
        let Some(window) = self.window.as_ref() else {
            return false;
        };
        match SoftbufferRuntimePresenter::new(window.clone()) {
            Ok(presenter) => {
                self.presenter = Some(presenter);
                true
            }
            Err(_) => {
                event_loop.exit();
                false
            }
        }
    }
}
