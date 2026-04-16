use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;
use zircon_graphics::ViewportInput;
use zircon_manager::{InputButton, InputEvent};
use zircon_math::{UVec2, Vec2};

use super::RuntimeEntryApp;
use crate::runtime_presenter::SoftbufferRuntimePresenter;

impl ApplicationHandler for RuntimeEntryApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Zircon Runtime")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
                )
                .expect("runtime window"),
        );
        let inner = window.inner_size();
        self.handle_viewport_input(ViewportInput::Resized(UVec2::new(
            inner.width,
            inner.height,
        )));
        self.window = Some(window.clone());
        self.presenter = Some(SoftbufferRuntimePresenter::new(window).expect("runtime presenter"));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(presenter) = self.presenter.as_mut() {
                    if presenter
                        .resize(UVec2::new(size.width, size.height))
                        .is_err()
                    {
                        event_loop.exit();
                    }
                }
                self.handle_viewport_input(ViewportInput::Resized(UVec2::new(
                    size.width,
                    size.height,
                )));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Vec2::new(position.x as f32, position.y as f32);
                self.input_manager.submit_event(InputEvent::CursorMoved {
                    x: self.cursor.x,
                    y: self.cursor.y,
                });
                self.handle_viewport_input(ViewportInput::PointerMoved(self.cursor));
            }
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
                    self.handle_viewport_input(ViewportInput::LeftPressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Left) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));
                    self.handle_viewport_input(ViewportInput::LeftReleased);
                }
                (ElementState::Pressed, MouseButton::Right) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonPressed(InputButton::MouseRight));
                    self.handle_viewport_input(ViewportInput::RightPressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Right) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonReleased(InputButton::MouseRight));
                    self.handle_viewport_input(ViewportInput::RightReleased);
                }
                (ElementState::Pressed, MouseButton::Middle) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonPressed(InputButton::MouseMiddle));
                    self.handle_viewport_input(ViewportInput::MiddlePressed(self.cursor));
                }
                (ElementState::Released, MouseButton::Middle) => {
                    self.input_manager
                        .submit_event(InputEvent::ButtonReleased(InputButton::MouseMiddle));
                    self.handle_viewport_input(ViewportInput::MiddleReleased);
                }
                _ => {}
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
                };
                self.input_manager
                    .submit_event(InputEvent::WheelScrolled { delta: amount });
                self.handle_viewport_input(ViewportInput::Scrolled(amount));
            }
            WindowEvent::RedrawRequested => {
                let extract = self.current_extract();
                let size = self.viewport_controller.viewport().size;
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}
