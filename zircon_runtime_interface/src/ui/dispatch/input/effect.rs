use serde::{Deserialize, Serialize};

use crate::ui::component::{UiComponentEvent, UiDragPayload};
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::{UiFrame, UiPoint};
use crate::ui::surface::UiNavigationEventKind;
use crate::ui::tree::UiDirtyFlags;

use super::{UiDragSessionId, UiPointerId};

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiInputMethodRequestKind {
    Enable,
    Disable,
    Reset,
    UpdateCursor,
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
