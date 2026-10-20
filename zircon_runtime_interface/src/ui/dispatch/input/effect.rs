use core::fmt;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ui::component::{UiComponentEvent, UiDragPayload};
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::{UiFrame, UiPoint};
use crate::ui::surface::UiNavigationEventKind;
use crate::ui::text::UiRichLinkTarget;
use crate::ui::tree::{UiCursor, UiDirtyFlags};

use super::{event::UiTextByteRange, UiClipboardRequest, UiDragSessionId, UiPointerId};

/// 周边文本按 UTF-8 字节计长；校验拒绝长度大于或等于此上限的内容。
pub const UI_INPUT_METHOD_SURROUNDING_TEXT_BYTE_LIMIT: usize = 4000;

/// Effects are ordered transient dispatch commands; widgets must persist state through normal data paths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 这些值描述有序的瞬态派发命令；Runtime 再检查目标与策略并执行，不能替代持久控件状态。
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
        /// Shares the immutable operation payload instead of copying it on every drag update.
        payload: Option<Arc<UiDragPayload>>,
    },
    RequestNavigation {
        kind: UiNavigationEventKind,
        policy: UiNavigationRequestPolicy,
    },
    Popup {
        kind: UiPopupEffectKind,
        popup_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        owner: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    },
    Tooltip {
        kind: UiTooltipEffectKind,
        tooltip_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        owner: Option<UiNodeId>,
    },
    DismissTransientUi {
        target: UiTransientDismissalTarget,
        reason: UiTransientDismissalReason,
    },
    RequestInputMethod {
        request: UiInputMethodRequest,
    },
    RequestClipboard {
        request: UiClipboardRequest,
    },
    /// Requests host-side activation of a parser-approved rich-text link.
    RequestLinkActivation {
        target: UiNodeId,
        #[serde(rename = "href")]
        link_target: UiRichLinkTarget,
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
    /// Requests the native Mouse cursor without changing UI ownership.
    RequestCursor {
        cursor: UiCursor,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiTransientDismissalTarget {
    #[default]
    All,
    PopupStack,
    Tooltip,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiTransientDismissalReason {
    #[default]
    Programmatic,
    OutsideInteraction,
    FocusLost,
    WindowAction,
    ApplicationDeactivated,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 发给宿主输入法服务的请求；位置字段使用 surface 坐标，周边文本使用 UTF-8 字节偏移。
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
/// 输入法服务请求类别；UpdateCursor 更新输入位置，Enable/Disable/Reset 控制会话状态。
pub enum UiInputMethodRequestKind {
    Enable,
    Disable,
    Reset,
    UpdateCursor,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 输入法周边文本及光标、锚点和组合区间；所有位置均为 UTF-8 字节偏移。构造和 validate 会检查长度与字符边界，反序列化本身不调用 validate。
pub struct UiInputMethodSurroundingText {
    pub text: String,
    pub cursor_byte: u32,
    pub anchor_byte: u32,
    /// Active composition in committed-text byte offsets, rebased to `text`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_range: Option<UiTextByteRange>,
}

impl UiInputMethodSurroundingText {
    /// 创建并校验周边文本；cursor_byte 与 anchor_byte 必须落在文本的 UTF-8 字符边界内。
    pub fn new(
        text: impl Into<String>,
        cursor_byte: u32,
        anchor_byte: u32,
    ) -> Result<Self, UiInputMethodSurroundingTextError> {
        let value = Self {
            text: text.into(),
            cursor_byte,
            anchor_byte,
            composition_range: None,
        };
        value.validate()?;
        Ok(value)
    }

    /// 设置并校验相对 text 的组合字节区间；失败时返回对应位置错误。
    pub fn with_composition_range(
        mut self,
        composition_range: UiTextByteRange,
    ) -> Result<Self, UiInputMethodSurroundingTextError> {
        self.composition_range = Some(composition_range);
        self.validate()?;
        Ok(self)
    }

    /// 返回 cursor_byte 与 anchor_byte 之间按起止顺序排列的字节区间。
    pub fn selection_range(&self) -> UiTextByteRange {
        UiTextByteRange::new(
            self.cursor_byte.min(self.anchor_byte),
            self.cursor_byte.max(self.anchor_byte),
        )
    }

    /// 检查字节长度上限、光标/锚点字符边界及可选组合区间。
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

        if let Some(composition_range) = self.composition_range {
            let start = composition_range.start_byte as usize;
            let end = composition_range.end_byte as usize;
            if start > end
                || end > self.text.len()
                || !self.text.is_char_boundary(start)
                || !self.text.is_char_boundary(end)
            {
                return Err(UiInputMethodSurroundingTextError::CompositionBadPosition);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 周边文本长度或 UTF-8 字节位置校验失败的分类。
pub enum UiInputMethodSurroundingTextError {
    TextTooLong,
    CursorBadPosition,
    AnchorBadPosition,
    CompositionBadPosition,
}

impl fmt::Display for UiInputMethodSurroundingTextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TextTooLong => formatter.write_str("surrounding text exceeds byte limit"),
            Self::CursorBadPosition => formatter.write_str("cursor byte is not a UTF-8 boundary"),
            Self::AnchorBadPosition => formatter.write_str("anchor byte is not a UTF-8 boundary"),
            Self::CompositionBadPosition => {
                formatter.write_str("composition range is not a UTF-8 byte range")
            }
        }
    }
}

impl std::error::Error for UiInputMethodSurroundingTextError {}

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
