use zircon_runtime_interface::ui::dispatch::{
    UiInputEvent, UiInputEventMetadata, UiMouseMotionInputEvent, UiPointerSource,
};
use zircon_runtime_interface::ui::layout::UiPoint;
use zircon_runtime_interface::ui::surface::{UiPointerButton, UiPointerEventKind};
use zircon_runtime_interface::{
    ZR_RUNTIME_ACCESSIBILITY_ACTION_REQUEST_LIMIT_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1,
    ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1, ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
    ZR_RUNTIME_EVENT_KIND_CLIPBOARD_RESULT_V1, ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1,
    ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1, ZR_RUNTIME_EVENT_KIND_EDITOR_TRANSFORM_WRITE_V1,
    ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1,
    ZR_RUNTIME_EVENT_KIND_IME_V1, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
    ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1,
    ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1, ZR_RUNTIME_EVENT_KIND_TOUCH_V1,
    ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1, ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1,
    ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1, ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
    ZR_RUNTIME_FILE_DRAG_CANCELLED_V1, ZR_RUNTIME_FILE_DRAG_DROPPED_V1,
    ZR_RUNTIME_FILE_DRAG_HOVERED_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
    ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1, ZR_RUNTIME_VIEWPORT_CAMERA_REQUEST_LIMIT_V1,
    ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1, ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
    ZR_RUNTIME_WINDOW_STATUS_MOVED_V1, ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SURFACE_RECREATED_V1, ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
    ZrRuntimeClipboardResultV1, ZrRuntimeEditorTransformWriteV1, ZrRuntimeEventV1,
    ZrRuntimeViewportCameraV1, ZrStatus, ui::accessibility::UiAccessibilityActionRequest,
};

use crate::core::framework::input::{
    FileDragDropEvent, InputEvent, MouseWheelEvent, WindowStatusEvent,
};
use crate::core::math::Vec2;
use crate::core::{ClockDiscontinuity, ClockLifecycleTransition};

use super::super::bounded_json;

use super::input_events::{
    input_button, mouse_scroll_unit, touch_phase, window_bool, window_scale_factor, window_theme,
};
use super::menu::{runtime_session_menu_action_at, write_runtime_menu_action};
use super::status::{
    error_status, invalid_argument, invalid_or_limit_payload, limit_exceeded, not_found,
};
use super::{DEFAULT_VIEWPORT, RuntimeDynamicSession};

