use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_log;

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
                self.report_fatal_failure(
                    "runtime_surface_present",
                    format!(
                        "fallback_presenter size={}x{}",
                        self.viewport_size.width, self.viewport_size.height
                    ),
                    format!("fallback presenter creation failed: {error}"),
                    "verify the graphics adapter and window surface, then restart zircon_runtime",
                );
                event_loop.exit();
                false
            }
        }
    }
}
