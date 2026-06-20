use std::sync::Arc;

use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, write_error, DiagnosticLogLevel,
};

use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::presenter::{
    create_host_chrome_presenter, HostPresenterBackend,
};

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn can_create_surfaces_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        if self.window.is_some() {
            return;
        }

        let size = self.host.window().size();
        let window_attributes = WindowAttributes::default()
            .with_title("Zircon Editor")
            .with_surface_size(winit::dpi::LogicalSize::new(
                size.width as f64,
                size.height as f64,
            ));
        let window: Arc<dyn Window> = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::from(window),
            Err(_) => {
                write_error("editor_host_window", "failed to create native window");
                event_loop.exit();
                return;
            }
        };
        self.sync_host_window_state(window.as_ref());
        let presenter_backend = HostPresenterBackend::default_native();
        let (presenter_backend, presenter) =
            match create_host_chrome_presenter(presenter_backend, window.clone()) {
                Ok(presenter) => (presenter_backend, presenter),
                Err(error) if presenter_backend.is_gpu() => {
                    write_error(
                        "editor_host_window",
                        format!(
                            "failed to create {} presenter, falling back to softbuffer: {error}",
                            presenter_backend.label()
                        ),
                    );
                    let fallback_backend = HostPresenterBackend::fallback();
                    match create_host_chrome_presenter(fallback_backend, window.clone()) {
                        Ok(presenter) => (fallback_backend, presenter),
                        Err(error) => {
                            write_error(
                                "editor_host_window",
                                format!(
                                    "failed to create {} presenter: {error}",
                                    fallback_backend.label()
                                ),
                            );
                            event_loop.exit();
                            return;
                        }
                    }
                }
                Err(error) => {
                    write_error(
                        "editor_host_window",
                        format!(
                            "failed to create {} presenter: {error}",
                            presenter_backend.label()
                        ),
                    );
                    event_loop.exit();
                    return;
                }
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
        state.window_visible = true;
        state.window_maximized = window.is_maximized();
        if let Ok(position) = window.outer_position() {
            state.window_position = PhysicalPosition::new(position.x, position.y);
        }
    }
}
