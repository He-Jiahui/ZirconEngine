use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{
    ButtonSource, DeviceEvent, DeviceId, ElementState, Ime, MouseButton, MouseScrollDelta,
    PointerKind, PointerSource, WindowEvent,
};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, NativeKeyCode, PhysicalKey};
use winit::window::{
    ImeCapabilities, ImeEnableRequest, ImeHint, ImePurpose, ImeRequest, ImeRequestData,
    ImeRequestError, ImeSurroundingText, Theme, Window, WindowId,
};
use zircon_runtime::diagnostic_log::{write_log, write_warn};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeHostRequestV1,
    ZrRuntimeImeCursorAreaV1, ZrRuntimeImeHostRequestKindV1, ZrRuntimeImeHostRequestV1,
    ZrRuntimeImeSurroundingTextV1, ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_IME_CURSOR_HIDDEN_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
    ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_MOVED_V1, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
    ZR_RUNTIME_WINDOW_THEME_DARK_V1, ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
};

use super::{
    window_attributes::runtime_window_attributes, window_surface::runtime_native_surface_target,
    RuntimeEntryApp,
};
use crate::runtime_presenter::SoftbufferRuntimePresenter;

impl ApplicationHandler for RuntimeEntryApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "can_create_surfaces");
        if self.window.is_some() {
            return;
        }

        let window_attributes = runtime_window_attributes(&self.window_descriptor);
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
        if self.resize_viewport(viewport_size).is_err() {
            event_loop.exit();
            return;
        }
        match self.bind_window_surface(window.as_ref()) {
            Ok(true) => self.enable_surface_present(),
            Ok(false) => self.fallback_surface_present(),
            Err(_) => {
                self.fail_surface_present();
            }
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
        zircon_runtime::profile_scope!("app", "runtime_entry", "window_event");
        match event {
            WindowEvent::CloseRequested => {
                let event = ZrRuntimeEventV1::window_close_requested(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                    return;
                }
                event_loop.exit();
            }
            WindowEvent::Destroyed => {
                let event = ZrRuntimeEventV1::window_destroyed(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::Moved(position) => {
                let event = ZrRuntimeEventV1::window_moved(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    position.x,
                    position.y,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::Occluded(occluded) => {
                let event = ZrRuntimeEventV1::window_occluded(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    occluded,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::ThemeChanged(theme) => {
                let event = ZrRuntimeEventV1::window_theme_changed(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    window_theme(theme),
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let scale_factor = scale_factor as f32;
                let backend_event = ZrRuntimeEventV1::window_backend_scale_factor_changed(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    scale_factor,
                );
                if self.session.handle_event(backend_event).is_err() {
                    event_loop.exit();
                    return;
                }

                let logical_event = ZrRuntimeEventV1::window_scale_factor_changed(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    scale_factor,
                );
                if self.session.handle_event(logical_event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::SurfaceResized(size) => {
                let viewport_size =
                    ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
                if self.resize_viewport(viewport_size).is_err() {
                    event_loop.exit();
                    return;
                }
                if self.surface_present_enabled && !self.surface_present_failed {
                    match self.bind_current_window_surface() {
                        Ok(true) => self.enable_surface_present(),
                        Ok(false) => self.fail_surface_present(),
                        Err(_) => self.fail_surface_present(),
                    }
                }
                if let Some(presenter) = self.presenter.as_mut() {
                    if presenter.resize(viewport_size).is_err() {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                let state = if focused {
                    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1
                } else {
                    ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1
                };
                let event = ZrRuntimeEventV1::lifecycle(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    state,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::PointerEntered { .. } => {
                let event =
                    ZrRuntimeEventV1::cursor_entered(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::PointerLeft { position, kind, .. } => {
                let event =
                    ZrRuntimeEventV1::cursor_left(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
                if let Some(pointer_id) = pointer_kind_touch_id(kind) {
                    let event = ZrRuntimeEventV1::touch(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        pointer_id,
                        ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
                        position
                            .map(|position| position.x as f32)
                            .unwrap_or_default(),
                        position
                            .map(|position| position.y as f32)
                            .unwrap_or_default(),
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::DragEntered { paths, .. } => {
                for path in paths {
                    let path_text = path.to_string_lossy().to_string();
                    let event = ZrRuntimeEventV1::file_hovered(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        byte_slice(path_text.as_str()),
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                        return;
                    }
                }
            }
            WindowEvent::DragDropped { paths, .. } => {
                for path in paths {
                    let path_text = path.to_string_lossy().to_string();
                    let event = ZrRuntimeEventV1::file_dropped(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        byte_slice(path_text.as_str()),
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                        return;
                    }
                }
            }
            WindowEvent::DragLeft { .. } => {
                let event = ZrRuntimeEventV1::file_drag_cancelled(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::PointerMoved {
                position, source, ..
            } => {
                let event = if let Some(pointer_id) = pointer_source_touch_id(&source) {
                    ZrRuntimeEventV1::touch(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        pointer_id,
                        ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
                        position.x as f32,
                        position.y as f32,
                    )
                } else {
                    ZrRuntimeEventV1::pointer_moved(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        position.x as f32,
                        position.y as f32,
                    )
                };
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
                if let Some((pointer_id, phase)) = touch_button_phase(&button, state) {
                    let event = ZrRuntimeEventV1::touch(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        pointer_id,
                        phase,
                        position.x as f32,
                        position.y as f32,
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                } else if let (Some(button), Some(state)) =
                    (mouse_button(button), button_state(state))
                {
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
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(action) = key_action(event.state) {
                    let payload = event
                        .text
                        .as_ref()
                        .map(|text| ZrByteSlice {
                            data: text.as_bytes().as_ptr(),
                            len: text.len(),
                        })
                        .unwrap_or_else(ZrByteSlice::empty);
                    let runtime_event = ZrRuntimeEventV1::keyboard(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        action,
                        physical_key_code(&event.physical_key),
                        0,
                        payload,
                    );
                    if self.session.handle_event(runtime_event).is_err() {
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::Ime(ime) => match ime {
                Ime::Enabled => {
                    let event =
                        ZrRuntimeEventV1::ime_enabled(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                }
                Ime::Disabled => {
                    let event = ZrRuntimeEventV1::ime_disabled(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                }
                Ime::Preedit(value, cursor) => {
                    let (cursor_start, cursor_end) = cursor
                        .map(|(start, end)| (usize_to_u32(start), usize_to_u32(end)))
                        .unwrap_or((
                            ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
                            ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
                        ));
                    let event = ZrRuntimeEventV1::ime_preedit(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        byte_slice(value.as_str()),
                        cursor_start,
                        cursor_end,
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                }
                Ime::Commit(value) => {
                    let event = ZrRuntimeEventV1::ime_commit(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        byte_slice(value.as_str()),
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                }
                Ime::DeleteSurrounding {
                    before_bytes,
                    after_bytes,
                } => {
                    let event = ZrRuntimeEventV1::ime_delete_surrounding(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        self.viewport,
                        usize_to_u32(before_bytes),
                        usize_to_u32(after_bytes),
                    );
                    if self.session.handle_event(event).is_err() {
                        event_loop.exit();
                    }
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let (unit, x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        (ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, x, y)
                    }
                    MouseScrollDelta::PixelDelta(position) => (
                        ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
                        position.x as f32,
                        position.y as f32,
                    ),
                };
                let event = ZrRuntimeEventV1::mouse_wheel_delta(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    self.viewport,
                    unit,
                    x,
                    y,
                );
                if self.session.handle_event(event).is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                zircon_runtime::profile_frame!("app", "runtime_redraw");
                zircon_runtime::profile_scope!("app", "runtime_entry", "redraw_requested");
                if self.surface_present_enabled && !self.surface_present_failed {
                    match self
                        .session
                        .present_viewport(self.viewport, self.viewport_size)
                    {
                        Ok(true) => return,
                        Ok(false) => {
                            self.fail_surface_present();
                        }
                        Err(_) => {
                            self.fail_surface_present();
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

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "about_to_wait");
        self.apply_event_loop_policy(event_loop);
        #[cfg(feature = "gamepad-gilrs")]
        self.poll_gamepads(event_loop);
        if self.session.tick_frame().is_err() {
            event_loop.exit();
            return;
        }
        if !self.apply_runtime_host_requests(event_loop) {
            return;
        }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _device_id: Option<DeviceId>,
        event: DeviceEvent,
    ) {
        zircon_runtime::profile_scope!("app", "runtime_entry", "device_event");
        let DeviceEvent::PointerMotion {
            delta: (delta_x, delta_y),
        } = event
        else {
            return;
        };
        let event = ZrRuntimeEventV1::mouse_motion(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            delta_x as f32,
            delta_y as f32,
        );
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }
}

fn pointer_source_touch_id(source: &PointerSource) -> Option<u64> {
    match source {
        PointerSource::Touch { finger_id, .. } => Some(finger_id.into_raw() as u64),
        _ => None,
    }
}

fn pointer_kind_touch_id(kind: PointerKind) -> Option<u64> {
    match kind {
        PointerKind::Touch(finger_id) => Some(finger_id.into_raw() as u64),
        _ => None,
    }
}

fn touch_button_phase(button: &ButtonSource, state: ElementState) -> Option<(u64, u32)> {
    let ButtonSource::Touch { finger_id, .. } = button else {
        return None;
    };
    let phase = match state {
        ElementState::Pressed => ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
        ElementState::Released => ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    };
    Some((finger_id.into_raw() as u64, phase))
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

fn key_action(state: ElementState) -> Option<u32> {
    match state {
        ElementState::Pressed => Some(ZR_RUNTIME_KEY_ACTION_PRESSED_V1),
        ElementState::Released => Some(ZR_RUNTIME_KEY_ACTION_RELEASED_V1),
    }
}

fn window_theme(theme: Theme) -> u32 {
    match theme {
        Theme::Light => ZR_RUNTIME_WINDOW_THEME_LIGHT_V1,
        Theme::Dark => ZR_RUNTIME_WINDOW_THEME_DARK_V1,
    }
}

fn physical_key_code(key: &PhysicalKey) -> u32 {
    match key {
        PhysicalKey::Code(code) => match code {
            KeyCode::ShiftLeft | KeyCode::ShiftRight => 16,
            KeyCode::ControlLeft | KeyCode::ControlRight => 17,
            KeyCode::AltLeft | KeyCode::AltRight => 18,
            _ => stable_key_code(format!("{code:?}").as_bytes()),
        },
        PhysicalKey::Unidentified(native) => native_key_code(native),
    }
}

fn native_key_code(native: &NativeKeyCode) -> u32 {
    match *native {
        NativeKeyCode::Unidentified => 0,
        NativeKeyCode::Android(code) | NativeKeyCode::Xkb(code) => code,
        NativeKeyCode::MacOS(code) | NativeKeyCode::Windows(code) => code as u32,
    }
}

fn stable_key_code(bytes: &[u8]) -> u32 {
    const FNV_OFFSET: u32 = 2_166_136_261;
    const FNV_PRIME: u32 = 16_777_619;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash.max(1)
}

fn byte_slice(value: &str) -> ZrByteSlice {
    ZrByteSlice {
        data: value.as_bytes().as_ptr(),
        len: value.len(),
    }
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX - 1)
}

fn apply_runtime_host_request(window: &dyn Window, request: ZrRuntimeHostRequestV1) {
    let result = match request {
        ZrRuntimeHostRequestV1::Ime(request) => apply_runtime_ime_host_request(window, request),
    };
    if let Err(error) = result {
        write_warn(
            "runtime_ime",
            format!("runtime_ime_host_request_failed:{error}"),
        );
    }
}

fn apply_runtime_ime_host_request(
    window: &dyn Window,
    request: ZrRuntimeImeHostRequestV1,
) -> Result<(), ImeRequestError> {
    match request.kind {
        ZrRuntimeImeHostRequestKindV1::Enable => enable_window_ime(window),
        ZrRuntimeImeHostRequestKindV1::Disable => window.request_ime_update(ImeRequest::Disable),
        ZrRuntimeImeHostRequestKindV1::SetCursorArea => {
            if let Some(area) = request.cursor_area {
                window.request_ime_update(ImeRequest::Update(
                    ImeRequestData::default()
                        .with_cursor_area(ime_logical_position(area), ime_logical_size(area)),
                ))
            } else {
                write_warn("runtime_ime", "runtime_ime_cursor_area_missing");
                Ok(())
            }
        }
        ZrRuntimeImeHostRequestKindV1::SetSurroundingText => {
            if let Some(text) = request.surrounding_text {
                update_window_ime_surrounding_text(window, text)
            } else {
                write_warn("runtime_ime", "runtime_ime_surrounding_text_missing");
                Ok(())
            }
        }
    }
}

fn enable_window_ime(window: &dyn Window) -> Result<(), ImeRequestError> {
    let capabilities = ImeCapabilities::new()
        .with_hint_and_purpose()
        .with_cursor_area()
        .with_surrounding_text();
    let Some(request) = ImeEnableRequest::new(capabilities, default_ime_request_data()) else {
        write_warn("runtime_ime", "runtime_ime_enable_request_invalid");
        return Ok(());
    };
    match window.request_ime_update(ImeRequest::Enable(request)) {
        Err(ImeRequestError::AlreadyEnabled) => Ok(()),
        result => result,
    }
}

fn default_ime_request_data() -> ImeRequestData {
    ImeRequestData::default()
        .with_hint_and_purpose(ImeHint::NONE, ImePurpose::Normal)
        .with_cursor_area(
            LogicalPosition::new(0.0, 0.0).into(),
            LogicalSize::new(1.0, 1.0).into(),
        )
        .with_surrounding_text(
            ImeSurroundingText::new(String::new(), 0, 0)
                .expect("empty IME surrounding text is valid"),
        )
}

fn update_window_ime_surrounding_text(
    window: &dyn Window,
    text: ZrRuntimeImeSurroundingTextV1,
) -> Result<(), ImeRequestError> {
    let Ok(text) = ImeSurroundingText::new(text.value, text.cursor, text.anchor) else {
        write_warn("runtime_ime", "runtime_ime_surrounding_text_invalid");
        return Ok(());
    };
    window.request_ime_update(ImeRequest::Update(
        ImeRequestData::default().with_surrounding_text(text),
    ))
}

fn ime_logical_position(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Position {
    LogicalPosition::new(area.x as f64, area.y as f64).into()
}

fn ime_logical_size(area: ZrRuntimeImeCursorAreaV1) -> winit::dpi::Size {
    LogicalSize::new(area.width as f64, area.height as f64).into()
}

impl RuntimeEntryApp {
    fn apply_runtime_host_requests(&mut self, event_loop: &dyn ActiveEventLoop) -> bool {
        let requests = match self.session.drain_host_requests() {
            Ok(requests) => requests,
            Err(_) => {
                event_loop.exit();
                return false;
            }
        };
        let Some(window) = self.window.as_ref() else {
            return true;
        };
        for request in requests {
            apply_runtime_host_request(window.as_ref(), request);
        }
        true
    }

    fn bind_current_window_surface(
        &mut self,
    ) -> Result<bool, crate::entry::runtime_library::RuntimeLibraryError> {
        let Some(window) = self.window.clone() else {
            return Ok(false);
        };
        self.bind_window_surface(window.as_ref())
    }

    fn bind_window_surface(
        &mut self,
        window: &dyn Window,
    ) -> Result<bool, crate::entry::runtime_library::RuntimeLibraryError> {
        if !self.session.supports_viewport_surface_present() {
            return Ok(false);
        }
        let Some(target) = runtime_native_surface_target(window) else {
            return Ok(false);
        };
        self.surface_present_attempted = true;
        self.session
            .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                self.viewport_size,
                target,
            ))
    }

    fn disable_surface_present(&mut self) {
        if self.surface_present_enabled || self.surface_present_attempted {
            let _ = self.session.unbind_viewport_surface(self.viewport);
        }
        self.surface_present_enabled = false;
        self.surface_present_attempted = false;
    }

    fn enable_surface_present(&mut self) {
        self.surface_present_enabled = true;
        self.surface_present_failed = false;
        write_log("runtime_surface_present", "runtime_surface_present_enabled");
    }

    fn fallback_surface_present(&mut self) {
        self.disable_surface_present();
        write_log(
            "runtime_surface_present",
            "runtime_surface_present_fallback",
        );
    }

    fn fail_surface_present(&mut self) {
        self.surface_present_failed = true;
        write_warn("runtime_surface_present", "runtime_surface_present_failed");
        self.fallback_surface_present();
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

impl Drop for RuntimeEntryApp {
    fn drop(&mut self) {
        self.disable_surface_present();
    }
}
