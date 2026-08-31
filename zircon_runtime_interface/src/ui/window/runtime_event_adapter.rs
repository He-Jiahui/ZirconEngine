use thiserror::Error;

use crate::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
    ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1, ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1,
    ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1,
    ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1, ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1,
    ZR_RUNTIME_EVENT_KIND_IME_V1, ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1,
    ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1,
    ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1, ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1,
    ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1, ZR_RUNTIME_EVENT_KIND_TOUCH_V1,
    ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1, ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
    ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1, ZR_RUNTIME_FILE_DRAG_CANCELLED_V1,
    ZR_RUNTIME_FILE_DRAG_DROPPED_V1, ZR_RUNTIME_FILE_DRAG_HOVERED_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_START_V1, ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1,
    ZR_RUNTIME_IME_STATE_COMMIT_V1, ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1,
    ZR_RUNTIME_IME_STATE_DISABLED_V1, ZR_RUNTIME_IME_STATE_PREEDIT_V1,
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
    ZR_RUNTIME_KEY_ACTION_TEXT_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1, ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1, ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1,
    ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1, ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1,
    ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
    ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_MOVED_V1, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
    ZR_RUNTIME_WINDOW_BOOL_FALSE_V1, ZR_RUNTIME_WINDOW_BOOL_TRUE_V1,
    ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1, ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
    ZR_RUNTIME_WINDOW_STATUS_MOVED_V1, ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1,
    ZR_RUNTIME_WINDOW_STATUS_SURFACE_RECREATED_V1, ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1,
};

use crate::ui::{
    accessibility::UiAccessibilityActionRequest,
    component::{UiDragPayload, UiDragPayloadKind},
    dispatch::{
        UiDeviceId, UiDragSessionId, UiImeInputEventKind, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputState, UiPointerId, UiPreciseScrollDelta, UiTextByteRange, UiUserId,
        UiWindowId,
    },
    layout::{UiPoint, UiSize},
    surface::UiPointerButton,
};

