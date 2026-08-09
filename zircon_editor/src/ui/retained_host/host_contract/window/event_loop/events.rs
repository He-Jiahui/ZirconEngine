mod keyboard;
mod pointer;
mod resize;

use crate::ui::retained_host::primitives::CloseRequestResponse;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;

use super::platform_input::event_uses_platform_input;
use super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn window_event_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: WindowEvent,
    ) {
        let platform_input_event = event_uses_platform_input(&event)
            .then(|| self.translate_platform_input_event(&event))
            .flatten();
        match event {
            WindowEvent::CloseRequested => {
                let response = self.host.close_requested_response();
                if matches!(response, CloseRequestResponse::HideWindow) {
                    self.host.state.borrow_mut().window_visible = false;
                    event_loop.exit();
                }
            }
            WindowEvent::SurfaceResized(size) => {
                self.handle_surface_resized(event_loop, size);
            }
            WindowEvent::Moved(position) => {
                self.handle_window_moved(position);
            }
            WindowEvent::PointerMoved { position, .. } => {
                self.handle_pointer_moved(platform_input_event, position);
            }
            WindowEvent::PointerButton { position, .. } => {
                self.handle_pointer_button(platform_input_event, position);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event, platform_input_event);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.current_modifiers = modifiers.state();
            }
            WindowEvent::Ime(_) => {
                self.handle_ime_input(platform_input_event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(platform_input_event, delta);
            }
            WindowEvent::RedrawRequested => {
                self.redraw_requested_impl(event_loop);
            }
            _ => {}
        }
    }
}
