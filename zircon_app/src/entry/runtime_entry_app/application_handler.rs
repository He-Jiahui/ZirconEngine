use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use zircon_runtime_interface::{
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeViewportSizeV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
};

use super::{window_surface::runtime_native_surface_target, RuntimeEntryApp};
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
        let viewport_size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
        self.window = Some(window.clone());
        self.viewport_size = viewport_size;
        self.surface_present_enabled = self.bind_window_surface(window.as_ref()).unwrap_or(false);
        if self.resize_viewport(viewport_size).is_err() {
            event_loop.exit();
            return;
        }
        if !self.surface_present_enabled && !self.ensure_fallback_presenter(event_loop) {
            return;
        }
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
                let viewport_size =
                    ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
                if let Some(presenter) = self.presenter.as_mut() {
                    if presenter.resize(viewport_size).is_err() {
                        event_loop.exit();
                    }
                }
                if self.surface_present_enabled {
                    self.viewport_size = viewport_size;
                    match self.bind_current_window_surface() {
                        Ok(true) => {}
                        Ok(false) | Err(_) => {
                            self.disable_surface_present();
                        }
                    }
                }
                if self.resize_viewport(viewport_size).is_err() {
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
                if self.surface_present_enabled {
                    match self
                        .session
                        .present_viewport(self.viewport, self.viewport_size)
                    {
                        Ok(true) => return,
                        Ok(false) => {
                            self.disable_surface_present();
                        }
                        Err(_) => {
                            self.disable_surface_present();
                        }
                    }
                }
                if !self.ensure_fallback_presenter(event_loop) {
                    return;
                }
                if let Some(presenter) = self.presenter.as_mut() {
                    match self
                        .session
                        .capture_frame(self.viewport, self.viewport_size)
                    {
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

impl RuntimeEntryApp {
    fn bind_current_window_surface(
        &self,
    ) -> Result<bool, crate::entry::runtime_library::RuntimeLibraryError> {
        let Some(window) = self.window.as_ref() else {
            return Ok(false);
        };
        self.bind_window_surface(window.as_ref())
    }

    fn bind_window_surface(
        &self,
        window: &dyn Window,
    ) -> Result<bool, crate::entry::runtime_library::RuntimeLibraryError> {
        if !self.session.supports_viewport_surface_present() {
            return Ok(false);
        }
        let Some(target) = runtime_native_surface_target(window) else {
            return Ok(false);
        };
        self.session
            .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                self.viewport_size,
                target,
            ))
    }

    fn disable_surface_present(&mut self) {
        if self.surface_present_enabled {
            let _ = self.session.unbind_viewport_surface(self.viewport);
        }
        self.surface_present_enabled = false;
    }

    fn ensure_fallback_presenter(&mut self, event_loop: &dyn ActiveEventLoop) -> bool {
        if self.presenter.is_some() {
            return true;
        }
        let Some(window) = self.window.as_ref() else {
            return false;
        };
        match SoftbufferRuntimePresenter::new(window.clone()) {
            Ok(presenter) => {
                self.presenter = Some(presenter);
                true
            }
            Err(_) => {
                event_loop.exit();
                false
            }
        }
    }
}
