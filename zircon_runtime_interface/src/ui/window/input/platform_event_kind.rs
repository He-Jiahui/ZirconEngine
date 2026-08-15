use serde::{Deserialize, Serialize};

use crate::ui::{
    accessibility::UiAccessibilityActionRequest,
    component::UiDragPayload,
    dispatch::{
        UiDragDropInputEventKind, UiDragSessionId, UiImeDeleteSurrounding, UiImeInputEventKind,
        UiImePreeditClause, UiKeyboardInputState, UiPointerEvent, UiPointerId,
        UiPopupInputEventKind, UiPreciseScrollDelta, UiTextByteRange, UiTooltipTimerInputEventKind,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::UiNavigationEventKind,
};

use super::UiWindowTouchPhase;

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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        preedit_clauses: Vec<UiImePreeditClause>,
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
