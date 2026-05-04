use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use slint::{CloseRequestResponse, PhysicalPosition, PhysicalSize, PlatformError};
use winit::application::ApplicationHandler;
use winit::event::{ButtonSource, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use zircon_runtime_interface::ui::surface::UiPointerButton;
use zircon_runtime::diagnostic_log::{write_diagnostic_log, write_error};

use super::data::{FrameRect, HostWindowBootstrapData, HostWindowPresentationData};
use super::globals::{HostContractGlobal, HostContractState};
use super::native_pointer::{
    dispatch_native_pointer_button, dispatch_native_pointer_move, dispatch_native_pointer_scroll,
    NativePointerButtonState, NativePointerDispatchResult,
};
use super::painter::{paint_host_frame, HostRgbaFrame};
use super::presenter::SoftbufferHostPresenter;

const DEFAULT_HOST_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_HOST_WINDOW_HEIGHT: u32 = 720;

#[derive(Clone)]
pub(crate) struct UiHostWindow {
    state: Rc<RefCell<HostContractState>>,
}

impl UiHostWindow {
    pub(crate) fn new() -> Result<Self, PlatformError> {
        Ok(Self {
            state: Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
                DEFAULT_HOST_WINDOW_WIDTH,
                DEFAULT_HOST_WINDOW_HEIGHT,
            )))),
        })
    }

    pub(crate) fn clone_strong(&self) -> Self {
        self.clone()
    }

    pub(crate) fn show(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = true;
        Ok(())
    }

    pub(crate) fn hide(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = false;
        Ok(())
    }

    pub(crate) fn run(&self) -> Result<(), PlatformError> {
        let event_loop = EventLoop::new().map_err(platform_error)?;
        let app = UiHostWindowEventLoop::new(self.clone_strong());
        event_loop.run_app(app).map_err(platform_error)
    }

    pub(crate) fn window(&self) -> HostWindowHandle {
        HostWindowHandle {
            state: self.state.clone(),
        }
    }

    pub(crate) fn global<T>(&self) -> T
    where
        T: HostContractGlobal,
    {
        T::from_state(self.state.clone())
    }

    pub(crate) fn set_host_presentation(&self, presentation: HostWindowPresentationData) {
        write_diagnostic_log(
            "editor_host_window",
            format!(
                "set_host_presentation project_path={} viewport_label={} status={} center={} document={} viewport={}",
                presentation.host_shell.project_path,
                presentation.host_shell.viewport_label,
                presentation.host_shell.status_secondary,
                frame_summary(&presentation.host_layout.center_band_frame),
                frame_summary(&presentation.host_layout.document_region_frame),
                frame_summary(&presentation.host_layout.viewport_content_frame)
            ),
        );
        self.state.borrow_mut().host_presentation = presentation;
    }

    pub(crate) fn get_host_presentation(&self) -> HostWindowPresentationData {
        self.state.borrow().host_presentation.clone()
    }

    pub(crate) fn get_host_window_bootstrap(&self) -> HostWindowBootstrapData {
        let state = self.state.borrow();
        HostWindowBootstrapData {
            shell_frame: FrameRect {
                x: 0.0,
                y: 0.0,
                width: state.window_size.width as f32,
                height: state.window_size.height as f32,
            },
            viewport_content_frame: state
                .host_presentation
                .host_layout
                .viewport_content_frame
                .clone(),
        }
    }

    pub(crate) fn request_frame_update(&self) {
        self.global::<super::globals::UiHostContext>()
            .invoke_frame_requested();
    }

    #[cfg(test)]
    pub(crate) fn request_host_frame_for_test(&self) {
        self.request_frame_update();
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_pointer_move_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_move(self, x, y)
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_primary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Primary),
            x,
            y,
        )
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_pointer_scroll_for_test(
        &self,
        x: f32,
        y: f32,
        delta: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_scroll(self, x, y, delta)
    }
}

#[derive(Clone)]
pub(crate) struct HostWindowHandle {
    state: Rc<RefCell<HostContractState>>,
}

impl HostWindowHandle {
    pub(crate) fn set_position(&self, position: PhysicalPosition) {
        self.state.borrow_mut().window_position = position;
    }

    pub(crate) fn set_size(&self, size: PhysicalSize) {
        self.state.borrow_mut().window_size = size;
    }

    pub(crate) fn size(&self) -> PhysicalSize {
        self.state.borrow().window_size
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.state.borrow().window_visible
    }

