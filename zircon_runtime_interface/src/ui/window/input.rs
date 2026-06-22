use serde::{Deserialize, Serialize};

use crate::ui::{
    accessibility::UiAccessibilityActionRequest,
    component::UiDragPayload,
    dispatch::{
        UiAccessibilityInputEvent, UiAnalogInputEvent, UiDeviceId, UiDragDropInputEvent,
        UiDragDropInputEventKind, UiDragSessionId, UiImeDeleteSurrounding, UiImeInputEvent,
        UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiInputModifiers,
        UiKeyboardInputEvent, UiKeyboardInputState, UiMouseMotionInputEvent,
        UiNavigationInputEvent, UiPointerEvent, UiPointerId, UiPointerInputEvent, UiPointerSource,
        UiPopupInputEvent, UiPopupInputEventKind, UiPreciseScrollDelta, UiSurfaceId,
        UiTextByteRange, UiTextInputEvent, UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind,
        UiUserId,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
};

use super::{UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata};

/// Platform event adapters use this context to attach stable window/user/device
/// identity before handing normalized events to the shared UI dispatcher.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowInputContext {
    pub metadata: UiInputEventMetadata,
}

impl UiWindowInputContext {
    pub fn from_window_metadata(metadata: &UiWindowEventMetadata) -> Self {
        let mut input = UiInputEventMetadata::new(metadata.timestamp, metadata.sequence);
        input.window_id = Some(metadata.window_id.clone());
        input.synthetic = metadata.synthetic;
        Self { metadata: input }
    }

    pub fn with_user_id(mut self, user_id: UiUserId) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    pub fn with_device_id(mut self, device_id: UiDeviceId) -> Self {
        self.metadata.device_id = Some(device_id);
        self
    }

    pub fn with_surface_id(mut self, surface_id: UiSurfaceId) -> Self {
        self.metadata.surface_id = Some(surface_id);
        self
    }

    pub fn with_pointer_id(mut self, pointer_id: UiPointerId) -> Self {
        self.metadata.pointer_id = Some(pointer_id);
        self
    }

    pub fn with_pointer_source(mut self, pointer_source: UiPointerSource) -> Self {
        self.metadata.pointer_source = pointer_source;
        self
    }

