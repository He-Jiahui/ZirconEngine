mod keyboard;
mod pointer;
mod resize;

use crate::ui::retained_host::primitives::CloseRequestResponse;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;

use super::platform_input::event_uses_platform_input;
use super::platform_input::PlatformInputTranslation;
use super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn window_event_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: WindowEvent,
    ) {
        let platform_input_event =
            event_uses_platform_input(&event).then(|| self.translate_platform_input_event(&event));
        match event {
            WindowEvent::CloseRequested => {
                let response = self.host.close_requested_response();
                if matches!(response, CloseRequestResponse::HideWindow) {
                    self.host.state.borrow_mut().window_visible = false;
                    event_loop.exit();
                }
            }
            WindowEvent::SurfaceResized(size) => {
                self.handle_surface_resized(
                    event_loop,
                    size,
                    require_platform_input(platform_input_event),
                );
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_window_scale_factor_changed(
                    event_loop,
                    scale_factor,
                    require_platform_input(platform_input_event),
                );
            }
            WindowEvent::Moved(position) => {
                self.handle_window_moved(position);
            }
            WindowEvent::PointerMoved { position, .. } => {
                self.handle_pointer_moved(require_platform_input(platform_input_event), position);
            }
            WindowEvent::PointerButton { position, .. } => {
                self.handle_pointer_button(require_platform_input(platform_input_event), position);
            }
            WindowEvent::PointerEntered { .. } | WindowEvent::PointerLeft { .. } => {
                let platform_event = require_platform_input(platform_input_event);
                self.begin_input_outcome(platform_event.sequence);
                self.reject_input_outcome();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event, require_platform_input(platform_input_event));
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.current_modifiers = modifiers.state();
            }
            WindowEvent::Ime(_) => {
                self.handle_ime_input(require_platform_input(platform_input_event));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(require_platform_input(platform_input_event), delta);
            }
            WindowEvent::RedrawRequested => {
                self.redraw_requested_impl(event_loop);
            }
            _ => {}
        }
    }
}

fn require_platform_input(
    translated: Option<PlatformInputTranslation>,
) -> PlatformInputTranslation {
    translated.expect("routed native input must retain its assigned sequence")
}

#[cfg(test)]
mod tests {
    #[test]
    fn routed_native_inputs_keep_translation_identity_until_their_handler() {
        let source = include_str!("events.rs");

        assert!(source.contains("then(|| self.translate_platform_input_event(&event))"));
        assert!(!source.contains(".flatten()"));
        assert!(source.contains("require_platform_input(platform_input_event)"));
        assert!(source.contains("self.begin_input_outcome(platform_event.sequence)"));
        assert!(source.contains("self.reject_input_outcome()"));
    }
}
