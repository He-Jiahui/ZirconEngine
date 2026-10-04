use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::super::RuntimeEntryApp;

impl ApplicationHandler for RuntimeEntryApp {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "resumed");
        self.handle_application_resumed(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "can_create_surfaces");
        self.handle_surface_availability(event_loop);
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "suspended");
        self.handle_application_suspended(event_loop);
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "destroy_surfaces");
        self.handle_surface_destruction(event_loop);
    }

    fn proxy_wake_up(&mut self, _event_loop: &dyn ActiveEventLoop) {
        if !self.failure_state.is_recorded() {
            self.request_runtime_frame();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "window_event");
        if !self.failure_state.is_recorded() {
            self.handle_window_event(event_loop, window_id, event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "about_to_wait");
        if !self.failure_state.is_recorded() && self.application_lifecycle.allows_frame_pump() {
            self.pump_frame_loop(event_loop);
        }
    }

    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _device_id: Option<DeviceId>,
        event: DeviceEvent,
    ) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "device_event");
        if !self.failure_state.is_recorded() {
            self.handle_device_event(event_loop, event);
        }
    }
}

#[cfg(test)]
#[path = "tests/hooks.rs"]
mod tests;