mod gamepad;
mod keyboard_ime;

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
                self.record_viewport_resize();
                ZrStatus::ok()
            }
            ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1 => self.handle_viewport_camera(event),
            ZR_RUNTIME_EVENT_KIND_EDITOR_TRANSFORM_WRITE_V1 => {
                self.handle_editor_transform_write(event)
            }
            ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1 => {
                let cursor = Vec2::new(event.x, event.y);
                self.cursor = cursor;
                if self.submit_input_event(InputEvent::CursorMoved {
                    x: cursor.x,
                    y: cursor.y,
                }) {
                    self.record_submitted_pointer_move();
                }
                match self.dispatch_runtime_ui_pointer(
                    UiPointerEventKind::Move,
                    None,
                    None,
                    UiPointerSource::Mouse,
                    0.0,
                ) {
                    Ok(true) => return ZrStatus::ok(),
                    Ok(false) => {}
                    Err(status) => return status,
                }
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
                match self.dispatch_runtime_ui_event(|metadata| {
                    UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
                        metadata,
                        delta_x: event.x,
                        delta_y: event.y,
                    })
                }) {
                    Ok(true) => return ZrStatus::ok(),
                    Ok(false) => {}
                    Err(status) => return status,
                }
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
            ZR_RUNTIME_EVENT_KIND_CLIPBOARD_RESULT_V1 => self.handle_clipboard_result(event),
            _ => invalid_argument(b"unknown runtime event kind"),
        }
    }

    fn handle_viewport_camera(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let camera = match unsafe {
            bounded_json::decode::<ZrRuntimeViewportCameraV1>(
                event.payload,
                ZR_RUNTIME_VIEWPORT_CAMERA_REQUEST_LIMIT_V1,
                |_| 1,
            )
        } {
            Ok(camera) => camera,
            Err(error) => {
                return invalid_or_limit_payload(
                    &error,
                    b"invalid runtime viewport camera payload",
                    b"runtime viewport camera payload exceeds limit",
                );
            }
        };
        match self.camera_controller.apply_editor_camera(camera) {
            Ok(_) => ZrStatus::ok(),
            Err(message) => invalid_argument(message.as_bytes()),
        }
    }

    fn handle_editor_transform_write(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let request = match unsafe { ZrRuntimeEditorTransformWriteV1::from_payload(event.payload) }
        {
            Ok(request) => request,
            Err(_) => return invalid_argument(b"invalid runtime editor transform payload"),
        };
        let editor_transform = &mut self.editor_transform;
        match self
            .level
            .with_world_mut_and_replacement_epoch(|world, replacement_epoch| {
                editor_transform.handle(world, replacement_epoch, request)
            }) {
            Ok(()) => ZrStatus::ok(),
            Err(super::editor_transform::RuntimeEditorTransformWriteError::TargetMissing {
                ..
            }) => not_found(b"runtime editor transform target not found"),
            Err(error) => error_status(error),
        }
    }

    fn handle_accessibility_action(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let request = match unsafe {
            bounded_json::decode::<UiAccessibilityActionRequest>(
                event.payload,
                ZR_RUNTIME_ACCESSIBILITY_ACTION_REQUEST_LIMIT_V1,
                |_| 1,
            )
        } {
            Ok(request) => request,
            Err(error) => {
                return invalid_or_limit_payload(
                    &error,
                    b"invalid accessibility action payload",
                    b"accessibility action payload exceeds limit",
                );
            }
        };
        if self.runtime_ui.is_empty() {
            return not_found(
                b"runtime UI surface accessibility action dispatch unavailable in dynamic preview",
            );
        }
        match self.runtime_ui.dispatch_accessibility_action(request) {
            Ok(true) => ZrStatus::ok(),
            Ok(false) => not_found(b"runtime UI accessibility action target not found"),
            Err(error) => error_status(format!(
                "dispatch declared runtime UI accessibility action: {error}"
            )),
        }
    }

    fn handle_clipboard_result(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let result = match unsafe {
            bounded_json::decode::<ZrRuntimeClipboardResultV1>(
                event.payload,
                ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1,
                |_| 1,
            )
        } {
            Ok(result) => result,
            Err(error) => {
                return invalid_or_limit_payload(
                    &error,
                    b"invalid runtime clipboard result payload",
                    b"runtime clipboard result payload exceeds limit",
                );
            }
        };
        if !result.transfer_id.is_valid() {
            return invalid_argument(b"invalid runtime clipboard result");
        }
        let outcome = match result.outcome {
            zircon_runtime_interface::ui::dispatch::UiClipboardTransferOutcome::ReadText { text }
                if text.len() > ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1 =>
            {
                zircon_runtime_interface::ui::dispatch::UiClipboardTransferOutcome::Failed {
                    reason: zircon_runtime_interface::ui::dispatch::UiClipboardTransferFailure::PayloadTooLarge,
                }
            }
            outcome => outcome,
        };
        match self.runtime_ui.dispatch_clipboard_result(
            result.target_surface,
            result.transfer_id,
            result.owner,
            outcome,
        ) {
            Ok(true) => ZrStatus::ok(),
            Ok(false) => not_found(b"runtime clipboard target surface not found"),
            Err(error) => error_status(format!("dispatch runtime clipboard result: {error}")),
        }
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
                if self.submit_input_event(InputEvent::ButtonPressed(button)) {
                    self.record_submitted_mouse_button_press();
                }
                match self.dispatch_runtime_ui_pointer(
                    UiPointerEventKind::Down,
                    ui_pointer_button(event.button),
                    None,
                    UiPointerSource::Mouse,
                    0.0,
                ) {
                    Ok(true) => return ZrStatus::ok(),
                    Ok(false) => {}
                    Err(status) => return status,
                }
                self.handle_pressed(event.button);
            }
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => {
                if self.submit_input_event(InputEvent::ButtonReleased(button)) {
                    self.record_submitted_mouse_button_release();
                }
                match self.dispatch_runtime_ui_pointer(
                    UiPointerEventKind::Up,
                    ui_pointer_button(event.button),
                    None,
                    UiPointerSource::Mouse,
                    0.0,
                ) {
                    Ok(true) => return ZrStatus::ok(),
                    Ok(false) => {}
                    Err(status) => return status,
                }
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
            match self.dispatch_runtime_ui_pointer(
                UiPointerEventKind::Scroll,
                None,
                None,
                UiPointerSource::Mouse,
                event.delta,
            ) {
                Ok(true) => return ZrStatus::ok(),
                Ok(false) => {}
                Err(status) => return status,
            }
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
        match self.dispatch_runtime_ui_pointer(
            UiPointerEventKind::Scroll,
            None,
            None,
            UiPointerSource::Mouse,
            delta_y,
        ) {
            Ok(true) => return ZrStatus::ok(),
            Ok(false) => {}
            Err(status) => return status,
        }
        self.handle_scroll(wheel.vertical_line_delta());
        ZrStatus::ok()
    }

    fn handle_lifecycle(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        if let Some(discontinuity) = clock_discontinuity_for_lifecycle_state(event.state) {
            self.runtime.submit_clock_discontinuity(discontinuity);
        }
        match event.state {
            ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1 | ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1 => {
                self.submit_input_event(InputEvent::FocusLost);
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
        let (kind, button) = match event.state {
            ZR_RUNTIME_TOUCH_PHASE_STARTED_V1 => {
                (UiPointerEventKind::Down, Some(UiPointerButton::Primary))
            }
            ZR_RUNTIME_TOUCH_PHASE_MOVED_V1 => (UiPointerEventKind::Move, None),
            ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 | ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1 => {
                let kind = if event.state == ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 {
                    UiPointerEventKind::Up
                } else {
                    UiPointerEventKind::Cancel
                };
                (kind, Some(UiPointerButton::Primary))
            }
            _ => unreachable!("touch phase was validated before dispatch"),
        };
        self.cursor = cursor;
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
        match self.dispatch_runtime_ui_pointer(
            kind,
            button,
            Some(event.pointer_id),
            UiPointerSource::Touch,
            0.0,
        ) {
            Ok(true) => return ZrStatus::ok(),
            Ok(false) => {}
            Err(status) => return status,
        }
        match event.state {
            ZR_RUNTIME_TOUCH_PHASE_STARTED_V1 => {
                self.handle_cursor_moved(cursor);
                self.handle_pressed(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1);
            }
            ZR_RUNTIME_TOUCH_PHASE_MOVED_V1 => self.handle_cursor_moved(cursor),
            ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 | ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1 => {
                self.handle_released(ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1);
            }
            _ => unreachable!("touch phase was validated before dispatch"),
        }
        ZrStatus::ok()
    }

    fn handle_file_drag_drop(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let payload = match event_payload(event) {
            Ok(payload) => payload,
            Err(status) => return status,
        };
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
            ZR_RUNTIME_WINDOW_STATUS_SURFACE_RECREATED_V1 => WindowStatusEvent::SurfaceRecreated,
            ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1 => WindowStatusEvent::CloseRequested,
            ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1 => WindowStatusEvent::Destroyed,
            _ => return invalid_argument(b"unknown runtime window status"),
        };
        if let Some(discontinuity) = clock_discontinuity_for_window_status(&window_event) {
            self.runtime.submit_clock_discontinuity(discontinuity);
        }
        self.submit_input_event(InputEvent::WindowStatus(window_event));
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

    fn dispatch_runtime_ui_pointer(
        &mut self,
        kind: UiPointerEventKind,
        button: Option<UiPointerButton>,
        pointer_id: Option<u64>,
        pointer_source: UiPointerSource,
        scroll_delta: f32,
    ) -> Result<bool, ZrStatus> {
        if self.runtime_ui.is_empty() {
            return Ok(false);
        }
        self.runtime_ui
            .dispatch_pointer(
                self.camera_controller.viewport_size(),
                kind,
                UiPoint::new(self.cursor.x, self.cursor.y),
                button,
                pointer_id,
                pointer_source,
                scroll_delta,
            )
            .map_err(|error| {
                error_status(format!(
                    "dispatch declared runtime UI pointer input: {error}"
                ))
            })
    }

    fn dispatch_runtime_ui_event(
        &mut self,
        event: impl FnOnce(UiInputEventMetadata) -> UiInputEvent,
    ) -> Result<bool, ZrStatus> {
        if self.runtime_ui.is_empty() {
            return Ok(false);
        }
        let event = event(self.runtime_ui.next_input_metadata());
        self.runtime_ui
            .dispatch_input(self.camera_controller.viewport_size(), event)
            .map_err(|error| error_status(format!("dispatch declared runtime UI input: {error}")))
    }
}

fn clock_discontinuity_for_lifecycle_state(state: u32) -> Option<ClockDiscontinuity> {
    let transition = match state {
        ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1 => ClockLifecycleTransition::Foregrounded,
        ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1 => ClockLifecycleTransition::Backgrounded,
        ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1 => ClockLifecycleTransition::Suspended,
        ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1 => ClockLifecycleTransition::Resumed,
        _ => return None,
    };
    Some(ClockDiscontinuity::ApplicationLifecycle(transition))
}

fn clock_discontinuity_for_window_status(event: &WindowStatusEvent) -> Option<ClockDiscontinuity> {
    match event {
        WindowStatusEvent::Occluded(occluded) => Some(ClockDiscontinuity::WindowOcclusionChanged {
            occluded: *occluded,
        }),
        WindowStatusEvent::SurfaceRecreated => Some(ClockDiscontinuity::WindowSurfaceRecreated),
        _ => None,
    }
}

fn event_payload(event: ZrRuntimeEventV1) -> Result<&'static [u8], ZrStatus> {
    match unsafe {
        event
            .payload
            .checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)
    } {
        Ok(payload) => Ok(payload),
        Err(error) if error.is_limit_exceeded() => {
            Err(limit_exceeded(b"runtime event payload exceeds limit"))
        }
        Err(_) => Err(invalid_argument(b"invalid runtime event payload slice")),
    }
}

