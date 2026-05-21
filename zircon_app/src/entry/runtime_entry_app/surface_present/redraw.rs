use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn present_redraw_frame(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        zircon_runtime::profile_frame!("app", "runtime_redraw");
        zircon_runtime::profile_scope!("app", "runtime_entry", "redraw_requested");
        if self.surface_present_enabled && !self.surface_present_failed {
            match self
                .session
                .present_viewport(self.viewport, self.viewport_size)
            {
                Ok(true) => return,
                Ok(false) => {
                    self.fail_surface_present();
                }
                Err(_) => {
                    self.fail_surface_present();
                }
            }
        }
        if !self.ensure_fallback_presenter(event_loop) {
            return;
        }
        if let Some(presenter) = self.presenter.as_mut() {
            match self
                .session
                .capture_frame(self.viewport, self.viewport_size)
            {
                Ok(frame) => {
                    if presenter.present(&frame).is_err() {
                        event_loop.exit();
                    }
                }
                Err(_) => {
                    event_loop.exit();
                }
            }
        }
    }
}
