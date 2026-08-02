use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::ui::retained_host::host_contract::presenter::{
    HostChromePresenter, HostPresenterBackend, create_host_chrome_presenter,
};

use super::super::super::UiHostWindow;

pub(super) fn create_presenter_or_exit(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
    window: Arc<dyn Window>,
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>)> {
    let presenter_backend = HostPresenterBackend::default_native();
    match create_host_chrome_presenter(presenter_backend, window.clone()) {
        Ok(presenter) => Some((presenter_backend, presenter)),
        Err(error) if presenter_backend.is_gpu() => {
            zircon_runtime::diagnostic_log::write_warn(
                "editor_host_window",
                format!(
                    "failed to create {} presenter, falling back to softbuffer: {error}",
                    presenter_backend.label()
                ),
            );
            create_fallback_presenter_or_exit(event_loop, host, window)
        }
        Err(error) => {
            report_presenter_error_and_exit(event_loop, host, presenter_backend, error);
            None
        }
    }
}

fn create_fallback_presenter_or_exit(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
    window: Arc<dyn Window>,
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>)> {
    let fallback_backend = HostPresenterBackend::fallback();
    match create_host_chrome_presenter(fallback_backend, window) {
        Ok(presenter) => Some((fallback_backend, presenter)),
        Err(error) => {
            report_presenter_error_and_exit(event_loop, host, fallback_backend, error);
            None
        }
    }
}

fn report_presenter_error_and_exit(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
    backend: HostPresenterBackend,
    error: impl std::fmt::Display,
) {
    host.report_fatal_failure(
        "editor_host_window",
        format!("presenter_backend={}", backend.label()),
        format!("presenter creation failed: {error}"),
        "verify the graphics adapter and window surface, then restart zircon_editor",
    );
    event_loop.exit();
}
