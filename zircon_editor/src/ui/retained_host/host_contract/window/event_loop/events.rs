mod focus;
mod keyboard;
mod pointer;
mod resize;

use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::primitives::CloseRequestResponse;
use winit::event::{ButtonSource, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey};
use zircon_runtime_interface::ui::layout::UiPoint;

use super::platform_input::event_uses_platform_input;
use super::platform_input::PlatformInputTranslation;
use super::UiHostWindowEventLoop;

#[cfg(test)]
use crate::ui::retained_host::host_contract::window::UiHostWindow;

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
                let response = self.handle_native_close_requested();
                if matches!(response, CloseRequestResponse::HideWindow) {
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
                self.handle_pointer_left(require_platform_input(platform_input_event));
            }
            WindowEvent::Focused(true) => self.handle_native_window_focused(),
            WindowEvent::Focused(false) => {
                self.pressed_mouse_button_count = 0;
                self.handle_native_window_focus_lost();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let platform_event = require_platform_input(platform_input_event);
                if play_stop_trace_enabled() && is_native_shift_f5(&event, self.current_modifiers) {
                    eprintln!(
                        "mvp_play_trace component=editor_native_input stage=shift_f5_received input_seq={:?} state={:?} logical_key={:?} physical_key={:?}",
                        platform_event.sequence, event.state, event.logical_key, event.physical_key
                    );
                }
                self.handle_keyboard_input(event, platform_event);
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

    fn handle_native_close_requested(&mut self) -> CloseRequestResponse {
        let response = self.host.close_requested_response();
        if matches!(response, CloseRequestResponse::HideWindow) {
            // The close callback has returned, so app Cancel callbacks cannot reenter it.
            self.cancel_native_pointer_capture();
            self.host.state.borrow_mut().window_visible = false;
        }
        response
    }

    pub(in crate::ui::retained_host::host_contract::window::event_loop) fn handle_pointer_left(
        &mut self,
        platform_event: PlatformInputTranslation,
    ) {
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
        // A window leave is not a terminal native capture event: the owning
        // pointer may release beyond the window to detach a dragged tab.
        self.reject_input_outcome();
    }
}

fn play_stop_trace_enabled() -> bool {
    static TRACE_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *TRACE_ENABLED.get_or_init(|| {
        std::env::var("ZIRCON_TRACE_PLAY_STOP")
            .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    })
}

fn is_native_shift_f5(event: &KeyEvent, modifiers: ModifiersState) -> bool {
    event.state == ElementState::Pressed
        && modifiers.contains(ModifiersState::SHIFT)
        && (event.logical_key == Key::Named(NamedKey::F5)
            || event.physical_key == PhysicalKey::Code(KeyCode::F5))
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
impl UiHostWindow {
    pub(crate) fn dispatch_native_close_request_event_for_test(&self) -> CloseRequestResponse {
        UiHostWindowEventLoop::new(self.clone_strong()).handle_native_close_requested()
    }

    pub(crate) fn dispatch_native_focus_lost_event_for_test(&self) {
        UiHostWindowEventLoop::new(self.clone_strong()).handle_native_window_focus_lost();
    }

    pub(crate) fn dispatch_native_pointer_leave_event_for_test(
        &self,
        kind: winit::event::PointerKind,
        x: f32,
        y: f32,
    ) {
        let mut event_loop = UiHostWindowEventLoop::new(self.clone_strong());
        event_loop.last_pointer_position = Some((x, y));
        let event = WindowEvent::PointerLeft {
            device_id: None,
            position: Some(winit::dpi::PhysicalPosition::new(
                f64::from(x),
                f64::from(y),
            )),
            primary: true,
            kind,
        };
        let platform_event = event_loop.translate_platform_input_event(&event);
        event_loop.handle_pointer_left(platform_event);
    }
}

#[cfg(test)]
#[path = "tests/events.rs"]
mod tests;
