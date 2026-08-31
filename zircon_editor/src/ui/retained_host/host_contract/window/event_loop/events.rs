mod focus;
mod keyboard;
mod pointer;
mod resize;

use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::primitives::CloseRequestResponse;
use winit::event::{ButtonSource, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::ui::layout::UiPoint;

use super::platform_input::event_uses_platform_input;
use super::platform_input::PlatformInputTranslation;
use super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn window_event_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: WindowEvent,
    ) {
        if self.try_defer_idle_pointer_move(&event) {
            return;
        }
        self.flush_pending_idle_pointer_move();
        let mouse_button_pressed = mouse_button_pressed(&event);
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
            WindowEvent::PointerEntered { .. } => {
                let platform_event = require_platform_input(platform_input_event);
                self.begin_input_outcome(platform_event.sequence);
                self.reject_input_outcome();
            }
            WindowEvent::PointerLeft { .. } => {
                let platform_event = require_platform_input(platform_input_event);
                self.begin_input_outcome(platform_event.sequence);
                let (x, y) = self.last_pointer_position.unwrap_or((0.0, 0.0));
                if let Some(pointer) = super::platform_input::platform_pointer_cancel_input(
                    platform_event.event,
                    UiPoint::new(x, y),
                ) {
                    self.host
                        .global::<UiHostContext>()
                        .invoke_workbench_pointer_input(pointer, None);
                }
                self.reject_input_outcome();
            }
            WindowEvent::Focused(true) => self.handle_native_window_focused(),
            WindowEvent::Focused(false) => {
                self.pressed_mouse_button_count = 0;
                self.handle_native_window_focus_lost();
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
        if let Some(pressed) = mouse_button_pressed {
            self.pressed_mouse_button_count = if pressed {
                self.pressed_mouse_button_count.saturating_add(1)
            } else {
                self.pressed_mouse_button_count.saturating_sub(1)
            };
        }
    }
}

fn mouse_button_pressed(event: &WindowEvent) -> Option<bool> {
    match event {
        WindowEvent::PointerButton {
            state: ElementState::Pressed,
            button: ButtonSource::Mouse(_),
            ..
        } => Some(true),
        WindowEvent::PointerButton {
            state: ElementState::Released,
            button: ButtonSource::Mouse(_),
            ..
        } => Some(false),
        _ => None,
    }
}

fn require_platform_input(
    translated: Option<PlatformInputTranslation>,
) -> PlatformInputTranslation {
    translated.expect("routed native input must retain its assigned sequence")
}

#[cfg(test)]
mod tests {
    use winit::event::{ButtonSource, ElementState, MouseButton, WindowEvent};

    use super::mouse_button_pressed;

    #[test]
    fn routed_native_inputs_keep_translation_identity_until_their_handler() {
        let source = include_str!("events.rs");

        assert!(source.contains("then(|| self.translate_platform_input_event(&event))"));
        assert!(!source.contains(".flatten()"));
        assert!(source.contains("require_platform_input(platform_input_event)"));
        assert!(source.contains("self.begin_input_outcome(platform_event.sequence)"));
        assert!(source.contains("self.reject_input_outcome()"));
    }

    #[test]
    fn native_focus_loss_routes_to_the_viewport_interaction_cancellation_callback() {
        let source = include_str!("events.rs");

        assert!(source.contains("WindowEvent::Focused(false)"));
        assert!(source.contains("handle_native_window_focus_lost"));
    }

    #[test]
    fn native_focus_gain_routes_to_the_owner_acknowledgement_callback() {
        let source = include_str!("events.rs");

        assert!(source.contains("WindowEvent::Focused(true)"));
        assert!(source.contains("handle_native_window_focused"));
    }

    #[test]
    fn only_mouse_buttons_gate_idle_move_coalescing() {
        let mouse = WindowEvent::PointerButton {
            device_id: None,
            state: ElementState::Pressed,
            position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            primary: true,
            button: ButtonSource::Mouse(MouseButton::Left),
        };
        let unknown = WindowEvent::PointerButton {
            device_id: None,
            state: ElementState::Pressed,
            position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            primary: true,
            button: ButtonSource::Unknown(1),
        };

        assert_eq!(mouse_button_pressed(&mouse), Some(true));
        assert_eq!(mouse_button_pressed(&unknown), None);
    }
}
