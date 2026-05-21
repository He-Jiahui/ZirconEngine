use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: WindowEvent,
    ) {
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
                self.handle_ime_input(event_loop, ime);
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
}
