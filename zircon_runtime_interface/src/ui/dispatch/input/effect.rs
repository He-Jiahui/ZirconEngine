use core::fmt;

use serde::{Deserialize, Serialize};

use crate::ui::component::{UiComponentEvent, UiDragPayload};
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::{UiFrame, UiPoint};
use crate::ui::surface::UiNavigationEventKind;
use crate::ui::tree::UiDirtyFlags;

use super::{event::UiTextByteRange, UiDragSessionId, UiPointerId};

pub const UI_INPUT_METHOD_SURROUNDING_TEXT_BYTE_LIMIT: usize = 4000;

/// Effects are ordered transient dispatch commands; widgets must persist state through normal data paths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiDispatchEffect {
    SetFocus {
        target: UiNodeId,
        reason: UiFocusEffectReason,
    },
    ClearFocus {
        target: UiNodeId,
        reason: UiFocusEffectReason,
    },
    CapturePointer {
        target: UiNodeId,
        pointer_id: UiPointerId,
        reason: UiPointerCaptureReason,
    },
    ReleasePointerCapture {
        target: UiNodeId,
        pointer_id: UiPointerId,
        reason: UiPointerCaptureReason,
    },
    LockPointer {
        target: UiNodeId,
        policy: UiPointerLockPolicy,
    },
    UnlockPointer {
        target: UiNodeId,
        policy: UiPointerLockPolicy,
    },
    UseHighPrecisionPointer {
        target: UiNodeId,
        enabled: bool,
    },
    DragDrop {
        kind: UiDragDropEffectKind,
        target: UiNodeId,
        pointer_id: UiPointerId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<UiDragSessionId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        point: Option<UiPoint>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<UiDragPayload>,
    },
    RequestNavigation {
        kind: UiNavigationEventKind,
        policy: UiNavigationRequestPolicy,
    },
    Popup {
        kind: UiPopupEffectKind,
        popup_id: String,
        anchor: Option<UiPoint>,
    },
    Tooltip {
        kind: UiTooltipEffectKind,
        tooltip_id: String,
    },
    RequestInputMethod {
        request: UiInputMethodRequest,
    },
    RequestClipboard {
        request: UiClipboardRequest,
    },
    DirtyRedraw {
        target: UiNodeId,
        dirty: UiDirtyFlags,
        reason: UiRedrawRequestReason,
    },
    EmitComponentEvent {
        target: UiNodeId,
        event: UiComponentEvent,
        policy: UiComponentEmissionPolicy,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiFocusEffectReason {
    #[default]
    Input,
    Navigation,
    Programmatic,
    Dismissal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerCaptureReason {
    #[default]
    Press,
    Drag,
    Programmatic,
    Cancel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerLockPolicy {
    #[default]
    Confined,
    HiddenCursor,
    RawDelta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiDragDropEffectKind {
    Begin,
    Update,
    Accept,
    Reject,
    Complete,
    Cancel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiNavigationRequestPolicy {
    #[default]
    Bubble,
    Direct,
    Wrap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPopupEffectKind {
    Open,
    Close,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiTooltipEffectKind {
    Arm,
    Show,
    Hide,
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiClipboardRequest {
    pub kind: UiClipboardRequestKind,
    pub owner: UiNodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiClipboardRequestKind {
    ReadText,
    WriteText,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiInputMethodRequest {
    pub kind: UiInputMethodRequestKind,
    pub owner: UiNodeId,
    /// Surface-space cursor rectangle used by native candidate windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_rect: Option<UiFrame>,
    /// Surface-space composition rectangles for active IME spans.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub composition_rects: Vec<UiFrame>,
    /// Text around the caret, excluding active preedit text, for native IME context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surrounding_text: Option<UiInputMethodSurroundingText>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiInputMethodRequestKind {
    Enable,
    Disable,
    Reset,
    UpdateCursor,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiInputMethodSurroundingText {
    pub text: String,
    pub cursor_byte: u32,
    pub anchor_byte: u32,
}

impl UiInputMethodSurroundingText {
    pub fn new(
        text: impl Into<String>,
        cursor_byte: u32,
        anchor_byte: u32,
    ) -> Result<Self, UiInputMethodSurroundingTextError> {
        let value = Self {
            text: text.into(),
            cursor_byte,
            anchor_byte,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn selection_range(&self) -> UiTextByteRange {
        UiTextByteRange::new(
            self.cursor_byte.min(self.anchor_byte),
            self.cursor_byte.max(self.anchor_byte),
        )
    }

    pub fn validate(&self) -> Result<(), UiInputMethodSurroundingTextError> {
        if self.text.len() >= UI_INPUT_METHOD_SURROUNDING_TEXT_BYTE_LIMIT {
            return Err(UiInputMethodSurroundingTextError::TextTooLong);
        }

        let cursor = self.cursor_byte as usize;
        if cursor > self.text.len() || !self.text.is_char_boundary(cursor) {
            return Err(UiInputMethodSurroundingTextError::CursorBadPosition);
        }

        let anchor = self.anchor_byte as usize;
        if anchor > self.text.len() || !self.text.is_char_boundary(anchor) {
            return Err(UiInputMethodSurroundingTextError::AnchorBadPosition);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputMethodSurroundingTextError {
    TextTooLong,
    CursorBadPosition,
    AnchorBadPosition,
}

impl fmt::Display for UiInputMethodSurroundingTextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TextTooLong => formatter.write_str("surrounding text exceeds byte limit"),
            Self::CursorBadPosition => formatter.write_str("cursor byte is not a UTF-8 boundary"),
            Self::AnchorBadPosition => formatter.write_str("anchor byte is not a UTF-8 boundary"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiRedrawRequestReason {
    #[default]
    Input,
    Animation,
    Style,
    Layout,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiComponentEmissionPolicy {
    #[default]
    Immediate,
    Queue,
    Coalesce,
}
