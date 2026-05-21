use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::super::RuntimeEntryApp;

impl ApplicationHandler for RuntimeEntryApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "can_create_surfaces");
        self.create_primary_window_surface(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "window_event");
        self.handle_window_event(event_loop, event);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "about_to_wait");
        self.pump_frame_loop(event_loop);
    }

    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _device_id: Option<DeviceId>,
        event: DeviceEvent,
    ) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "device_event");
        self.handle_device_event(event_loop, event);
    }
}
