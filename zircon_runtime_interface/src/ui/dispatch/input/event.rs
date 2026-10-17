use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::ui::accessibility::UiAccessibilityActionRequest;
use crate::ui::component::UiDragPayload;
use crate::ui::dispatch::UiPointerEvent;
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiPoint;
use crate::ui::surface::UiNavigationEventKind;

pub use crate::ui::surface::{
    UiTextByteRange, UiTextPreeditClause as UiImePreeditClause,
    UiTextPreeditClauseError as UiImePreeditClauseError,
    UiTextPreeditClauseKind as UiImePreeditClauseKind,
};

use super::{UiClipboardInputEvent, UiDragSessionId, UiInputEventMetadata};

/// 窗口适配层归一化后的共享输入契约；Runtime 分发器在此层消费平台事件与内部计时事件。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiInputEvent {
    Pointer(UiPointerInputEvent),
    Keyboard(UiKeyboardInputEvent),
    Text(UiTextInputEvent),
    Ime(UiImeInputEvent),
    Clipboard(UiClipboardInputEvent),
    Navigation(UiNavigationInputEvent),
    Analog(UiAnalogInputEvent),
    MouseMotion(UiMouseMotionInputEvent),
    DragDrop(UiDragDropInputEvent),
    Popup(UiPopupInputEvent),
    TooltipTimer(UiTooltipTimerInputEvent),
    TypeaheadTimer(UiTypeaheadTimerInputEvent),
    SubmenuHoverTimer(UiSubmenuHoverTimerInputEvent),
    ToastTimer(UiToastTimerInputEvent),
    Accessibility(UiAccessibilityInputEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 归一化后的指针输入及其来源元数据，可附带保留原单位的精确滚动增量。
pub struct UiPointerInputEvent {
    pub metadata: UiInputEventMetadata,
    pub event: UiPointerEvent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precise_scroll: Option<UiPreciseScrollDelta>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// 带显式单位的二维滚动增量；x/y 的数值按 unit 解释，不在此处换算。
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
/// 精确滚动增量采用的单位：Lines 表示行数，Pixels 表示像素。
pub enum UiScrollDeltaUnit {
    #[default]
    Lines,
    Pixels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 键盘动作状态，区分首次按下、释放和系统重复。
pub enum UiKeyboardInputState {
    Pressed,
    Released,
    Repeated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 归一化键盘事件，同时保留物理键、逻辑键及可选文本，不自行解释键码平台映射。
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
/// 已提交的文本输入片段；组合态文本由 UiImeInputEvent 表示。
pub struct UiTextInputEvent {
    pub metadata: UiInputEventMetadata,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 输入法事件阶段：预编辑、提交、取消或按字节请求删除周边文本。
pub enum UiImeInputEventKind {
    Preedit,
    Commit,
    Cancel,
    DeleteSurrounding,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 光标前后请求删除的 UTF-8 字节数；实际删除仍由 Runtime 文本编辑路径处理。
pub struct UiImeDeleteSurrounding {
    pub before_bytes: u32,
    pub after_bytes: u32,
}

impl UiImeDeleteSurrounding {
    pub const fn new(before_bytes: u32, after_bytes: u32) -> Self {
        Self {
            before_bytes,
            after_bytes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 输入法组合事件；validate 要求组合 clauses 仅出现在 Preedit，并验证预编辑文本中的字节范围。
pub struct UiImeInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiImeInputEventKind,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_range: Option<UiTextByteRange>,
    /// Optional platform conversion styling for `Preedit` text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preedit_clauses: Vec<UiImePreeditClause>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_surrounding: Option<UiImeDeleteSurrounding>,
}

impl UiImeInputEvent {
    /// 校验当前事件的 preedit clauses 与文本范围关系。
    pub fn validate(&self) -> Result<(), UiImePreeditClauseError> {
        if self.kind != UiImeInputEventKind::Preedit {
            return self
                .preedit_clauses
                .is_empty()
                .then_some(())
                .ok_or(UiImePreeditClauseError::ClausesRequirePreedit);
        }
        UiImePreeditClause::validate_preedit_payload(
            &self.text,
            self.cursor_range,
            &self.preedit_clauses,
        )
    }

    /// Validates preedit-local byte ranges and their non-overlapping text order.
    pub fn validate_preedit_payload(
        text: &str,
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[UiImePreeditClause],
    ) -> Result<(), UiImePreeditClauseError> {
        UiImePreeditClause::validate_preedit_payload(text, cursor_range, preedit_clauses)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 已归一化的导航动作及其输入来源元数据。
pub struct UiNavigationInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiNavigationEventKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 带设备元数据的控制名称与模拟数值；数值含义由对应控制的消费方定义。
pub struct UiAnalogInputEvent {
    pub metadata: UiInputEventMetadata,
    pub control: String,
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 鼠标运动的二维增量及其输入来源元数据。
pub struct UiMouseMotionInputEvent {
    pub metadata: UiInputEventMetadata,
    pub delta_x: f32,
    pub delta_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 拖放输入阶段，记录一次拖动中的开始、进入、悬停、离开、投放或结束通知。
pub enum UiDragDropInputEventKind {
    Begin,
    Enter,
    Over,
    Leave,
    Drop,
    End,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 拖放阶段、位置和可选共享 payload；payload 由 Arc 共享，避免更新事件反复复制操作数据。
pub struct UiDragDropInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiDragDropInputEventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<UiDragSessionId>,
    pub point: UiPoint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Immutable operation payload shared by the input result, effect, and retained state.
    pub payload: Option<Arc<UiDragPayload>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 弹出层输入通知，区分打开请求、关闭请求与已关闭状态。
pub enum UiPopupInputEventKind {
    OpenRequested,
    CloseRequested,
    Dismissed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 弹出层标识及可选 owner/anchor 的输入通知；该 DTO 不直接创建或关闭弹出层。
pub struct UiPopupInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiPopupInputEventKind,
    pub popup_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<UiPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// tooltip 定时器的布防、到期和取消通知。
pub enum UiTooltipTimerInputEventKind {
    Armed,
    Elapsed,
    Canceled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 关联 tooltip 标识与可选 owner 的定时器通知。
pub struct UiTooltipTimerInputEvent {
    pub metadata: UiInputEventMetadata,
    pub kind: UiTooltipTimerInputEventKind,
    pub tooltip_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<UiNodeId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 发给指定节点的 typeahead 定时器通知。
pub struct UiTypeaheadTimerInputEvent {
    pub metadata: UiInputEventMetadata,
    pub target: UiNodeId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 发给指定节点与选项的子菜单悬停定时器通知。
pub struct UiSubmenuHoverTimerInputEvent {
    pub metadata: UiInputEventMetadata,
    pub target: UiNodeId,
    pub option_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 发给指定节点与 toast 的定时器通知。
pub struct UiToastTimerInputEvent {
    pub metadata: UiInputEventMetadata,
    pub target: UiNodeId,
    pub toast_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 携带无障碍动作请求的输入事件；请求本身的校验由 Runtime 对应处理路径完成。
pub struct UiAccessibilityInputEvent {
    pub metadata: UiInputEventMetadata,
    pub request: UiAccessibilityActionRequest,
}
