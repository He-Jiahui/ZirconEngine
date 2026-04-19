use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use zircon_runtime::core::math::{UVec2, Vec2};
use zircon_runtime::input::{InputButton, InputEvent};

use super::RuntimeEntryApp;
use crate::runtime_presenter::SoftbufferRuntimePresenter;

impl ApplicationHandler for RuntimeEntryApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = WindowAttributes::default()
            .with_title("Zircon Runtime")
            .with_surface_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        let window: Arc<dyn Window> = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::from(window),
            Err(_) => {
                event_loop.exit();
                return;
            }
        };
        let size = window.surface_size();
        self.resize_viewport(UVec2::new(size.width, size.height));
        self.window = Some(window.clone());
        self.presenter = Some(SoftbufferRuntimePresenter::new(window).expect("runtime presenter"));
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::SurfaceResized(size) => {
                if let Some(presenter) = self.presenter.as_mut() {
                    if presenter
                        .resize(UVec2::new(size.width, size.height))
                        .is_err()
                    {
                        event_loop.exit();
                    }
                }
                self.resize_viewport(UVec2::new(size.width, size.height));
            }
            WindowEvent::PointerMoved { position, .. } => {
                let cursor = Vec2::new(position.x as f32, position.y as f32);
                self.input_manager.submit_event(InputEvent::CursorMoved {
                    x: cursor.x,
                    y: cursor.y,
                });
                self.handle_cursor_moved(cursor);
            }
            WindowEvent::PointerButton {
                state,
                button,
                position,
                ..
            } => {
                self.cursor = Vec2::new(position.x as f32, position.y as f32);
                match (state, button.mouse_button()) {
                    (ElementState::Pressed, Some(MouseButton::Left)) => {
                        self.input_manager
                            .submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
                        self.handle_left_pressed();
                    }
                    (ElementState::Released, Some(MouseButton::Left)) => {
                        self.input_manager
                            .submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));
                        self.handle_left_released();
                    }
                    (ElementState::Pressed, Some(MouseButton::Right)) => {
                        self.input_manager
                            .submit_event(InputEvent::ButtonPressed(InputButton::MouseRight));
                        self.handle_right_pressed();
                    }
                    (ElementState::Released, Some(MouseButton::Right)) => {
                        self.input_manager
                            .submit_event(InputEvent::ButtonReleased(InputButton::MouseRight));
                        self.handle_right_released();
                    }
                    (ElementState::Pressed, Some(MouseButton::Middle)) => {
                        self.input_manager
                            .submit_event(InputEvent::ButtonPressed(InputButton::MouseMiddle));
                        self.handle_middle_pressed();
                    }
                    (ElementState::Released, Some(MouseButton::Middle)) => {
                        self.input_manager
                            .submit_event(InputEvent::ButtonReleased(InputButton::MouseMiddle));
                        self.handle_middle_released();
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
                };
                self.input_manager
                    .submit_event(InputEvent::WheelScrolled { delta: amount });
                self.handle_scroll(amount);
            }
            WindowEvent::RedrawRequested => {
                let extract = self.current_extract();
                let size = self.camera_controller.viewport_size();
                if let Some(presenter) = self.presenter.as_mut() {
                    match self.render_bridge.submit_extract(extract, size) {
                        Ok(Some(frame)) => {
                            if presenter.present(&frame).is_err() {
                                event_loop.exit();
                            }
                        }
                        Ok(None) => {}
                        Err(_) => {
                            event_loop.exit();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
