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

/// 平台适配器拥有的输入内容，使用 Rust/serde 表示，不作为固定 C ABI 布局。
/// 字段在规范化时移入共享 DTO；除触摸身份重写外不在此补全或验证内容。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiWindowPlatformInputEventKind {
    /// 指针事件与可选精确滚动信息；兼容标量仍在内部指针事件中。
    Pointer {
        /// 位置、按钮、点击次数和兼容滚动量由来源提供。
        event: UiPointerEvent,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 保留行/像素单位；为空时只有兼容标量。
        precise_scroll: Option<UiPreciseScrollDelta>,
    },
    /// 键状态、数值代码、拥有的键名与可选文本，物理键与逻辑键含义分别保留。
    Keyboard {
        state: UiKeyboardInputState,
        key_code: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 可选扫描码，未提供时保持为空。
        scan_code: Option<u32>,
        physical_key: String,
        logical_key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    Text {
        /// 拥有的提交文本，不代表已经写入编辑文档。
        text: String,
    },
    /// IME 内容；范围和周边删除数量均以 UTF-8 字节计，本枚举不校验它们。
    Ime {
        kind: UiImeInputEventKind,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 预编辑文本中的可选光标范围。
        cursor_range: Option<UiTextByteRange>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        /// 预编辑分句；显式分句工厂会校验，直接构造或反序列化不会调用该工厂。
        preedit_clauses: Vec<UiImePreeditClause>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 可选的光标前后删除请求，不拥有目标文档。
        delete_surrounding: Option<UiImeDeleteSurrounding>,
    },
    Navigation {
        /// 供分发器处理的导航意图。
        kind: UiNavigationEventKind,
    },
    Analog {
        /// 拥有的控制名称；对应浮点值不在此归一化。
        control: String,
        value: f32,
    },
    MouseMotion {
        /// 原始相对位移分量；两个方向均不隐含窗口坐标或 DPI 变换。
        delta_x: f32,
        delta_y: f32,
    },
    /// 拖拽阶段、位置与可选拥有的载荷；规范化后载荷使用共享所有权。
    DragDrop {
        kind: UiDragDropInputEventKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 来源提供的可选拖拽会话，不由本 DTO 分配。
        session_id: Option<UiDragSessionId>,
        point: UiPoint,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 可选拥有的载荷，未附带时不从历史事件补取。
        payload: Option<UiDragPayload>,
    },
    /// 弹窗状态或请求；所有者与锚点均可缺省。
    Popup {
        kind: UiPopupInputEventKind,
        popup_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 可选所有者节点，不保证节点当前仍存在。
        owner: Option<UiNodeId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 可选锚点位置，实际定位由消费者决定。
        anchor: Option<UiPoint>,
    },
    /// 提示计时的状态通知；不包含定时器资源。
    TooltipTimer {
        kind: UiTooltipTimerInputEventKind,
        tooltip_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        /// 可选所有者节点。
        owner: Option<UiNodeId>,
    },
    /// 拥有的无障碍动作请求，执行与对象查找由消费者承担。
    Accessibility {
        request: UiAccessibilityActionRequest,
    },
    /// 触摸相位与身份；规范化为主按钮指针事件，不携带压力值。
    Touch {
        phase: UiWindowTouchPhase,
        pointer_id: UiPointerId,
        point: UiPoint,
    },
}