    pub(crate) fn set_maximized(&self, maximized: bool) {
        self.state.borrow_mut().window_maximized = maximized;
    }

    pub(crate) fn is_maximized(&self) -> bool {
        self.state.borrow().window_maximized
    }

    pub(crate) fn on_close_requested(&self, callback: impl Fn() -> CloseRequestResponse + 'static) {
        self.state.borrow_mut().close_requested = Some(Rc::new(callback));
    }

    pub(crate) fn take_snapshot(&self) -> Result<HostWindowSnapshot, PlatformError> {
        let state = self.state.borrow();
        let frame = paint_host_frame(
            state.window_size.width,
            state.window_size.height,
            &state.host_presentation,
        );
        Ok(HostWindowSnapshot::from_rgba_frame(frame))
    }
}

struct UiHostWindowEventLoop {
    host: UiHostWindow,
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferHostPresenter>,
}

impl UiHostWindowEventLoop {
    fn new(host: UiHostWindow) -> Self {
        Self {
            host,
            window: None,
            presenter: None,
        }
    }

    fn sync_host_window_state(&self, window: &dyn Window) {
        let size = window.surface_size();
        let mut state = self.host.state.borrow_mut();
        state.window_size = PhysicalSize::new(size.width, size.height);
        state.window_visible = true;
        state.window_maximized = window.is_maximized();
        if let Ok(position) = window.outer_position() {
            state.window_position = PhysicalPosition::new(position.x, position.y);
        }
    }
}

impl ApplicationHandler for UiHostWindowEventLoop {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let size = self.host.window().size();
        let window_attributes = WindowAttributes::default()
            .with_title("Zircon Editor")
            .with_surface_size(winit::dpi::LogicalSize::new(
                size.width as f64,
                size.height as f64,
            ));
        let window: Arc<dyn Window> = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::from(window),
            Err(_) => {
                write_error("editor_host_window", "failed to create native window");
                event_loop.exit();
                return;
            }
        };
        self.sync_host_window_state(window.as_ref());
        let presenter = match SoftbufferHostPresenter::new(window.clone()) {
            Ok(presenter) => presenter,
            Err(_) => {
                write_error(
                    "editor_host_window",
                    "failed to create softbuffer presenter",
                );
                event_loop.exit();
                return;
            }
        };
        write_diagnostic_log(
            "editor_host_window",
            format!("created native window size={}x{}", size.width, size.height),
        );
        window.request_redraw();
        self.window = Some(window);
        self.presenter = Some(presenter);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                let response = self
                    .host
                    .state
                    .borrow()
                    .close_requested
                    .as_ref()
                    .map(|callback| callback())
                    .unwrap_or(CloseRequestResponse::HideWindow);
                if matches!(response, CloseRequestResponse::HideWindow) {
                    self.host.state.borrow_mut().window_visible = false;
                    event_loop.exit();
                }
            }
            WindowEvent::SurfaceResized(size) => {
                self.host
                    .window()
                    .set_size(PhysicalSize::new(size.width, size.height));
                self.host.request_frame_update();
                if let Some(presenter) = self.presenter.as_mut() {
                    if presenter.resize((size.width, size.height)).is_err() {
                        write_error(
                            "editor_host_window",
                            format!(
                                "presenter resize failed size={}x{}",
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
                if let Some(state) = pointer_button_state(state) {
                    self.dispatch_pointer_result(dispatch_native_pointer_button(
                        &self.host,
                        state,
                        pointer_button(button),
                        position.x as f32,
                        position.y as f32,
                    ));
                }
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
                self.host.request_frame_update();
                if let Some(presenter) = self.presenter.as_mut() {
                    let presentation = self.host.get_host_presentation();
                    if presenter.present(&presentation).is_err() {
                        write_error("editor_host_window", "presenter present failed");
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            self.sync_host_window_state(window.as_ref());
            window.request_redraw();
        }
    }
}

fn frame_summary(frame: &FrameRect) -> String {
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        frame.x, frame.y, frame.width, frame.height
    )
}

fn platform_error(error: impl std::fmt::Display) -> PlatformError {
    PlatformError::Other(error.to_string())
}

pub(crate) struct HostWindowSnapshot {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

impl HostWindowSnapshot {
    fn from_rgba_frame(frame: HostRgbaFrame) -> Self {
        Self {
            width: frame.width(),
            height: frame.height(),
            bytes: frame.into_bytes(),
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }
}
