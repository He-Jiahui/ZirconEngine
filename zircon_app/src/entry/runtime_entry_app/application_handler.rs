use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportSizeV1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

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
        if self
            .resize_viewport(ZrRuntimeViewportSizeV1::new(size.width, size.height))
            .is_err()
        {
            event_loop.exit();
            return;
        }
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
                        .resize(ZrRuntimeViewportSizeV1::new(size.width, size.height))
                        .is_err()
                    {
                        event_loop.exit();
                    }
                }
                if self
                    .resize_viewport(ZrRuntimeViewportSizeV1::new(size.width, size.height))
                    .is_err()
                {
                    event_loop.exit();
                }
            }
            WindowEvent::PointerMoved { position, .. } => {
                let event = ZrRuntimeEventV1::pointer_moved(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    position.x as f32,
                    position.y as f32,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::PointerButton {
                state,
                button,
                position,
                ..
            } => {
                if let (Some(button), Some(state)) = (mouse_button(button), button_state(state)) {
                    let event = ZrRuntimeEventV1::mouse_button(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        button,
                        state,
                        position.x as f32,
                        position.y as f32,
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
                };
                let event = ZrRuntimeEventV1::mouse_wheel(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    amount,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(presenter) = self.presenter.as_mut() {
                    match self.session.capture_frame(self.viewport, self.viewport_size) {
                        Ok(frame) => {
                            if presenter.present(&frame).is_err() {
                                event_loop.exit();
                            }
                        }
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

fn mouse_button(button: ButtonSource) -> Option<u32> {
    match button.mouse_button() {
        Some(MouseButton::Left) => Some(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1),
        Some(MouseButton::Right) => Some(ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1),
        Some(MouseButton::Middle) => Some(ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1),
        _ => None,
    }
}

fn button_state(state: ElementState) -> Option<u32> {
    match state {
        ElementState::Pressed => Some(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1),
        ElementState::Released => Some(ZR_RUNTIME_BUTTON_STATE_RELEASED_V1),
    }
}
