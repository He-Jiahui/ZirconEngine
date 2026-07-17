use zircon_runtime_interface::{
    ui::accessibility::UiAccessibilityActionRequest, ZrRuntimeEventV1, ZrStatus,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1, ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1,
    ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1, ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1, ZR_RUNTIME_EVENT_KIND_IME_V1,
    ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
    ZR_RUNTIME_EVENT_KIND_TOUCH_V1, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
    ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1, ZR_RUNTIME_FILE_DRAG_CANCELLED_V1,
    ZR_RUNTIME_FILE_DRAG_DROPPED_V1, ZR_RUNTIME_FILE_DRAG_HOVERED_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1,
    ZR_RUNTIME_IME_STATE_COMMIT_V1, ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1,
    ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1, ZR_RUNTIME_IME_STATE_DISABLED_V1,
    ZR_RUNTIME_IME_STATE_ENABLED_V1, ZR_RUNTIME_IME_STATE_PREEDIT_V1,
    ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1, ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1,
    ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_TEXT_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1, ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1, ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
    ZR_RUNTIME_WINDOW_STATUS_MOVED_V1, ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1, ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
};

use crate::core::framework::input::{
    FileDragDropEvent, GamepadConnectionInfo, GamepadId, ImeDeleteSurrounding, ImeEvent,
    ImeHostRequest, ImePreedit, InputEvent, MouseWheelEvent, WindowStatusEvent,
};
use crate::core::math::Vec2;

use super::input_events::{
    gamepad_axis, gamepad_button, ime_cursor, ime_cursor_area, ime_surrounding_text, input_button,
    keyboard_logical_key, mouse_scroll_unit, nonzero_u16, touch_phase, window_bool,
    window_scale_factor, window_theme,
};
use super::menu::{runtime_session_menu_action_at, write_runtime_menu_action};
use super::status::{invalid_argument, not_found};
use super::{RuntimeDynamicSession, DEFAULT_VIEWPORT};

