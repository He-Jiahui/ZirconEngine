use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

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
            if !shared_gpu_presenter_active {
                record_degraded_presenter(host, presenter_backend);
            }
            Some((presenter_backend, presenter, shared_gpu_presenter_active))
        }
        Err(error) if runtime_factory.is_some() && presenter_backend.is_gpu() => {
            host.set_graphics_degradation(Some(format!(
                "Standalone GPU presenter: shared GPU presenter creation failed ({error}); viewport/PIE capture uses CPU copies. Restart to retry."
            )));
            create_standalone_presenter_or_exit(event_loop, host, window)
        }
        Err(error) if presenter_backend.is_gpu() => {
            host.set_graphics_degradation(Some(format!(
                "Software presenter (backend=softbuffer): GPU unavailable ({error}); falling back to softbuffer. Viewport/PIE may be slow or unavailable. See the runtime log for adapter diagnostics."
            )));
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
        Ok((presenter, _)) => {
            record_degraded_presenter(host, fallback_backend);
            Some((fallback_backend, presenter, false))
        }
        Err(error) => {
            report_presenter_error_and_exit(event_loop, host, fallback_backend, error);
            None
        }
    }
}

fn record_degraded_presenter(host: &UiHostWindow, backend: HostPresenterBackend) {
    zircon_runtime::profile_counter!("editor", "ui.presenter.cpu_viewport_fallback_count", 1_u8);
    if backend == HostPresenterBackend::Softbuffer {
        zircon_runtime::profile_counter!("editor", "ui.presenter.software_fallback_count", 1_u8);
        if host.graphics_degradation().is_none() {
            host.set_graphics_degradation(Some(
                "Software presenter (backend=softbuffer, forced by profiling); viewport/PIE may be slow or unavailable.".to_owned(),
            ));
        }
    } else if host.graphics_degradation().is_none() {
        host.set_graphics_degradation(Some(
            "Standalone GPU presenter: shared GPU presentation is unavailable; viewport/PIE capture uses CPU copies. Restart to retry. See the runtime log for adapter diagnostics.".to_owned(),
        ));
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
