use crate::ui::retained_host::primitives::{CloseRequestResponse, PhysicalPosition, PhysicalSize};
use winit::event::{Ime, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_error;

use super::input::{pointer_button, pointer_button_state, scroll_delta};
use super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::native_pointer::{
    dispatch_native_pointer_button, dispatch_native_pointer_move, dispatch_native_pointer_scroll,
};
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract) fn window_event_impl(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                let response = self.host.close_requested_response();
                if matches!(response, CloseRequestResponse::HideWindow) {
                    self.host.state.borrow_mut().window_visible = false;
                    event_loop.exit();
                }
            }
            WindowEvent::SurfaceResized(size) => {
                self.host
                    .window()
                    .set_size(PhysicalSize::new(size.width, size.height));
                self.queue_redraw(HostRedrawRequest::full_frame_for_scenario(
                    UiPerfScenario::Startup,
                    true,
                ));
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
                if let Some(presenter) = self.presenter.as_mut() {
                    if let Err(error) = presenter.resize((size.width, size.height)) {
                        write_error(
                            "editor_host_window",
                            format!(
                                "presenter resize failed size={}x{}: {error}",
                                size.width, size.height
                            ),
                        );
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::Moved(position) => {
                self.host
                    .window()
                    .set_position(PhysicalPosition::new(position.x, position.y));
            }
            WindowEvent::PointerMoved { position, .. } => {
                self.last_pointer_position = Some((position.x as f32, position.y as f32));
                self.dispatch_pointer_result(dispatch_native_pointer_move(
                    &self.host,
                    position.x as f32,
                    position.y as f32,
                ));
            }
            WindowEvent::PointerButton {
                state,
                button,
                position,
                ..
            } => {
                self.last_pointer_position = Some((position.x as f32, position.y as f32));
                if let Some(state) = pointer_button_state(state) {
                    let result = dispatch_native_pointer_button(
                        &self.host,
                        state,
                        pointer_button(button),
                        position.x as f32,
                        position.y as f32,
                    );
                    self.dispatch_pointer_result(result);
                    self.sync_ime_allowed();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let metadata = self.next_input_metadata();
                let result = self.host.dispatch_native_keyboard_event(
                    &event,
                    self.current_modifiers,
                    metadata,
                    false,
                );
                self.dispatch_pointer_result(result);
                self.sync_ime_allowed();
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.current_modifiers = modifiers.state();
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                let result = self.host.dispatch_focused_text_insert(&text);
                self.dispatch_pointer_result(result);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.last_pointer_position.unwrap_or((0.0, 0.0));
                self.dispatch_pointer_result(dispatch_native_pointer_scroll(
                    &self.host,
                    x,
                    y,
                    scroll_delta(delta),
                ));
            }
            WindowEvent::RedrawRequested => {
                self.redraw_requested_impl(event_loop);
            }
            _ => {}
        }
    }
}