impl RuntimeDynamicSession {
    pub(super) fn handle_event(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        if event.viewport != DEFAULT_VIEWPORT {
            return not_found(b"runtime viewport not found");
        }
        match event.kind {
            ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1 => {
                self.resize_viewport(crate::core::math::UVec2::new(
                    event.size.width,
                    event.size.height,
                ));
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1 => {
                let cursor = Vec2::new(event.x, event.y);
                self.submit_input_event(InputEvent::CursorMoved {
                    x: cursor.x,
                    y: cursor.y,
                });
                self.handle_cursor_moved(cursor);
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1 => {
                self.submit_input_event(InputEvent::CursorEntered);
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1 => {
                self.submit_input_event(InputEvent::CursorLeft);
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1 => self.handle_mouse_button(event),
            ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1 => self.handle_mouse_wheel(event),
            ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1 => {
                self.submit_input_event(InputEvent::MouseMotion {
                    delta_x: event.x,
                    delta_y: event.y,
                });
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1 => self.handle_lifecycle(event),
            ZR_RUNTIME_EVENT_KIND_TOUCH_V1 => self.handle_touch(event),
            ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1 => self.handle_keyboard(event),
            ZR_RUNTIME_EVENT_KIND_IME_V1 => self.handle_ime(event),
            ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1 => self.handle_file_drag_drop(event),
            ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1 => self.handle_window_status(event),
            ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1 => self.handle_gamepad_connection(event),
            ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1 => self.handle_gamepad_button(event),
            ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1 => self.handle_gamepad_axis(event),
            ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1 => {
                self.handle_accessibility_action(event)
            }
            _ => invalid_argument(b"unknown runtime event kind"),
        }
    }

    fn handle_accessibility_action(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        if serde_json::from_slice::<UiAccessibilityActionRequest>(payload).is_err() {
            return invalid_argument(b"invalid accessibility action payload");
        }
        not_found(
            b"runtime UI surface accessibility action dispatch unavailable in dynamic preview",
        )
    }

    fn handle_cursor_moved(&mut self, position: Vec2) {
        self.cursor = position;
        self.level
            .with_world_mut(|world| self.camera_controller.pointer_moved(world, position));
    }

    fn handle_mouse_button(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        self.cursor = Vec2::new(event.x, event.y);
        let Some(button) = input_button(event.button) else {
            return invalid_argument(b"unknown runtime mouse button");
        };
        match event.state {
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => {
                self.submit_input_event(InputEvent::ButtonPressed(button));
                self.handle_pressed(event.button);
            }
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => {
                self.submit_input_event(InputEvent::ButtonReleased(button));
                self.handle_released(event.button);
            }
            _ => return invalid_argument(b"unknown runtime button state"),
        }
        ZrStatus::ok()
    }

    fn handle_mouse_wheel(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let unit = match mouse_scroll_unit(event.state) {
            Ok(unit) => unit,
            Err(status) => return status,
        };
        let Some(unit) = unit else {
            self.submit_input_event(InputEvent::WheelScrolled { delta: event.delta });
            self.handle_scroll(event.delta);
            return ZrStatus::ok();
        };
        let (delta_x, delta_y) = if event.button == ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1 {
            (
                f32::from_bits(event.key_code),
                f32::from_bits(event.scan_code),
            )
        } else {
            (event.x, event.y)
        };
        if !delta_x.is_finite() || !delta_y.is_finite() {
            return invalid_argument(b"invalid runtime mouse wheel delta");
        }
        let wheel = MouseWheelEvent::new(unit, delta_x, delta_y);
        self.submit_input_event(InputEvent::MouseWheel(wheel));
        self.handle_scroll(wheel.vertical_line_delta());
        ZrStatus::ok()
    }

    fn handle_lifecycle(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        match event.state {
            ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1
            | ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1
            | ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1 => {
                self.submit_input_event(InputEvent::KeyboardFocusLost);
            }
            _ => {}
        }
        ZrStatus::ok()
    }

    fn handle_touch(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let cursor = Vec2::new(event.x, event.y);
        let Some(phase) = touch_phase(event.state) else {
            return invalid_argument(b"unknown runtime touch phase");
        };
        self.submit_input_event(InputEvent::CursorMoved {
            x: cursor.x,
            y: cursor.y,
        });
        self.submit_input_event(InputEvent::Touch {
            id: event.pointer_id,
            phase,
            x: cursor.x,
            y: cursor.y,
        });
        match event.state {
            ZR_RUNTIME_TOUCH_PHASE_STARTED_V1 => {
                self.handle_cursor_moved(cursor);
                self.handle_pressed(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1);
            }
            ZR_RUNTIME_TOUCH_PHASE_MOVED_V1 => self.handle_cursor_moved(cursor),
            ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 | ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1 => {
                self.cursor = cursor;
                self.handle_released(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1);
            }
            _ => unreachable!("touch phase was validated before dispatch"),
        }
        ZrStatus::ok()
    }

    fn handle_keyboard(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        let text = if payload.is_empty() {
            None
        } else {
            String::from_utf8(payload.to_vec()).ok()
        };
        if event.button == ZR_RUNTIME_KEY_ACTION_TEXT_V1 {
            if let Some(text) = text {
                self.submit_input_event(InputEvent::KeyboardInput {
                    key_code: event.key_code,
                    logical_key: None,
                    text: Some(text),
                    pressed: false,
                    repeat: false,
                });
            }
            return ZrStatus::ok();
        }

        let pressed = match event.button {
            ZR_RUNTIME_KEY_ACTION_PRESSED_V1 => true,
            ZR_RUNTIME_KEY_ACTION_RELEASED_V1 => false,
            _ => return ZrStatus::ok(),
        };
        self.submit_input_event(InputEvent::KeyboardInput {
            key_code: event.key_code,
            logical_key: keyboard_logical_key(event.key_code, text.as_deref()),
            text,
            pressed,
            repeat: false,
        });
        ZrStatus::ok()
    }

    fn handle_ime(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        let text_payload = || match String::from_utf8(payload.to_vec()) {
            Ok(text) => Ok(text),
            Err(_) => Err(invalid_argument(b"invalid runtime ime payload")),
        };
        let input_event = match event.state {
            ZR_RUNTIME_IME_STATE_ENABLED_V1 => InputEvent::Ime(ImeEvent::Enabled),
            ZR_RUNTIME_IME_STATE_DISABLED_V1 => InputEvent::Ime(ImeEvent::Disabled),
            ZR_RUNTIME_IME_STATE_PREEDIT_V1 => InputEvent::Ime(ImeEvent::Preedit(ImePreedit {
                value: match text_payload() {
                    Ok(text) => text,
                    Err(status) => return status,
                },
                cursor: ime_cursor(event),
            })),
            ZR_RUNTIME_IME_STATE_COMMIT_V1 => match text_payload() {
                Ok(text) => InputEvent::Ime(ImeEvent::Commit(text)),
                Err(status) => return status,
            },
            ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1 => {
                InputEvent::Ime(ImeEvent::DeleteSurrounding(ImeDeleteSurrounding::new(
                    event.key_code as usize,
                    event.scan_code as usize,
                )))
            }
            ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1 => {
                InputEvent::ImeHostRequest(ImeHostRequest::Enable)
            }
            ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1 => {
                InputEvent::ImeHostRequest(ImeHostRequest::Disable)
            }
            ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1 => match ime_cursor_area(event) {
                Some(area) => InputEvent::ImeHostRequest(ImeHostRequest::SetCursorArea(area)),
                None => return invalid_argument(b"invalid runtime ime cursor area"),
            },
            ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1 => {
                match ime_surrounding_text(event, payload) {
                    Ok(text) => {
                        InputEvent::ImeHostRequest(ImeHostRequest::SetSurroundingText(text))
                    }
                    Err(status) => return status,
                }
            }
            _ => return invalid_argument(b"unknown runtime ime state"),
        };
        self.submit_input_event(input_event);
        ZrStatus::ok()
    }

    fn handle_file_drag_drop(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = unsafe { event.payload.as_slice() };
        let path_payload = || match String::from_utf8(payload.to_vec()) {
            Ok(path) => Ok(path),
            Err(_) => Err(invalid_argument(b"invalid runtime file drag path")),
        };
        let file_event = match event.state {
            ZR_RUNTIME_FILE_DRAG_HOVERED_V1 => match path_payload() {
                Ok(path) => FileDragDropEvent::Hovered { path },
                Err(status) => return status,
            },
            ZR_RUNTIME_FILE_DRAG_DROPPED_V1 => match path_payload() {
                Ok(path) => FileDragDropEvent::Dropped { path },
                Err(status) => return status,
            },
            ZR_RUNTIME_FILE_DRAG_CANCELLED_V1 => FileDragDropEvent::Cancelled,
            _ => return invalid_argument(b"unknown runtime file drag state"),
        };
        self.submit_input_event(InputEvent::FileDragDrop(file_event));
        ZrStatus::ok()
    }

    fn handle_window_status(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let window_event = match event.state {
            ZR_RUNTIME_WINDOW_STATUS_MOVED_V1 => WindowStatusEvent::Moved {
                x: event.x as i32,
                y: event.y as i32,
            },
            ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1 => match window_bool(event.button) {
                Some(occluded) => WindowStatusEvent::Occluded(occluded),
                None => return invalid_argument(b"unknown runtime window bool"),
            },
            ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1 => {
                WindowStatusEvent::ThemeChanged(window_theme(event.button))
            }
            ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1 => {
                WindowStatusEvent::ScaleFactorChanged {
                    scale_factor: match window_scale_factor(event.delta) {
                        Some(scale_factor) => scale_factor,
                        None => return invalid_argument(b"invalid runtime window scale factor"),
                    },
                }
            }
            ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1 => {
                WindowStatusEvent::BackendScaleFactorChanged {
                    scale_factor: match window_scale_factor(event.delta) {
                        Some(scale_factor) => scale_factor,
                        None => return invalid_argument(b"invalid runtime window scale factor"),
                    },
                }
            }
            ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1 => WindowStatusEvent::CloseRequested,
            ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1 => WindowStatusEvent::Destroyed,
            _ => return invalid_argument(b"unknown runtime window status"),
        };
        self.submit_input_event(InputEvent::WindowStatus(window_event));
        ZrStatus::ok()
    }

    fn handle_gamepad_connection(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let connected = match event.state {
            ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1 => true,
            ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1 => false,
            _ => return invalid_argument(b"unknown runtime gamepad connection state"),
        };
        let payload = unsafe { event.payload.as_slice() };
        let name = if payload.is_empty() {
            None
        } else {
            String::from_utf8(payload.to_vec()).ok()
        };
        self.submit_input_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
            gamepad: GamepadId(event.pointer_id),
            connected,
            name,
            vendor_id: nonzero_u16(event.key_code),
            product_id: nonzero_u16(event.scan_code),
        }));
        ZrStatus::ok()
    }

    fn handle_gamepad_button(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let pressed = match event.state {
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => true,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => false,
            _ => return invalid_argument(b"unknown runtime gamepad button state"),
        };
        self.submit_input_event(InputEvent::GamepadButton {
            gamepad: GamepadId(event.pointer_id),
            button: gamepad_button(event.button),
            value: event.delta,
            pressed,
        });
        ZrStatus::ok()
    }

    fn handle_gamepad_axis(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        self.submit_input_event(InputEvent::GamepadAxis {
            gamepad: GamepadId(event.pointer_id),
            axis: gamepad_axis(event.button),
            value: event.delta,
        });
        ZrStatus::ok()
    }

    fn handle_pressed(&mut self, button: u32) {
        match button {
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => {
                let viewport_size = self.camera_controller.viewport_size();
                let menu_hit = self.level.with_world(|world| {
                    runtime_session_menu_action_at(world, viewport_size, self.cursor).is_some()
                });
                if !menu_hit {
                    self.camera_controller.left_pressed(self.cursor);
                }
            }
            ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => self.camera_controller.right_pressed(self.cursor),
            ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => self.camera_controller.middle_pressed(self.cursor),
            _ => {}
        }
    }

    fn handle_released(&mut self, button: u32) {
        match button {
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => {
                let viewport_size = self.camera_controller.viewport_size();
                if let Some(action) = self.level.with_world(|world| {
                    runtime_session_menu_action_at(world, viewport_size, self.cursor)
                }) {
                    self.level
                        .with_world_mut(|world| write_runtime_menu_action(world, action));
                }
                self.camera_controller.left_released();
            }
            ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => self.camera_controller.right_released(),
            ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => self.camera_controller.middle_released(),
            _ => {}
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        self.level
            .with_world_mut(|world| self.camera_controller.scrolled(world, delta));
    }
}
