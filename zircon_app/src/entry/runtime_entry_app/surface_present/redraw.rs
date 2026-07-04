use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::{write_error, write_warn};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn present_redraw_frame(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let exit_after_first_presented_frame = self.exit_after_first_presented_frame;
        zircon_runtime::profile_frame!("app", "runtime_redraw");
        zircon_runtime::profile_scope!("app", "runtime_entry", "redraw_requested");
        if self.surface_present_enabled && !self.surface_present_failed {
            match self
                .session
                .present_viewport(self.viewport, self.viewport_size)
            {
                Ok(true) => {
                    exit_after_presented_frame(exit_after_first_presented_frame, event_loop);
                    return;
                }
                Ok(false) => {
                    write_warn(
                        "runtime_surface_present",
                        format!(
                            "runtime_surface_present_returned_false viewport={:?} size={}x{}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                    );
                    self.fail_surface_present();
                }
                Err(error) => {
                    write_warn(
                        "runtime_surface_present",
                        format!(
                            "runtime_surface_present_error viewport={:?} size={}x{} error={error}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                    );
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
                    if let Err(error) = presenter.present(&frame) {
                        write_error(
                            "runtime_surface_present",
                            format!(
                                "runtime_fallback_present_failed viewport={:?} size={}x{} frame={}x{} error={error}",
                                self.viewport,
                                self.viewport_size.width,
                                self.viewport_size.height,
                                frame.width(),
                                frame.height()
                            ),
                        );
                        event_loop.exit();
                    } else {
                        exit_after_presented_frame(exit_after_first_presented_frame, event_loop);
                    }
                }
                Err(error) => {
                    write_error(
                        "runtime_surface_present",
                        format!(
                            "runtime_capture_frame_failed viewport={:?} size={}x{} error={error}",
                            self.viewport, self.viewport_size.width, self.viewport_size.height
                        ),
                    );
                    event_loop.exit();
                }
            }
        }
    }
}

fn exit_after_presented_frame(enabled: bool, event_loop: &dyn ActiveEventLoop) {
    if enabled {
        event_loop.exit();
    }
}