fn ui_pointer_button(button: u32) -> Option<UiPointerButton> {
    match button {
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => Some(UiPointerButton::Primary),
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => Some(UiPointerButton::Secondary),
        ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => Some(UiPointerButton::Middle),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::profile::RuntimeDynamicSessionProfile;
    use super::gamepad::{ui_gamepad_analog_control, ui_gamepad_navigation};
    use super::{
        RuntimeDynamicSession, clock_discontinuity_for_lifecycle_state,
        clock_discontinuity_for_window_status,
    };
    use crate::core::framework::input::WindowStatusEvent;
    use crate::core::{
        ClockDiscontinuity, ClockLifecycleTransition, FrameClockRebaseCause, FrameTimeDiscontinuity,
    };
    use zircon_runtime_interface::ui::surface::UiNavigationEventKind;
    use zircon_runtime_interface::{
        ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1, ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1,
        ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1, ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
        ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
        ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1, ZrByteSlice, ZrRuntimeEventV1,
        ZrRuntimeViewportCameraV1, ZrRuntimeViewportHandle, ZrStatusCode,
    };

    #[test]
    fn gamepad_buttons_map_to_shared_ui_navigation_semantics() {
        assert_eq!(
            ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1),
            Some(UiNavigationEventKind::Activate)
        );
        assert_eq!(
            ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1),
            Some(UiNavigationEventKind::Cancel)
        );
        assert_eq!(
            ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1),
            Some(UiNavigationEventKind::Up)
        );
        assert_eq!(
            ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1),
            Some(UiNavigationEventKind::Down)
        );
        assert_eq!(
            ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1),
            Some(UiNavigationEventKind::Left)
        );
        assert_eq!(
            ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1),
            Some(UiNavigationEventKind::Right)
        );
        assert_eq!(ui_gamepad_navigation(u32::MAX), None);
    }

    #[test]
    fn gamepad_left_stick_axes_use_the_shared_ui_analog_navigation_controls() {
        assert_eq!(
            ui_gamepad_analog_control(ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1),
            Some("gamepad_left_stick_x")
        );
        assert_eq!(
            ui_gamepad_analog_control(ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1),
            Some("gamepad_left_stick_y")
        );
        assert_eq!(ui_gamepad_analog_control(u32::MAX), None);
    }

    #[test]
    fn lifecycle_clock_mapping_keeps_low_memory_out_of_the_time_authority() {
        assert_eq!(
            clock_discontinuity_for_lifecycle_state(ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1),
            Some(ClockDiscontinuity::ApplicationLifecycle(
                ClockLifecycleTransition::Suspended,
            ))
        );
        assert_eq!(
            clock_discontinuity_for_lifecycle_state(ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1),
            None
        );
    }

    #[test]
    fn window_clock_mapping_marks_occlusion_and_surface_recreation() {
        assert_eq!(
            clock_discontinuity_for_window_status(&WindowStatusEvent::Occluded(true)),
            Some(ClockDiscontinuity::WindowOcclusionChanged { occluded: true })
        );
        assert_eq!(
            clock_discontinuity_for_window_status(&WindowStatusEvent::SurfaceRecreated),
            Some(ClockDiscontinuity::WindowSurfaceRecreated)
        );
    }

    #[test]
    fn dynamic_lifecycle_event_replaces_activation_rebase_with_a_typed_clock_cause() {
        let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
            .expect("headless runtime session should construct");

        let status = session.handle_event(ZrRuntimeEventV1::lifecycle(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
        ));
        let snapshot = session
            .runtime
            .tick_time(session.time_policy.max_fixed_steps_per_frame());

        assert_eq!(status.status_code(), ZrStatusCode::Ok);
        assert!(matches!(
            snapshot.discontinuity(),
            Some(FrameTimeDiscontinuity::FrameClockRebased(receipt))
                if receipt.cause()
                    == FrameClockRebaseCause::ClockDiscontinuity(
                        ClockDiscontinuity::ApplicationLifecycle(
                            ClockLifecycleTransition::Suspended,
                        ),
                    )
        ));
    }

    #[test]
    fn simulate_camera_event_overrides_render_extract_without_mutating_the_play_world() {
        let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
            .expect("headless runtime session should construct");
        let active_camera = session.level.with_world(|world| world.active_camera());
        let world_transform_before = session
            .level
            .with_world(|world| world.world_transform(active_camera).unwrap());
        let camera = ZrRuntimeViewportCameraV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            crate::core::math::Transform::from_translation(crate::core::math::Vec3::new(
                7.0, 8.0, 9.0,
            )),
            ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
            60.0_f32.to_radians(),
            18.0,
            0.5,
            750.0,
        );
        let payload = serde_json::to_vec(&camera).expect("camera DTO should encode");

        let status = session.handle_event(ZrRuntimeEventV1::viewport_camera(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            ZrByteSlice {
                data: payload.as_ptr(),
                len: payload.len(),
            },
        ));
        let extract = session.current_extract();
        let world_transform_after = session
            .level
            .with_world(|world| world.world_transform(active_camera).unwrap());

        assert_eq!(status.status_code(), ZrStatusCode::Ok);
        assert_eq!(extract.view.camera.transform, camera.transform);
        assert_eq!(extract.view.camera.ortho_size, 18.0);
        assert_eq!(world_transform_after, world_transform_before);
    }
}
