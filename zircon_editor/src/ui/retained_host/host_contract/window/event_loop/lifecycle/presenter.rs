use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use zircon_runtime::diagnostic_log::write_error;

use crate::ui::retained_host::host_contract::presenter::{
    create_host_chrome_presenter, HostChromePresenter, HostPresenterBackend,
};

pub(super) fn create_presenter_or_exit(
    event_loop: &dyn ActiveEventLoop,
    window: Arc<dyn Window>,
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>)> {
    let presenter_backend = HostPresenterBackend::default_native();
    match create_host_chrome_presenter(presenter_backend, window.clone()) {
        Ok(presenter) => Some((presenter_backend, presenter)),
        Err(error) if presenter_backend.is_gpu() => {
            write_error(
                "editor_host_window",
                format!(
                    "failed to create {} presenter, falling back to softbuffer: {error}",
                    presenter_backend.label()
                ),
            );
            create_fallback_presenter_or_exit(event_loop, window)
        }
        Err(error) => {
            write_presenter_error_and_exit(event_loop, presenter_backend, error);
            None
        }
    }
}

fn create_fallback_presenter_or_exit(
    event_loop: &dyn ActiveEventLoop,
    window: Arc<dyn Window>,
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>)> {
    let fallback_backend = HostPresenterBackend::fallback();
    match create_host_chrome_presenter(fallback_backend, window) {
        Ok(presenter) => Some((fallback_backend, presenter)),
        Err(error) => {
            write_presenter_error_and_exit(event_loop, fallback_backend, error);
            None
        }
    }
}

fn write_presenter_error_and_exit(
    event_loop: &dyn ActiveEventLoop,
    backend: HostPresenterBackend,
    error: impl std::fmt::Display,
) {
    write_error(
        "editor_host_window",
        format!("failed to create {} presenter: {error}", backend.label()),
    );
    event_loop.exit();
}
