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

    fn exiting(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "exiting");
        self.handle_application_exit(event_loop);
    }

    fn proxy_wake_up(&mut self, _event_loop: &dyn ActiveEventLoop) {
        if !self.failure_state.is_recorded() {
            self.request_runtime_frame();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "window_event");
        if !self.failure_state.is_recorded() {
            self.handle_window_event(event_loop, event);
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
mod tests {
    #[test]
    fn surface_ownership_is_confirmed_before_input_probe_controls_initial_frame_scheduling() {
        let source = include_str!("../application_lifecycle/events.rs");
        let surface_created = source
            .find("if self.create_primary_window_surface(event_loop) {")
            .expect(
                "surface availability should create the primary window only after winit admission",
            );
        let ownership_confirmed = source
            .find("self.application_lifecycle.confirm_surface_created();")
            .expect("successful native creation should immediately update lifecycle ownership");
        let input_probe = source
            .find("if self.submit_mvp_input_probe_if_requested(event_loop) {")
            .expect("input probe should continue to gate the initial frame");
        let frame_requested = source
            .find("self.request_runtime_frame();")
            .expect("input probe success should schedule the initial frame");

        assert!(surface_created < ownership_confirmed);
        assert!(ownership_confirmed < input_probe);
        assert!(input_probe < frame_requested);
    }
}
