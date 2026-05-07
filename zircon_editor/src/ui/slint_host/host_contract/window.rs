use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use slint::{CloseRequestResponse, PhysicalPosition, PhysicalSize, PlatformError};
use winit::application::ApplicationHandler;
use winit::event::{
    ButtonSource, ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};
use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, write_error, DiagnosticLogLevel,
};
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::data::{
    FrameRect, HostClosePromptData, HostMenuStateData, HostPaneInteractionStateData,
    HostTextInputFocusData, HostWindowBootstrapData, HostWindowPresentationData,
};
use super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::globals::{
    HostContractGlobal, HostContractState, PaneSurfaceHostContext, UiHostContext,
};
use super::native_pointer::{
    dispatch_native_pointer_button, dispatch_native_pointer_move, dispatch_native_pointer_scroll,
    NativePointerButtonState,
};
use super::painter::{paint_host_frame, HostRgbaFrame};
use super::presenter::SoftbufferHostPresenter;
use super::redraw::{HostRedrawRequest, NativePointerDispatchResult};

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
        let mut state = self.state.borrow_mut();
        state.presentation_rebuild_count = state.presentation_rebuild_count.saturating_add(1);
        if state.presentation_rebuild_count <= 8
            || state.presentation_rebuild_count.is_power_of_two()
        {
            if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
                write_diagnostic_log(
                    "editor_host_window",
                    format!(
                        "set_host_presentation count={} project_path={} viewport_label={} status={} center={} document={} viewport={}",
                        state.presentation_rebuild_count,
                        presentation.host_shell.project_path,
                        presentation.host_shell.viewport_label,
                        presentation.host_shell.status_secondary,
                        frame_summary(&presentation.host_layout.center_band_frame),
                        frame_summary(&presentation.host_layout.document_region_frame),
                        frame_summary(&presentation.host_layout.viewport_content_frame)
                    ),
                );
            }
        }
        state.host_presentation = presentation;
    }

    pub(crate) fn get_host_presentation(&self) -> HostWindowPresentationData {
        let state = self.state.borrow();
        host_presentation_from_state(&state)
    }

    pub(crate) fn set_close_prompt(&self, prompt: HostClosePromptData) {
        let damage = {
            let mut state = self.state.borrow_mut();
            let current = state.host_presentation.close_prompt.clone();
            let damage = if current.visible {
                current.overlay_frame
            } else {
                prompt.overlay_frame.clone()
            };
            state.host_presentation.close_prompt = prompt;
            damage
        };
        self.queue_external_redraw(HostRedrawRequest::region(damage));
    }

    pub(crate) fn clear_close_prompt(&self) {
        self.set_close_prompt(HostClosePromptData::default());
    }

    pub(crate) fn set_host_refresh_invalidation_diagnostics(
        &self,
        diagnostics: HostInvalidationDiagnostics,
    ) {
        self.state.borrow_mut().refresh_invalidation_diagnostics = diagnostics;
    }

    fn refresh_invalidation_diagnostics(&self) -> HostInvalidationDiagnostics {
        self.state.borrow().refresh_invalidation_diagnostics
    }

    fn set_host_refresh_diagnostics_overlay(&self, diagnostics: HostRefreshDiagnostics) {
        self.set_host_refresh_diagnostics_overlay_text(diagnostics.overlay_text().into());
    }

    fn set_host_refresh_diagnostics_overlay_text(&self, overlay_text: slint::SharedString) {
        let mut state = self.state.borrow_mut();
        state.host_presentation.host_shell.debug_refresh_rate = overlay_text;
    }

    pub(crate) fn get_menu_state(&self) -> HostMenuStateData {
        self.state.borrow().menu_state.clone()
    }

    pub(crate) fn get_pane_interaction_state(&self) -> HostPaneInteractionStateData {
        self.state.borrow().pane_interaction_state.clone()
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

    pub(crate) fn request_redraw_region(&self, frame: FrameRect) {
        self.queue_external_redraw(HostRedrawRequest::region(frame));
    }

    pub(crate) fn text_input_focus_active(&self) -> bool {
        self.state.borrow().text_input_focus.is_active()
    }

    pub(crate) fn request_exit(&self) {
        let mut state = self.state.borrow_mut();
        state.window_visible = false;
        state.exit_requested = true;
    }

    #[cfg(test)]
    pub(crate) fn exit_requested_for_test(&self) -> bool {
        self.state.borrow().exit_requested
    }

    pub(crate) fn dispatch_focused_text_insert(&self, text: &str) -> NativePointerDispatchResult {
        let text: String = text.chars().filter(|ch| !ch.is_control()).collect();
        if text.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            value.push_str(&text);
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        self.dispatch_text_focus_value(focus.clone(), focus.edit_target_id(), value)
    }

    pub(crate) fn dispatch_focused_text_backspace(&self) -> NativePointerDispatchResult {
        let (focus, value) = {
            let mut state = self.state.borrow_mut();
            let focus = state.text_input_focus.clone();
            if !focus.is_active() {
                return NativePointerDispatchResult::idle();
            }
            let mut value = focus.value_text.to_string();
            if value.pop().is_none() {
                return NativePointerDispatchResult::idle();
            }
            state.text_input_focus.value_text = value.clone().into();
            (focus, value)
        };
        self.dispatch_text_focus_value(focus.clone(), focus.edit_target_id(), value)
    }

    fn dispatch_focused_key_event(&self, event: &KeyEvent) -> NativePointerDispatchResult {
        if event.state != ElementState::Pressed {
            return NativePointerDispatchResult::idle();
        }
        match &event.logical_key {
            Key::Named(NamedKey::Backspace) => self.dispatch_focused_text_backspace(),
            Key::Named(NamedKey::Escape) => {
                self.global::<UiHostContext>().clear_text_input_focus();
                NativePointerDispatchResult::idle()
            }
            Key::Named(NamedKey::Enter) => self.dispatch_focused_text_commit(),
            _ => event
                .text
                .as_deref()
                .map_or_else(NativePointerDispatchResult::idle, |text| {
                    self.dispatch_focused_text_insert(text)
                }),
        }
    }

    fn dispatch_focused_text_commit(&self) -> NativePointerDispatchResult {
        let focus = self.state.borrow().text_input_focus.clone();
        if !focus.is_active() || focus.commit_action_id.is_empty() {
            return NativePointerDispatchResult::idle();
        }
        self.dispatch_text_focus_value(
            focus.clone(),
            focus.commit_target_id(),
            focus.value_text.to_string(),
        )
    }

    fn dispatch_text_focus_value(
        &self,
        focus: super::data::HostTextInputFocusData,
        target_id: slint::SharedString,
        value: String,
    ) -> NativePointerDispatchResult {
        let value: slint::SharedString = value.into();
        let control_id = focus.control_id.clone();
        let pane_host = self.global::<PaneSurfaceHostContext>();
        match focus.dispatch_kind.as_str() {
            "welcome_text" => pane_host.invoke_welcome_control_changed(target_id, value),
            "showcase" => {
                pane_host.invoke_component_showcase_control_edited(control_id, target_id, value)
            }
            "inspector" => pane_host.invoke_inspector_control_changed(control_id, value),
            "asset" => pane_host.invoke_asset_control_changed("activity".into(), control_id, value),
            _ if !focus.edit_action_id.is_empty() => {
                pane_host.invoke_surface_control_edited(control_id, target_id, value)
            }
            _ => return NativePointerDispatchResult::idle(),
        }
        text_input_focus_redraw(&focus)
    }

    fn queue_external_redraw(&self, redraw: HostRedrawRequest) {
        if !redraw.request_redraw() {
            return;
        }
        let mut state = self.state.borrow_mut();
        let existing = std::mem::replace(
            &mut state.external_redraw_request,
            HostRedrawRequest::none(),
        );
        if existing.request_redraw() {
            state.external_redraw_coalesced_count =
                state.external_redraw_coalesced_count.saturating_add(1);
        }
        state.external_redraw_request = existing.merge(redraw);
        state.external_redraw_queued_count = state.external_redraw_queued_count.saturating_add(1);
    }

    fn take_external_redraw(&self) -> HostRedrawRequest {
        let mut state = self.state.borrow_mut();
        let redraw = std::mem::replace(
            &mut state.external_redraw_request,
            HostRedrawRequest::none(),
        );
        if redraw.request_redraw() {
            state.external_redraw_drained_count =
                state.external_redraw_drained_count.saturating_add(1);
        }
        redraw
    }

    #[cfg(test)]
    pub(crate) fn request_host_frame_for_test(&self) {
        self.request_frame_update();
    }

    #[cfg(test)]
    pub(crate) fn presentation_rebuild_count_for_test(&self) -> u64 {
        self.state.borrow().presentation_rebuild_count
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
    pub(crate) fn dispatch_native_primary_release_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Released,
            Some(UiPointerButton::Primary),
            x,
            y,
        )
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_secondary_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Secondary),
            x,
            y,
        )
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_middle_press_for_test(
        &self,
        x: f32,
        y: f32,
    ) -> NativePointerDispatchResult {
        dispatch_native_pointer_button(
            self,
            NativePointerButtonState::Pressed,
            Some(UiPointerButton::Middle),
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

    #[cfg(test)]
    pub(crate) fn dispatch_native_text_input_for_test(
        &self,
        text: &str,
    ) -> NativePointerDispatchResult {
        self.dispatch_focused_text_insert(text)
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_backspace_for_test(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_backspace()
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_enter_for_test(&self) -> NativePointerDispatchResult {
        self.dispatch_focused_text_commit()
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
        let presentation = host_presentation_from_state(&state);
        let frame = paint_host_frame(
            state.window_size.width,
            state.window_size.height,
            &presentation,
        );
        Ok(HostWindowSnapshot::from_rgba_frame(frame))
    }
}

fn host_presentation_from_state(state: &HostContractState) -> HostWindowPresentationData {
    let mut presentation = state.host_presentation.clone();
    presentation.menu_state = state.menu_state.clone();
    presentation.pane_interaction_state = state.pane_interaction_state.clone();
    presentation.text_input_focus = state.text_input_focus.clone();
    presentation.viewport_image = state.viewport_image.clone();
    presentation
}

fn text_input_focus_redraw(focus: &HostTextInputFocusData) -> NativePointerDispatchResult {
    let result = NativePointerDispatchResult::region(focus.edit_frame.clone());
    if result.request_redraw() {
        result
    } else {
        NativePointerDispatchResult::full_frame()
    }
}

struct UiHostWindowEventLoop {
    host: UiHostWindow,
    window: Option<Arc<dyn Window>>,
    presenter: Option<SoftbufferHostPresenter>,
    last_pointer_position: Option<(f32, f32)>,
    pending_redraw: HostRedrawRequest,
    ime_allowed: bool,
}

impl UiHostWindowEventLoop {
    fn new(host: UiHostWindow) -> Self {
        Self {
            host,
            window: None,
            presenter: None,
            last_pointer_position: None,
            pending_redraw: HostRedrawRequest::Full { frame_update: true },
            ime_allowed: false,
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

    fn dispatch_pointer_result(&mut self, result: NativePointerDispatchResult) {
        let redraw = result.redraw();
        if redraw.request_redraw() {
            self.queue_redraw(redraw);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    fn queue_redraw(&mut self, redraw: HostRedrawRequest) {
        self.pending_redraw = self.pending_redraw.clone().merge(redraw);
    }

    fn take_pending_redraw(&mut self) -> HostRedrawRequest {
        std::mem::replace(&mut self.pending_redraw, HostRedrawRequest::None)
    }

    fn drain_external_redraw_request(&mut self) {
        let redraw = self.host.take_external_redraw();
        if redraw.request_redraw() {
            self.queue_redraw(redraw);
            if let Some(window) = self.window.as_ref() {
                schedule_native_redraw(window.as_ref());
            }
        }
    }

    fn sync_ime_allowed(&mut self) {
        let allowed = self.host.text_input_focus_active();
        if self.ime_allowed == allowed {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            set_window_ime_allowed(window.as_ref(), allowed);
            self.ime_allowed = allowed;
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
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_window",
                format!("created native window size={}x{}", size.width, size.height),
            );
        }
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
                self.queue_redraw(HostRedrawRequest::Full { frame_update: true });
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
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
                let result = self.host.dispatch_focused_key_event(&event);
                self.dispatch_pointer_result(result);
                self.sync_ime_allowed();
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
                let redraw = self.take_pending_redraw();
                if !redraw.request_redraw() {
                    return;
                }
                if redraw.requires_frame_update() {
                    self.host.request_frame_update();
                }
                if let Some(presenter) = self.presenter.as_mut() {
                    let presentation = self.host.get_host_presentation();
                    let invalidation = self.host.refresh_invalidation_diagnostics();
                    match presenter.present(
                        &presentation,
                        redraw.damage_region().cloned(),
                        invalidation,
                    ) {
                        Ok(diagnostics) => {
                            self.host.set_host_refresh_diagnostics_overlay(diagnostics)
                        }
                        Err(_) => {
                            write_error("editor_host_window", "presenter present failed");
                            event_loop.exit();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {
        if self.host.state.borrow().exit_requested {
            _event_loop.exit();
            return;
        }
        if let Some(window) = self.window.as_ref() {
            self.sync_host_window_state(window.as_ref());
        }
        self.drain_external_redraw_request();
    }
}

fn schedule_native_redraw(window: &dyn Window) {
    window.request_redraw();
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

fn pointer_button(button: ButtonSource) -> Option<UiPointerButton> {
    match button.mouse_button() {
        Some(MouseButton::Left) => Some(UiPointerButton::Primary),
        Some(MouseButton::Right) => Some(UiPointerButton::Secondary),
        Some(MouseButton::Middle) => Some(UiPointerButton::Middle),
        _ => None,
    }
}

fn pointer_button_state(state: ElementState) -> Option<NativePointerButtonState> {
    match state {
        ElementState::Pressed => Some(NativePointerButtonState::Pressed),
        ElementState::Released => Some(NativePointerButtonState::Released),
    }
}

fn scroll_delta(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(position) => position.y as f32 * 0.1,
    }
}

#[allow(deprecated)]
fn set_window_ime_allowed(window: &dyn Window, allowed: bool) {
    window.set_ime_allowed(allowed);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_window_refresh_diagnostics_update_state_overlay_text() {
        let host = UiHostWindow::new().expect("host window should construct for state test");
        host.set_host_presentation(HostWindowPresentationData::default());

        let mut diagnostics = HostRefreshDiagnostics::default();
        diagnostics.record_present(96, false, true);
        host.set_host_refresh_diagnostics_overlay(diagnostics.with_invalidation_diagnostics(
            HostInvalidationDiagnostics {
                slow_path_rebuild_count: 2,
                render_rebuild_count: 3,
                paint_only_request_count: 4,
            },
        ));

        let presentation = host.get_host_presentation();
        let overlay = presentation.host_shell.debug_refresh_rate.as_str();
        assert!(overlay.contains("present 1"));
        assert!(overlay.contains("full 0"));
        assert!(overlay.contains("region 1"));
        assert!(overlay.contains("pixels 96"));
        assert!(overlay.contains("slow 2"));
        assert!(overlay.contains("render 3"));
        assert!(overlay.contains("paint-only 4"));
    }
}
