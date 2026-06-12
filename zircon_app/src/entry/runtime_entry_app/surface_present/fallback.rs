use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::{write_error, write_log};

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
                write_log(
                    "runtime_surface_present",
                    format!(
                        "runtime_fallback_presenter_created size={}x{}",
                        self.viewport_size.width, self.viewport_size.height
                    ),
                );
                true
            }
            Err(error) => {
                write_error(
                    "runtime_surface_present",
                    format!(
                        "runtime_fallback_presenter_create_failed size={}x{} error={error}",
                        self.viewport_size.width, self.viewport_size.height
                    ),
                );
                event_loop.exit();
                false
            }
        }
    }
}
