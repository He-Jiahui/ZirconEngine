use std::sync::Arc;

use crate::ui::dispatch::{
    UiAccessibilityInputEvent, UiAnalogInputEvent, UiDragDropInputEvent, UiImeInputEvent,
    UiInputEvent, UiKeyboardInputEvent, UiMouseMotionInputEvent, UiNavigationInputEvent,
    UiPointerInputEvent, UiPointerSource, UiPopupInputEvent, UiTextInputEvent,
    UiTooltipTimerInputEvent,
};

use super::{UiWindowPlatformInputEvent, UiWindowPlatformInputEventKind};

impl UiWindowPlatformInputEvent {
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
                preedit_clauses,
                delete_surrounding,
            } => UiInputEvent::Ime(UiImeInputEvent {
                metadata,
                kind,
                text,
                cursor_range,
                preedit_clauses,
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
                payload: payload.map(Arc::new),
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
