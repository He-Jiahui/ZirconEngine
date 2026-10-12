use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::host_contract::presenter::{
    create_host_chrome_presenter, HostChromePresenter, HostPresenterBackend,
};

use super::super::super::UiHostWindow;

pub(super) fn create_presenter_or_exit(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
    window: Arc<dyn Window>,
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>, bool)> {
    let runtime_factory = host.runtime_presenter_factory();
    create_presenter_with_runtime_factory(event_loop, host, window, runtime_factory.as_deref())
}

pub(super) fn create_standalone_presenter_or_exit(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
    window: Arc<dyn Window>,
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>, bool)> {
    create_presenter_with_runtime_factory(event_loop, host, window, None)
}

fn create_presenter_with_runtime_factory(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
    window: Arc<dyn Window>,
    runtime_factory: Option<
        &dyn crate::ui::retained_host::host_contract::presenter::RuntimeUiSurfacePresenterFactory,
    >,
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>, bool)> {
    let presenter_backend = HostPresenterBackend::default_native();
    match create_host_chrome_presenter(presenter_backend, window.clone(), runtime_factory) {
        Ok((presenter, shared_gpu_presenter_active)) => {
            Some((presenter_backend, presenter, shared_gpu_presenter_active))
        }
        Err(error) if presenter_backend.is_gpu() => {
            host.record_host_diagnostic(
                HostWindowDiagnosticSeverity::Warning,
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
) -> Option<(HostPresenterBackend, Box<dyn HostChromePresenter>, bool)> {
    let fallback_backend = HostPresenterBackend::fallback();
    match create_host_chrome_presenter(fallback_backend, window, None) {
        Ok((presenter, _)) => Some((fallback_backend, presenter, false)),
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