use super::{
    UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputContext,
    UiWindowInputPumpBatch, UiWindowInputPumpEvent, UiWindowMetrics, UiWindowPixelPosition,
    UiWindowPixelSize, UiWindowPlatformInputEvent, UiWindowRedrawReason, UiWindowTouchPhase,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiRuntimeEventAdapterContext {
    pub window_id: UiWindowId,
    pub timestamp: UiInputTimestamp,
    pub sequence: UiInputSequence,
    pub user_id: Option<UiUserId>,
    pub device_id: Option<UiDeviceId>,
    pub synthetic: bool,
}

impl UiRuntimeEventAdapterContext {
    pub fn for_window(window_id: impl Into<String>) -> Self {
        Self {
            window_id: UiWindowId::new(window_id),
            timestamp: UiInputTimestamp::default(),
            sequence: UiInputSequence::default(),
            user_id: None,
            device_id: None,
            synthetic: false,
        }
    }

    pub const fn with_timestamp(mut self, timestamp: UiInputTimestamp) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub const fn with_sequence(mut self, sequence: UiInputSequence) -> Self {
        self.sequence = sequence;
        self
    }

    pub const fn with_user_id(mut self, user_id: UiUserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub const fn with_device_id(mut self, device_id: UiDeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub const fn synthetic(mut self, synthetic: bool) -> Self {
        self.synthetic = synthetic;
        self
    }

    fn window_metadata(&self) -> UiWindowEventMetadata {
        UiWindowEventMetadata::for_window(self.window_id.clone(), self.timestamp, self.sequence)
            .synthetic(self.synthetic)
    }

    fn input_context(&self) -> UiWindowInputContext {
        let mut context = UiWindowInputContext::from_window_metadata(&self.window_metadata());
        if let Some(user_id) = self.user_id {
            context = context.with_user_id(user_id);
        }
        if let Some(device_id) = self.device_id {
            context = context.with_device_id(device_id);
        }
        context
    }
}

impl Default for UiRuntimeEventAdapterContext {
    fn default() -> Self {
        Self::for_window("")
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiRuntimeEventAdapterError {
    #[error("unsupported runtime ABI version {actual}; expected {expected}")]
    UnsupportedAbi { actual: u32, expected: u32 },
    #[error("unsupported runtime event kind {0}")]
    UnsupportedKind(u32),
    #[error("runtime event kind {0} has no UI window/input pump equivalent")]
    NoPumpEquivalent(u32),
    #[error("unknown runtime mouse button {0}")]
    UnknownMouseButton(u32),
    #[error("unknown runtime mouse button state {0}")]
    UnknownButtonState(u32),
    #[error("unknown runtime mouse wheel unit {0}")]
    UnknownMouseWheelUnit(u32),
    #[error("unknown runtime touch phase {0}")]
    UnknownTouchPhase(u32),
    #[error("unknown runtime key action {0}")]
    UnknownKeyAction(u32),
    #[error("unknown runtime lifecycle state {0}")]
    UnknownLifecycleState(u32),
    #[error("unknown runtime window status {0}")]
    UnknownWindowStatus(u32),
    #[error("unknown runtime window bool {0}")]
    UnknownWindowBool(u32),
    #[error("unknown runtime file drag/drop state {0}")]
    UnknownFileDragDropState(u32),
    #[error("invalid runtime event text payload")]
    InvalidTextPayload,
    #[error("invalid runtime accessibility action payload")]
    InvalidAccessibilityPayload,
}

pub type UiRuntimeEventAdapterResult<T> = Result<T, UiRuntimeEventAdapterError>;

pub fn runtime_event_to_window_input_pump_event(
    context: &UiRuntimeEventAdapterContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowInputPumpEvent> {
    validate_abi(event)?;
    let pump_event = match event.kind {
        ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1 => UiWindowInputPumpEvent::Window(
            UiWindowEvent::size_changed(context.window_metadata(), viewport_metrics(event)),
        ),
        ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1 => {
            UiWindowInputPumpEvent::Window(UiWindowEvent::new(
                context.window_metadata(),
                UiWindowEventKind::CursorMoved {
                    position: event_point(event),
                    delta: None,
                },
            ))
        }
        ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1 => UiWindowInputPumpEvent::Window(
            UiWindowEvent::new(context.window_metadata(), UiWindowEventKind::CursorEntered),
        ),
        ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1 => UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            context.window_metadata(),
            UiWindowEventKind::CursorLeft,
        )),
        ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1 => {
            input_event(mouse_button_event(context.input_context(), event)?)
        }
        ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1 => {
            input_event(mouse_wheel_event(context.input_context(), event)?)
        }
        ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1 => UiWindowInputPumpEvent::Input(
            UiWindowPlatformInputEvent::raw_mouse_motion(context.input_context(), event.x, event.y)
                .normalize(),
        ),
        ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1 => UiWindowInputPumpEvent::Window(
            lifecycle_window_event(context.window_metadata(), event)?,
        ),
        ZR_RUNTIME_EVENT_KIND_TOUCH_V1 => input_event(touch_event(context.input_context(), event)?),
        ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1 => keyboard_pump_event(context.input_context(), event)?,
        ZR_RUNTIME_EVENT_KIND_IME_V1 => ime_pump_event(context.input_context(), event)?,
        ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1 => {
            input_event(file_drag_drop_event(context.input_context(), event)?)
        }
        ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1 => {
            UiWindowInputPumpEvent::Window(window_status_event(context.window_metadata(), event)?)
        }
        ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1 => {
            input_event(gamepad_button_event(context.input_context(), event)?)
        }
        ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1 => {
            input_event(gamepad_axis_event(context.input_context(), event))
        }
        ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1 => {
            input_event(accessibility_event(context.input_context(), event)?)
        }
        ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1 => {
            return Err(UiRuntimeEventAdapterError::NoPumpEquivalent(event.kind));
        }
        _ => return Err(UiRuntimeEventAdapterError::UnsupportedKind(event.kind)),
    };
    Ok(pump_event)
}

pub fn runtime_events_to_window_input_pump_batch(
    context: &UiRuntimeEventAdapterContext,
    events: impl IntoIterator<Item = ZrRuntimeEventV1>,
) -> UiRuntimeEventAdapterResult<UiWindowInputPumpBatch> {
    let mut batch = UiWindowInputPumpBatch::default();
    for event in events {
        batch.push(runtime_event_to_window_input_pump_event(context, event)?);
    }
    Ok(batch)
}

fn validate_abi(event: ZrRuntimeEventV1) -> UiRuntimeEventAdapterResult<()> {
    if event.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Ok(());
    }
    Err(UiRuntimeEventAdapterError::UnsupportedAbi {
        actual: event.abi_version,
        expected: ZIRCON_RUNTIME_ABI_VERSION_V1,
    })
}

fn input_event(event: UiWindowPlatformInputEvent) -> UiWindowInputPumpEvent {
    UiWindowInputPumpEvent::Input(event.normalize())
}

fn viewport_metrics(event: ZrRuntimeEventV1) -> UiWindowMetrics {
    let metrics = event.metrics;
    if metrics.logical_size.width > 0
        || metrics.logical_size.height > 0
        || metrics.physical_size.width > 0
        || metrics.physical_size.height > 0
    {
        UiWindowMetrics::new(
            UiSize::new(
                metrics.logical_size.width as f32,
                metrics.logical_size.height as f32,
            ),
            UiWindowPixelSize::new(metrics.physical_size.width, metrics.physical_size.height),
            sanitized_scale_factor(metrics.device_scale_factor),
        )
    } else {
        UiWindowMetrics::new(
            UiSize::new(event.size.width as f32, event.size.height as f32),
            UiWindowPixelSize::new(event.size.width, event.size.height),
            1.0,
        )
    }
}

fn sanitized_scale_factor(scale_factor: f32) -> f64 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        f64::from(scale_factor)
    } else {
        1.0
    }
}

fn event_point(event: ZrRuntimeEventV1) -> UiPoint {
    UiPoint::new(event.x, event.y)
}

fn mouse_button_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowPlatformInputEvent> {
    let button = pointer_button(event.button)?;
    match event.state {
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => Ok(UiWindowPlatformInputEvent::mouse_button_down(
            context,
            button,
            event_point(event),
        )),
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => Ok(UiWindowPlatformInputEvent::mouse_button_up(
            context,
            button,
            event_point(event),
        )),
        state => Err(UiRuntimeEventAdapterError::UnknownButtonState(state)),
    }
}

fn mouse_wheel_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowPlatformInputEvent> {
    let point = if event.button == ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1 {
        event_point(event)
    } else {
        UiPoint::default()
    };
    let delta_x = if event.button == ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1 {
        f32::from_bits(event.key_code)
    } else {
        event.x
    };
    let delta_y = if event.button == ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1 {
        f32::from_bits(event.scan_code)
    } else {
        event.y
    };
    match event.state {
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1 => Ok(UiWindowPlatformInputEvent::mouse_wheel_delta(
            context,
            point,
            UiPreciseScrollDelta::lines(delta_x, delta_y),
        )),
        ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1 => Ok(UiWindowPlatformInputEvent::mouse_wheel_delta(
            context,
            point,
            UiPreciseScrollDelta::pixels(delta_x, delta_y),
        )),
        unit => Err(UiRuntimeEventAdapterError::UnknownMouseWheelUnit(unit)),
    }
}

fn lifecycle_window_event(
    metadata: UiWindowEventMetadata,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowEvent> {
    match event.state {
        ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1 | ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1 => Ok(
            UiWindowEvent::application_activation_changed(metadata, true),
        ),
        ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1 | ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1 => Ok(
            UiWindowEvent::application_activation_changed(metadata, false),
        ),
        ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1 => {
            Ok(UiWindowEvent::window_focused(metadata, false))
        }
        state => Err(UiRuntimeEventAdapterError::UnknownLifecycleState(state)),
    }
}

fn touch_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowPlatformInputEvent> {
    let phase = match event.state {
        ZR_RUNTIME_TOUCH_PHASE_STARTED_V1 => UiWindowTouchPhase::Started,
        ZR_RUNTIME_TOUCH_PHASE_MOVED_V1 => UiWindowTouchPhase::Moved,
        ZR_RUNTIME_TOUCH_PHASE_ENDED_V1 => UiWindowTouchPhase::Ended,
        ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1 => UiWindowTouchPhase::Canceled,
        phase => return Err(UiRuntimeEventAdapterError::UnknownTouchPhase(phase)),
    };
    Ok(UiWindowPlatformInputEvent::touch(
        context,
        phase,
        UiPointerId::new(event.pointer_id),
        event_point(event),
    ))
}

fn keyboard_pump_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowInputPumpEvent> {
    let text = optional_payload_text(event)?;
    match event.button {
        ZR_RUNTIME_KEY_ACTION_PRESSED_V1 => Ok(input_event(UiWindowPlatformInputEvent::keyboard(
            context,
            UiKeyboardInputState::Pressed,
            event.key_code,
            scan_code(event),
            physical_key_name(event.key_code),
            logical_key_name(event.key_code),
            text,
        ))),
        ZR_RUNTIME_KEY_ACTION_RELEASED_V1 => Ok(input_event(UiWindowPlatformInputEvent::keyboard(
            context,
            UiKeyboardInputState::Released,
            event.key_code,
            scan_code(event),
            physical_key_name(event.key_code),
            logical_key_name(event.key_code),
            text,
        ))),
        ZR_RUNTIME_KEY_ACTION_TEXT_V1 => {
            let text = text.unwrap_or_default();
            Ok(UiWindowInputPumpEvent::Input(
                UiWindowPlatformInputEvent::text(context, text).normalize(),
            ))
        }
        action => Err(UiRuntimeEventAdapterError::UnknownKeyAction(action)),
    }
}

fn ime_pump_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowInputPumpEvent> {
    match event.state {
        ZR_RUNTIME_IME_STATE_PREEDIT_V1 => Ok(UiWindowInputPumpEvent::Input(
            UiWindowPlatformInputEvent::ime_with_cursor_range(
                context,
                UiImeInputEventKind::Preedit,
                payload_text(event)?,
                Some(UiTextByteRange::new(event.key_code, event.scan_code)),
            )
            .normalize(),
        )),
        ZR_RUNTIME_IME_STATE_COMMIT_V1 => Ok(UiWindowInputPumpEvent::Input(
            UiWindowPlatformInputEvent::ime(
                context,
                UiImeInputEventKind::Commit,
                payload_text(event)?,
            )
            .normalize(),
        )),
        ZR_RUNTIME_IME_STATE_DISABLED_V1 => Ok(UiWindowInputPumpEvent::Input(
            UiWindowPlatformInputEvent::ime(context, UiImeInputEventKind::Cancel, "").normalize(),
        )),
        ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1 => Ok(UiWindowInputPumpEvent::Input(
            UiWindowPlatformInputEvent::ime_delete_surrounding(
                context,
                event.key_code,
                event.scan_code,
            )
            .normalize(),
        )),
        _ => Err(UiRuntimeEventAdapterError::NoPumpEquivalent(event.kind)),
    }
}

fn file_drag_drop_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowPlatformInputEvent> {
    let point = event_point(event);
    match event.state {
        ZR_RUNTIME_FILE_DRAG_HOVERED_V1 => Ok(UiWindowPlatformInputEvent::drag_over(
            context,
            point,
            Some(UiDragSessionId::new(event.pointer_id)),
        )),
        ZR_RUNTIME_FILE_DRAG_DROPPED_V1 => Ok(UiWindowPlatformInputEvent::drag_drop_at(
            context,
            point,
            Some(UiDragSessionId::new(event.pointer_id)),
            Some(UiDragPayload::new(
                UiDragPayloadKind::Asset,
                payload_text(event)?,
            )),
        )),
        ZR_RUNTIME_FILE_DRAG_CANCELLED_V1 => Ok(UiWindowPlatformInputEvent::drag_end(
            context,
            point,
            Some(UiDragSessionId::new(event.pointer_id)),
        )),
        state => Err(UiRuntimeEventAdapterError::UnknownFileDragDropState(state)),
    }
}

fn window_status_event(
    metadata: UiWindowEventMetadata,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowEvent> {
    match event.state {
        ZR_RUNTIME_WINDOW_STATUS_MOVED_V1 => Ok(UiWindowEvent::moved_window(
            metadata,
            UiWindowPixelPosition::new(event.x as i32, event.y as i32),
        )),
        ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1 => Ok(UiWindowEvent::new(
            metadata,
            UiWindowEventKind::Occluded {
                occluded: runtime_bool(event.button)?,
            },
        )),
        ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1 => Ok(UiWindowEvent::window_close(metadata)),
        ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1 => {
            Ok(UiWindowEvent::new(metadata, UiWindowEventKind::Destroyed))
        }
        ZR_RUNTIME_WINDOW_STATUS_SURFACE_RECREATED_V1 => Ok(UiWindowEvent::request_redraw(
            metadata,
            UiWindowRedrawReason::Host,
        )),
        ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1 => Ok(UiWindowEvent::new(
            metadata,
            UiWindowEventKind::ScaleFactorChanged {
                scale_factor: f64::from(event.delta),
            },
        )),
        ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1 => Ok(UiWindowEvent::new(
            metadata,
            UiWindowEventKind::BackendScaleFactorChanged {
                scale_factor: f64::from(event.delta),
            },
        )),
        ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1 => {
            Err(UiRuntimeEventAdapterError::NoPumpEquivalent(event.kind))
        }
        state => Err(UiRuntimeEventAdapterError::UnknownWindowStatus(state)),
    }
}

fn gamepad_button_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowPlatformInputEvent> {
    let button = gamepad_button_name(event.button);
    match event.state {
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => Ok(
            UiWindowPlatformInputEvent::controller_button_pressed(context, button, false),
        ),
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => Ok(
            UiWindowPlatformInputEvent::controller_button_released(context, button),
        ),
        state => Err(UiRuntimeEventAdapterError::UnknownButtonState(state)),
    }
}

fn gamepad_axis_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiWindowPlatformInputEvent {
    UiWindowPlatformInputEvent::controller_analog(
        context,
        gamepad_axis_name(event.button),
        event.delta,
    )
}

fn accessibility_event(
    context: UiWindowInputContext,
    event: ZrRuntimeEventV1,
) -> UiRuntimeEventAdapterResult<UiWindowPlatformInputEvent> {
    let payload = payload_bytes(event)
        .map_err(|_| UiRuntimeEventAdapterError::InvalidAccessibilityPayload)?;
    let request = serde_json::from_slice::<UiAccessibilityActionRequest>(&payload)
        .map_err(|_| UiRuntimeEventAdapterError::InvalidAccessibilityPayload)?;
    Ok(UiWindowPlatformInputEvent::accessibility(context, request))
}

fn pointer_button(button: u32) -> UiRuntimeEventAdapterResult<UiPointerButton> {
    match button {
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 => Ok(UiPointerButton::Primary),
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 => Ok(UiPointerButton::Secondary),
        ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1 => Ok(UiPointerButton::Middle),
        button => Err(UiRuntimeEventAdapterError::UnknownMouseButton(button)),
    }
}

fn runtime_bool(value: u32) -> UiRuntimeEventAdapterResult<bool> {
    match value {
        ZR_RUNTIME_WINDOW_BOOL_FALSE_V1 => Ok(false),
        ZR_RUNTIME_WINDOW_BOOL_TRUE_V1 => Ok(true),
        value => Err(UiRuntimeEventAdapterError::UnknownWindowBool(value)),
    }
}

fn optional_payload_text(event: ZrRuntimeEventV1) -> UiRuntimeEventAdapterResult<Option<String>> {
    let payload = payload_bytes(event)?;
    if payload.is_empty() {
        return Ok(None);
    }
    String::from_utf8(payload)
        .map(Some)
        .map_err(|_| UiRuntimeEventAdapterError::InvalidTextPayload)
}

fn payload_text(event: ZrRuntimeEventV1) -> UiRuntimeEventAdapterResult<String> {
    String::from_utf8(payload_bytes(event)?)
        .map_err(|_| UiRuntimeEventAdapterError::InvalidTextPayload)
}

fn payload_bytes(event: ZrRuntimeEventV1) -> UiRuntimeEventAdapterResult<Vec<u8>> {
    unsafe {
        event
            .payload
            .checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)
    }
    .map(<[u8]>::to_vec)
    .map_err(|_| UiRuntimeEventAdapterError::InvalidTextPayload)
}

fn scan_code(event: ZrRuntimeEventV1) -> Option<u32> {
    (event.scan_code != 0).then_some(event.scan_code)
}

fn physical_key_name(key_code: u32) -> String {
    if let Some(name) = named_keyboard_key(key_code) {
        name.to_string()
    } else {
        format!("KeyCode{key_code}")
    }
}

fn logical_key_name(key_code: u32) -> String {
    named_keyboard_key(key_code)
        .map(str::to_string)
        .unwrap_or_else(|| key_code.to_string())
}

fn named_keyboard_key(key_code: u32) -> Option<&'static str> {
    match key_code {
        13 => Some("Enter"),
        16 => Some("Shift"),
        17 => Some("Control"),
        18 => Some("Alt"),
        27 => Some("Escape"),
        32 => Some("Space"),
        48 => Some("0"),
        49 => Some("1"),
        50 => Some("2"),
        51 => Some("3"),
        52 => Some("4"),
        53 => Some("5"),
        54 => Some("6"),
        55 => Some("7"),
        56 => Some("8"),
        57 => Some("9"),
        65 => Some("A"),
        66 => Some("B"),
        67 => Some("C"),
        68 => Some("D"),
        69 => Some("E"),
        70 => Some("F"),
        71 => Some("G"),
        72 => Some("H"),
        73 => Some("I"),
        74 => Some("J"),
        75 => Some("K"),
        76 => Some("L"),
        77 => Some("M"),
        78 => Some("N"),
        79 => Some("O"),
        80 => Some("P"),
        81 => Some("Q"),
        82 => Some("R"),
        83 => Some("S"),
        84 => Some("T"),
        85 => Some("U"),
        86 => Some("V"),
        87 => Some("W"),
        88 => Some("X"),
        89 => Some("Y"),
        90 => Some("Z"),
        _ => None,
    }
}

fn gamepad_button_name(button: u32) -> String {
    match button {
        ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1 => "Virtual_Accept",
        ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1 => "Virtual_Back",
        ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1 => "Gamepad_North",
        ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1 => "Gamepad_West",
        ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1 => "Gamepad_Select",
        ZR_RUNTIME_GAMEPAD_BUTTON_START_V1 => "Gamepad_Start",
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1 => "Gamepad_DPad_Up",
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1 => "Gamepad_DPad_Down",
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1 => "Gamepad_DPad_Left",
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1 => "Gamepad_DPad_Right",
        _ => "Gamepad_Button",
    }
    .to_string()
}

fn gamepad_axis_name(axis: u32) -> String {
    match axis {
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1 => "Gamepad_LeftX",
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1 => "Gamepad_LeftY",
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1 => "Gamepad_LeftZ",
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1 => "Gamepad_RightX",
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1 => "Gamepad_RightY",
        ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1 => "Gamepad_RightZ",
        ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1 => "Gamepad_DPadX",
        ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1 => "Gamepad_DPadY",
        _ => "Gamepad_Axis",
    }
    .to_string()
}
