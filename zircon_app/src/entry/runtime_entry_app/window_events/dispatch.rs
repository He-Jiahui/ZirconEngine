use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::super::surface_present::surface_resize_changes_viewport;
use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: WindowEvent,
    ) {
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
mod tests {
    use super::window_event_requests_runtime_frame;
    use winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::WindowEvent,
    };
    use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

    #[test]
    fn redraw_delivery_does_not_schedule_another_reactive_frame() {
        assert!(!window_event_requests_runtime_frame(
            &WindowEvent::RedrawRequested,
            ZrRuntimeViewportSizeV1::new(1280, 720),
        ));
    }

    #[test]
    fn unhandled_window_noise_does_not_schedule_a_reactive_frame() {
        assert!(!window_event_requests_runtime_frame(
            &WindowEvent::DragMoved {
                position: PhysicalPosition::new(10.0, 20.0),
            },
            ZrRuntimeViewportSizeV1::new(1280, 720),
        ));
    }

    #[test]
    fn handled_window_events_schedule_frames_but_duplicate_resize_does_not() {
        let viewport_size = ZrRuntimeViewportSizeV1::new(1280, 720);
        assert!(window_event_requests_runtime_frame(
            &WindowEvent::Moved(PhysicalPosition::new(20, 30)),
            viewport_size,
        ));
        assert!(!window_event_requests_runtime_frame(
            &WindowEvent::Focused(false),
            viewport_size,
        ));
        assert!(!window_event_requests_runtime_frame(
            &WindowEvent::Occluded(true),
            viewport_size,
        ));
        assert!(!window_event_requests_runtime_frame(
            &WindowEvent::SurfaceResized(PhysicalSize::new(1280, 720)),
            viewport_size,
        ));
        assert!(window_event_requests_runtime_frame(
            &WindowEvent::SurfaceResized(PhysicalSize::new(1281, 720)),
            viewport_size,
        ));
    }
}
