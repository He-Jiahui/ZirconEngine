mod native_window;
mod presenter;

use std::time::Instant;

use crate::ui::retained_host::host_contract::presenter::HostPresenterBackend;
use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::Window;
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::UiHostWindowEventLoop;
use native_window::create_native_window_or_exit;
use presenter::{create_presenter_or_exit, try_upgrade_to_runtime_presenter};

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
        let Some(window) = create_native_window_or_exit(event_loop, &self.host, requested_size)
        else {
            return;
        };
        self.sync_host_window_state(window.as_ref());
        let Some((presenter_backend, presenter, shared_gpu_presenter_active)) =
            create_presenter_or_exit(event_loop, &self.host, window.clone())
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
        self.shared_gpu_presenter_active = shared_gpu_presenter_active;
        self.host
            .set_direct_viewport_products_active(shared_gpu_presenter_active);
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
        self.try_upgrade_to_runtime_presenter(event_loop);
        self.drain_external_redraw_request();
        if self.host.take_due_runtime_frame_wake(Instant::now()) {
            // Materialize the wake through the regular external-redraw bridge so
            // redraw_requested_impl observes a pending frame update.
            self.drain_external_redraw_request();
        }
        match self.host.runtime_frame_wake_deadline() {
            Some(deadline) => event_loop.set_control_flow(ControlFlow::WaitUntil(deadline)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    fn try_upgrade_to_runtime_presenter(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.shared_gpu_presenter_active
            || self.presenter_backend != Some(HostPresenterBackend::Gpu)
        {
            return;
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        // Native graphics backends cannot configure two surfaces for the same HWND at once.
        // Release the startup presenter before the runtime-owned presenter claims the window.
        drop(self.presenter.take());
        let Some(presenter) = try_upgrade_to_runtime_presenter(&self.host, window.clone()) else {
            let Some((backend, presenter, shared_gpu_presenter_active)) =
                create_presenter_or_exit(event_loop, &self.host, window.clone())
            else {
                return;
            };
            self.presenter = Some(presenter);
            self.presenter_backend = Some(backend);
            self.shared_gpu_presenter_active = shared_gpu_presenter_active;
            self.host
                .set_direct_viewport_products_active(shared_gpu_presenter_active);
            window.request_redraw();
            return;
        };
        self.presenter = Some(presenter);
        self.shared_gpu_presenter_active = true;
        self.host.set_direct_viewport_products_active(true);
        window.request_redraw();
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

#[cfg(test)]
mod tests {
    #[test]
    fn no_runtime_deadline_resets_the_native_wait_policy() {
        let source = include_str!("lifecycle.rs");
        assert!(source.contains("None => event_loop.set_control_flow(ControlFlow::Wait)"));
    }

    #[test]
    fn runtime_presenter_upgrade_releases_previous_native_surface_first() {
        let source = include_str!("lifecycle.rs");
        let release = source
            .find("drop(self.presenter.take());")
            .expect("upgrade must release the startup surface");
        let create = source
            .find("let Some(presenter) = try_upgrade_to_runtime_presenter")
            .expect("upgrade must create the runtime presenter");
        assert!(release < create);
    }
}
