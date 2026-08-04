use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::super::RuntimeEntryApp;

impl ApplicationHandler for RuntimeEntryApp {
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "resumed");
        if self.create_primary_window_surface(event_loop)
            && self.submit_mvp_input_probe_if_requested(event_loop)
        {
            self.request_runtime_frame();
        }
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "can_create_surfaces");
        if self.create_primary_window_surface(event_loop)
            && self.submit_mvp_input_probe_if_requested(event_loop)
        {
            self.request_runtime_frame();
        }
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
        if !self.failure_state.is_recorded() {
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
    fn input_probe_failure_blocks_initial_frame_scheduling_for_both_surface_hooks() {
        let source = include_str!("hooks.rs");
        let probe_then_frame = [
            "&& self.submit_mvp_input_probe_if_requested(event_loop)\n",
            "        {\n            self.request_runtime_frame();\n        }",
        ]
        .concat();

        assert_eq!(source.matches(&probe_then_frame).count(), 2);
    }
}
