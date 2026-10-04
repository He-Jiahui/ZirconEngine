use winit::event::{Ime, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::super::surface_present::surface_resize_changes_viewport;
use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if !window_event_belongs_to_primary(
            self.window.as_ref().map(|window| window.id()),
            window_id,
        ) {
            return;
        }
        if window_event_requests_runtime_frame(&event, self.viewport_size) {
            self.request_runtime_frame();
        }
        match event {
            WindowEvent::CloseRequested => {
                self.handle_window_close_requested(event_loop);
            }
            WindowEvent::Destroyed => {
                self.handle_window_destroyed(event_loop);
            }
            WindowEvent::Moved(position) => {
                self.handle_window_moved(event_loop, position);
            }
            WindowEvent::Occluded(occluded) => {
                self.handle_window_occluded(event_loop, occluded);
            }
            WindowEvent::ThemeChanged(theme) => {
                self.handle_window_theme_changed(event_loop, theme);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_window_scale_factor_changed(event_loop, scale_factor);
            }
            WindowEvent::SurfaceResized(size) => {
                self.resize_surface_presenter(event_loop, size);
            }
            WindowEvent::Focused(focused) => {
                self.handle_window_focus_changed(event_loop, focused);
            }
            WindowEvent::PointerEntered { .. } => {
                self.handle_pointer_entered(event_loop);
            }
            WindowEvent::PointerLeft { position, kind, .. } => {
                self.handle_pointer_left(event_loop, position, kind);
            }
            WindowEvent::DragEntered { paths, .. } => {
                self.handle_files_hovered(event_loop, paths);
            }
            WindowEvent::DragDropped { paths, .. } => {
                self.handle_files_dropped(event_loop, paths);
            }
            WindowEvent::DragLeft { .. } => {
                self.handle_file_drag_cancelled(event_loop);
            }
            WindowEvent::PointerMoved {
                position, source, ..
            } => {
                self.handle_pointer_moved(event_loop, position, source);
            }
            WindowEvent::PointerButton {
                state,
                button,
                position,
                ..
            } => {
                self.handle_pointer_button(event_loop, state, button, position);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event_loop, event);
            }
            WindowEvent::Ime(ime) => {
                self.handle_window_ime_event(event_loop, ime);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(event_loop, delta);
            }
            WindowEvent::RedrawRequested => {
                self.present_redraw_frame(event_loop);
            }
            _ => {}
        }
    }

    /// The real Winit event-loop arm owns source arbitration. Native Windows IMM callbacks are
    /// consumed here before the legacy V1 adapter, so production delivery cannot bypass the V2
    /// producer through a helper-only test path or emit both streams for one composition.
    fn handle_window_ime_event(&mut self, event_loop: &dyn ActiveEventLoop, ime: Ime) {
        if native_ime_result_claims_event(self.handle_native_ime_window_event(event_loop, &ime)) {
            if matches!(ime, Ime::Disabled) {
                self.ime_input_admission.disable();
            }
            return;
        }
        self.handle_ime_input(event_loop, ime);
    }
}

fn native_ime_result_claims_event(result: Option<bool>) -> bool {
    // `Some(false)` means the requested V2 source attempted and failed dispatch. It still owns
    // this composition, so replaying it through V1 would produce a duplicate or reordered event.
    result.is_some()
}

fn window_event_belongs_to_primary<T: PartialEq>(
    primary_window_id: Option<T>,
    event_window_id: T,
) -> bool {
    primary_window_id.as_ref() == Some(&event_window_id)
}

fn window_event_requests_runtime_frame(
    event: &WindowEvent,
    viewport_size: ZrRuntimeViewportSizeV1,
) -> bool {
    match event {
        WindowEvent::SurfaceResized(size) => surface_resize_changes_viewport(viewport_size, *size),
        WindowEvent::Moved(_)
        | WindowEvent::ThemeChanged(_)
        | WindowEvent::ScaleFactorChanged { .. }
        | WindowEvent::PointerEntered { .. }
        | WindowEvent::PointerLeft { .. }
        | WindowEvent::DragEntered { .. }
        | WindowEvent::DragDropped { .. }
        | WindowEvent::DragLeft { .. }
        | WindowEvent::PointerMoved { .. }
        | WindowEvent::PointerButton { .. }
        | WindowEvent::KeyboardInput { .. }
        | WindowEvent::Ime(_)
        | WindowEvent::MouseWheel { .. } => true,
        WindowEvent::CloseRequested
        | WindowEvent::Destroyed
        | WindowEvent::Occluded(_)
        | WindowEvent::Focused(_)
        | WindowEvent::RedrawRequested => false,
        _ => false,
    }
}

#[cfg(test)]
#[path = "tests/dispatch.rs"]
mod tests;
