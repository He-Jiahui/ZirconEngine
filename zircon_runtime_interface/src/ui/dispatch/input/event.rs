use serde::{Deserialize, Serialize};

use crate::ui::component::UiDragPayload;
use crate::ui::dispatch::UiPointerEvent;
use crate::ui::layout::UiPoint;
use crate::ui::surface::UiNavigationEventKind;

use super::{UiDragSessionId, UiInputEventMetadata};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiInputEvent {
    Pointer(UiPointerInputEvent),
    Keyboard(UiKeyboardInputEvent),
    Text(UiTextInputEvent),
    Ime(UiImeInputEvent),
    Navigation(UiNavigationInputEvent),
    Analog(UiAnalogInputEvent),
    DragDrop(UiDragDropInputEvent),
    Popup(UiPopupInputEvent),
    TooltipTimer(UiTooltipTimerInputEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerInputEvent {
    pub metadata: UiInputEventMetadata,
    pub event: UiPointerEvent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precise_scroll: Option<UiPreciseScrollDelta>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPreciseScrollDelta {
    pub x: f32,
    pub y: f32,
    pub unit: UiScrollDeltaUnit,
}

impl UiPreciseScrollDelta {
    pub const fn pixels(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            unit: UiScrollDeltaUnit::Pixels,
        }
    }

    pub const fn lines(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            unit: UiScrollDeltaUnit::Lines,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiScrollDeltaUnit {
    #[default]
    Lines,
    Pixels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiKeyboardInputState {
    Pressed,
    Released,
    Repeated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiKeyboardInputEvent {
    pub metadata: UiInputEventMetadata,
    pub state: UiKeyboardInputState,
    pub key_code: u32,
    pub scan_code: Option<u32>,
    pub physical_key: String,
    pub logical_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextInputEvent {
    pub metadata: UiInputEventMetadata,
    pub text: String,
}

/// UTF-8 byte offsets into the event text, matching Rust string slicing units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiTextByteRange {
    pub start_byte: u32,
    pub end_byte: u32,
}

impl UiTextByteRange {
    pub const fn new(start_byte: u32, end_byte: u32) -> Self {
        Self {
            start_byte,
            end_byte,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiImeInputEventKind {
    Preedit,
    Commit,
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiImeInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiImeInputEventKind,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_range: Option<UiTextByteRange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiNavigationEventKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAnalogInputEvent {
    pub metadata: UiInputEventMetadata,
    pub control: String,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiDragDropInputEventKind {
    Begin,
    Enter,
    Over,
    Leave,
    Drop,
    End,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDragDropInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiDragDropInputEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<UiDragSessionId>,
    pub point: UiPoint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<UiDragPayload>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPopupInputEventKind {
    OpenRequested,
    CloseRequested,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPopupInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiPopupInputEventKind,
    pub popup_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<UiPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiTooltipTimerInputEventKind {
    Armed,
    Elapsed,
    Canceled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTooltipTimerInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiTooltipTimerInputEventKind,
    pub tooltip_id: String,
}
