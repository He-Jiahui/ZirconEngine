mod native_window;
mod presenter;

use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::UiHostWindowEventLoop;
use native_window::create_native_window_or_exit;
use presenter::create_presenter_or_exit;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn can_create_surfaces_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        if self.window.is_some() {
            return;
        }

        let size = self.host.window().size();
        let requested_size = size.clone();
        let Some(window) = create_native_window_or_exit(event_loop, requested_size) else {
            return;
        };
        self.sync_host_window_state(window.as_ref());
        let Some((presenter_backend, presenter)) =
            create_presenter_or_exit(event_loop, window.clone())
        else {
            return;
        };
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_window",
                format!(
                    "created native window size={}x{} presenter_backend={}",
                    size.width,
                    size.height,
                    presenter_backend.label()
                ),
            );
        }
        window.request_redraw();
        self.window = Some(window);
        self.presenter = Some(presenter);
        self.presenter_backend = Some(presenter_backend);
    }

    pub(in crate::ui::retained_host::host_contract) fn about_to_wait_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        if self.host.state.borrow().exit_requested {
            event_loop.exit();
            return;
        }
        if let Some(window) = self.window.as_ref() {
            self.sync_host_window_state(window.as_ref());
        }
        self.drain_external_redraw_request();
    }

    pub(in crate::ui::retained_host::host_contract) fn sync_host_window_state(
        &self,
        window: &dyn Window,
    ) {
        let size = window.surface_size();
        let mut state = self.host.state.borrow_mut();
        state.window_size = PhysicalSize::new(size.width, size.height);
        state.set_window_scale_factor(window.scale_factor() as f32);
        state.window_visible = true;
        state.window_maximized = window.is_maximized();
        if let Ok(position) = window.outer_position() {
            state.window_position = PhysicalPosition::new(position.x, position.y);
        }
    }
}