    pub fn with_modifiers(mut self, modifiers: UiInputModifiers) -> Self {
        self.metadata.modifiers = modifiers;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWindowPlatformInputEvent {
    pub context: UiWindowInputContext,
    pub kind: UiWindowPlatformInputEventKind,
}

impl UiWindowPlatformInputEvent {
    pub const fn new(context: UiWindowInputContext, kind: UiWindowPlatformInputEventKind) -> Self {
        Self { context, kind }
    }

    pub const fn pointer(
        context: UiWindowInputContext,
        event: UiPointerEvent,
        precise_scroll: Option<UiPreciseScrollDelta>,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Pointer {
                event,
                precise_scroll,
            },
        )
    }

    pub const fn mouse_move(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Move, point),
            None,
        )
    }

    pub const fn cursor_entered(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Move, point),
            None,
        )
    }

    pub const fn cursor_left(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Cancel, point),
            None,
        )
    }

    pub const fn mouse_capture_lost(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Cancel, point),
            None,
        )
    }

    pub const fn mouse_wheel(context: UiWindowInputContext, point: UiPoint, delta: f32) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
            Some(UiPreciseScrollDelta::lines(0.0, delta)),
        )
    }

    pub const fn mouse_wheel_delta(
        context: UiWindowInputContext,
        point: UiPoint,
        delta: UiPreciseScrollDelta,
    ) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta.y),
            Some(delta),
        )
    }

    pub const fn raw_mouse_motion(
        context: UiWindowInputContext,
        delta_x: f32,
        delta_y: f32,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::MouseMotion { delta_x, delta_y },
        )
    }

    pub const fn mouse_button_down(
        context: UiWindowInputContext,
        button: UiPointerButton,
        point: UiPoint,
    ) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Down, point).with_button(button),
            None,
        )
    }

    pub const fn mouse_button_up(
        context: UiWindowInputContext,
        button: UiPointerButton,
        point: UiPoint,
    ) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Up, point).with_button(button),
            None,
        )
    }

    pub const fn mouse_double_click(
        context: UiWindowInputContext,
        button: UiPointerButton,
        point: UiPoint,
    ) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(button)
                .with_click_count(2),
            None,
        )
    }

    pub fn keyboard(
        context: UiWindowInputContext,
        state: UiKeyboardInputState,
        key_code: u32,
        scan_code: Option<u32>,
        physical_key: impl Into<String>,
        logical_key: impl Into<String>,
        text: Option<String>,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Keyboard {
                state,
                key_code,
                scan_code,
                physical_key: physical_key.into(),
                logical_key: logical_key.into(),
                text,
            },
        )
    }

    pub fn key_down(
        context: UiWindowInputContext,
        key_code: u32,
        scan_code: Option<u32>,
        physical_key: impl Into<String>,
        logical_key: impl Into<String>,
        is_repeat: bool,
    ) -> Self {
        Self::keyboard(
            context,
            if is_repeat {
                UiKeyboardInputState::Repeated
            } else {
                UiKeyboardInputState::Pressed
            },
            key_code,
            scan_code,
            physical_key,
            logical_key,
            None,
        )
    }

    pub fn key_up(
        context: UiWindowInputContext,
        key_code: u32,
        scan_code: Option<u32>,
        physical_key: impl Into<String>,
        logical_key: impl Into<String>,
    ) -> Self {
        Self::keyboard(
            context,
            UiKeyboardInputState::Released,
            key_code,
            scan_code,
            physical_key,
            logical_key,
            None,
        )
    }

    pub fn key_char(context: UiWindowInputContext, character: char, is_repeat: bool) -> Self {
        let text = character.to_string();
        Self::keyboard(
            context,
            if is_repeat {
                UiKeyboardInputState::Repeated
            } else {
                UiKeyboardInputState::Pressed
            },
            u32::from(character),
            None,
            "Character",
            text.clone(),
            Some(text),
        )
    }

    pub fn controller_button_pressed(
        context: UiWindowInputContext,
        button: impl Into<String>,
        is_repeat: bool,
    ) -> Self {
        let button = button.into();
        Self::keyboard(
            context,
            if is_repeat {
                UiKeyboardInputState::Repeated
            } else {
                UiKeyboardInputState::Pressed
            },
            0,
            None,
            button.clone(),
            button,
            None,
        )
    }

    pub fn controller_button_released(
        context: UiWindowInputContext,
        button: impl Into<String>,
    ) -> Self {
        let button = button.into();
        Self::keyboard(
            context,
            UiKeyboardInputState::Released,
            0,
            None,
            button.clone(),
            button,
            None,
        )
    }

    pub fn text(context: UiWindowInputContext, text: impl Into<String>) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Text { text: text.into() },
        )
    }

    pub fn ime(
        context: UiWindowInputContext,
        kind: UiImeInputEventKind,
        text: impl Into<String>,
    ) -> Self {
        Self::ime_with_cursor_range(context, kind, text, None)
    }

    pub fn ime_with_cursor_range(
        context: UiWindowInputContext,
        kind: UiImeInputEventKind,
        text: impl Into<String>,
        cursor_range: Option<UiTextByteRange>,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Ime {
                kind,
                text: text.into(),
                cursor_range,
                delete_surrounding: None,
            },
        )
    }

    pub fn ime_delete_surrounding(
        context: UiWindowInputContext,
        before_bytes: u32,
        after_bytes: u32,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Ime {
                kind: UiImeInputEventKind::DeleteSurrounding,
                text: String::new(),
                cursor_range: None,
                delete_surrounding: Some(UiImeDeleteSurrounding::new(before_bytes, after_bytes)),
            },
        )
    }

    pub fn navigation(context: UiWindowInputContext, kind: UiNavigationEventKind) -> Self {
        Self::new(context, UiWindowPlatformInputEventKind::Navigation { kind })
    }

    pub fn analog(context: UiWindowInputContext, control: impl Into<String>, value: f32) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Analog {
                control: control.into(),
                value,
            },
        )
    }

    pub fn controller_analog(
        context: UiWindowInputContext,
        control: impl Into<String>,
        value: f32,
    ) -> Self {
        Self::analog(context, control, value)
    }

    pub fn drag_drop(
        context: UiWindowInputContext,
        kind: UiDragDropInputEventKind,
        point: UiPoint,
        session_id: Option<UiDragSessionId>,
        payload: Option<UiDragPayload>,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::DragDrop {
                kind,
                session_id,
                point,
                payload,
            },
        )
    }

    pub fn drag_enter(
        context: UiWindowInputContext,
        point: UiPoint,
        session_id: Option<UiDragSessionId>,
        payload: Option<UiDragPayload>,
    ) -> Self {
        Self::drag_drop(
            context,
            UiDragDropInputEventKind::Enter,
            point,
            session_id,
            payload,
        )
    }

    pub fn drag_over(
        context: UiWindowInputContext,
        point: UiPoint,
        session_id: Option<UiDragSessionId>,
    ) -> Self {
        Self::drag_drop(
            context,
            UiDragDropInputEventKind::Over,
            point,
            session_id,
            None,
        )
    }

    pub fn drag_leave(
        context: UiWindowInputContext,
        point: UiPoint,
        session_id: Option<UiDragSessionId>,
    ) -> Self {
        Self::drag_drop(
            context,
            UiDragDropInputEventKind::Leave,
            point,
            session_id,
            None,
        )
    }

    pub fn drag_drop_at(
        context: UiWindowInputContext,
        point: UiPoint,
        session_id: Option<UiDragSessionId>,
        payload: Option<UiDragPayload>,
    ) -> Self {
        Self::drag_drop(
            context,
            UiDragDropInputEventKind::Drop,
            point,
            session_id,
            payload,
        )
    }

    pub fn drag_end(
        context: UiWindowInputContext,
        point: UiPoint,
        session_id: Option<UiDragSessionId>,
    ) -> Self {
        Self::drag_drop(
            context,
            UiDragDropInputEventKind::End,
            point,
            session_id,
            None,
        )
    }

    pub fn popup(
        context: UiWindowInputContext,
        kind: UiPopupInputEventKind,
        popup_id: impl Into<String>,
        owner: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Popup {
                kind,
                popup_id: popup_id.into(),
                owner,
                anchor,
            },
        )
    }

    pub fn popup_open_requested(
        context: UiWindowInputContext,
        popup_id: impl Into<String>,
        owner: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    ) -> Self {
        Self::popup(
            context,
            UiPopupInputEventKind::OpenRequested,
            popup_id,
            owner,
            anchor,
        )
    }

    pub fn popup_close_requested(
        context: UiWindowInputContext,
        popup_id: impl Into<String>,
        owner: Option<UiNodeId>,
    ) -> Self {
        Self::popup(
            context,
            UiPopupInputEventKind::CloseRequested,
            popup_id,
            owner,
            None,
        )
    }

    pub fn popup_dismissed(context: UiWindowInputContext, popup_id: impl Into<String>) -> Self {
        Self::popup(
            context,
            UiPopupInputEventKind::Dismissed,
            popup_id,
            None,
            None,
        )
    }

    pub fn tooltip_timer(
        context: UiWindowInputContext,
        kind: UiTooltipTimerInputEventKind,
        tooltip_id: impl Into<String>,
        owner: Option<UiNodeId>,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::TooltipTimer {
                kind,
                tooltip_id: tooltip_id.into(),
                owner,
            },
        )
    }

    pub fn tooltip_armed(
        context: UiWindowInputContext,
        tooltip_id: impl Into<String>,
        owner: Option<UiNodeId>,
    ) -> Self {
        Self::tooltip_timer(
            context,
            UiTooltipTimerInputEventKind::Armed,
            tooltip_id,
            owner,
        )
    }

    pub fn tooltip_elapsed(
        context: UiWindowInputContext,
        tooltip_id: impl Into<String>,
        owner: Option<UiNodeId>,
    ) -> Self {
        Self::tooltip_timer(
            context,
            UiTooltipTimerInputEventKind::Elapsed,
            tooltip_id,
            owner,
        )
    }

    pub fn tooltip_canceled(context: UiWindowInputContext, tooltip_id: impl Into<String>) -> Self {
        Self::tooltip_timer(
            context,
            UiTooltipTimerInputEventKind::Canceled,
            tooltip_id,
            None,
        )
    }

    pub const fn accessibility(
        context: UiWindowInputContext,
        request: UiAccessibilityActionRequest,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Accessibility { request },
        )
    }

    pub const fn touch(
        context: UiWindowInputContext,
        phase: UiWindowTouchPhase,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Touch {
                phase,
                pointer_id,
                point,
            },
        )
    }

    pub const fn touch_started(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Started, pointer_id, point)
    }

    pub const fn touch_moved(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Moved, pointer_id, point)
    }

    pub const fn touch_force_changed(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
        _force: f32,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Moved, pointer_id, point)
    }

    pub const fn touch_first_move(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
        _force: f32,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Moved, pointer_id, point)
    }

    pub const fn touch_ended(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Ended, pointer_id, point)
    }

    pub const fn touch_canceled(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Canceled, pointer_id, point)
    }

    pub fn normalize(self) -> UiInputEvent {
        let mut metadata = self.context.metadata;
        match self.kind {
            UiWindowPlatformInputEventKind::Pointer {
                event,
                precise_scroll,
            } => UiInputEvent::Pointer(UiPointerInputEvent {
                metadata,
                event,
                precise_scroll,
            }),
            UiWindowPlatformInputEventKind::Keyboard {
                state,
                key_code,
                scan_code,
                physical_key,
                logical_key,
                text,
            } => UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata,
                state,
                key_code,
                scan_code,
                physical_key,
                logical_key,
                text,
            }),
            UiWindowPlatformInputEventKind::Text { text } => {
                UiInputEvent::Text(UiTextInputEvent { metadata, text })
            }
            UiWindowPlatformInputEventKind::Ime {
                kind,
                text,
                cursor_range,
                delete_surrounding,
            } => UiInputEvent::Ime(UiImeInputEvent {
                metadata,
                kind,
                text,
                cursor_range,
                delete_surrounding,
            }),
            UiWindowPlatformInputEventKind::Navigation { kind } => {
                UiInputEvent::Navigation(UiNavigationInputEvent { metadata, kind })
            }
            UiWindowPlatformInputEventKind::Analog { control, value } => {
                UiInputEvent::Analog(UiAnalogInputEvent {
                    metadata,
                    control,
                    value,
                })
            }
            UiWindowPlatformInputEventKind::MouseMotion { delta_x, delta_y } => {
                UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
                    metadata,
                    delta_x,
                    delta_y,
                })
            }
            UiWindowPlatformInputEventKind::DragDrop {
                kind,
                session_id,
                point,
                payload,
            } => UiInputEvent::DragDrop(UiDragDropInputEvent {
                metadata,
                kind,
                session_id,
                point,
                payload,
            }),
            UiWindowPlatformInputEventKind::Popup {
                kind,
                popup_id,
                owner,
                anchor,
            } => UiInputEvent::Popup(UiPopupInputEvent {
                metadata,
                kind,
                popup_id,
                owner,
                anchor,
            }),
            UiWindowPlatformInputEventKind::TooltipTimer {
                kind,
                tooltip_id,
                owner,
            } => UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
                metadata,
                kind,
                tooltip_id,
                owner,
            }),
            UiWindowPlatformInputEventKind::Accessibility { request } => {
                UiInputEvent::Accessibility(UiAccessibilityInputEvent { metadata, request })
            }
            UiWindowPlatformInputEventKind::Touch {
                phase,
                pointer_id,
                point,
            } => {
                metadata.pointer_id = Some(pointer_id);
                metadata.pointer_source = UiPointerSource::Touch;
                UiInputEvent::Pointer(UiPointerInputEvent {
                    metadata,
                    event: phase.pointer_event(point),
                    precise_scroll: None,
                })
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiWindowPlatformInputEventKind {
    Pointer {
        event: UiPointerEvent,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        precise_scroll: Option<UiPreciseScrollDelta>,
    },
    Keyboard {
        state: UiKeyboardInputState,
        key_code: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scan_code: Option<u32>,
        physical_key: String,
        logical_key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    Text {
        text: String,
    },
    Ime {
        kind: UiImeInputEventKind,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cursor_range: Option<UiTextByteRange>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delete_surrounding: Option<UiImeDeleteSurrounding>,
    },
    Navigation {
        kind: UiNavigationEventKind,
    },
    Analog {
        control: String,
        value: f32,
    },
    MouseMotion {
        delta_x: f32,
        delta_y: f32,
    },
    DragDrop {
        kind: UiDragDropInputEventKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<UiDragSessionId>,
        point: UiPoint,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<UiDragPayload>,
    },
    Popup {
        kind: UiPopupInputEventKind,
        popup_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        owner: Option<UiNodeId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<UiPoint>,
    },
    TooltipTimer {
        kind: UiTooltipTimerInputEventKind,
        tooltip_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        owner: Option<UiNodeId>,
    },
    Accessibility {
        request: UiAccessibilityActionRequest,
    },
    Touch {
        phase: UiWindowTouchPhase,
        pointer_id: UiPointerId,
        point: UiPoint,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowTouchPhase {
    Started,
    Moved,
    Ended,
    Canceled,
}

impl UiWindowTouchPhase {
    const fn pointer_event(self, point: UiPoint) -> UiPointerEvent {
        match self {
            Self::Started => UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
            Self::Moved => UiPointerEvent::new(UiPointerEventKind::Move, point),
            Self::Ended => UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
            Self::Canceled => UiPointerEvent::new(UiPointerEventKind::Cancel, point),
        }
    }
}

impl UiWindowEvent {
    pub fn input_context(&self) -> UiWindowInputContext {
        UiWindowInputContext::from_window_metadata(&self.metadata)
    }

    pub fn normalized_cursor_move_input(&self) -> Option<UiInputEvent> {
        match self.kind {
            UiWindowEventKind::CursorMoved { position, .. } => Some(
                UiWindowPlatformInputEvent::pointer(
                    self.input_context(),
                    UiPointerEvent::new(UiPointerEventKind::Move, position),
                    None,
                )
                .normalize(),
            ),
            _ => None,
        }
    }

    pub fn normalized_pointer_cancel_input(&self, point: UiPoint) -> Option<UiInputEvent> {
        if !matches!(
            self.kind,
            UiWindowEventKind::CursorLeft
                | UiWindowEventKind::Closed
                | UiWindowEventKind::Destroyed
        ) {
            return None;
        }

        Some(
            UiWindowPlatformInputEvent::pointer(
                self.input_context(),
                UiPointerEvent::new(UiPointerEventKind::Cancel, point),
                None,
            )
            .normalize(),
        )
    }
}
