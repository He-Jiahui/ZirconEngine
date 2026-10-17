use serde::{Deserialize, Serialize};

use crate::ui::{
    accessibility::UiAccessibilityActionRequest,
    component::UiDragPayload,
    dispatch::{
        UiDragDropInputEventKind, UiDragSessionId, UiImeDeleteSurrounding, UiImeInputEvent,
        UiImeInputEventKind, UiImePreeditClause, UiImePreeditClauseError, UiKeyboardInputState,
        UiPointerEvent, UiPointerId, UiPreciseScrollDelta, UiTextByteRange,
        UiTooltipTimerInputEventKind,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::{UiNavigationEventKind, UiPointerButton, UiPointerEventKind},
};

use super::{UiWindowInputContext, UiWindowPlatformInputEventKind, UiWindowTouchPhase};

/// 拥有平台输入与来源上下文的中间 DTO；可经 [`Self::normalize`] 消费为共享输入。
/// 工厂只组织数据，不发送事件、不维护按键/拖拽状态，也不执行宿主或 UI 操作。
/// 平台输入的中间形态；保留窗口上下文，调用 `normalize` 后才进入共享输入分发。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWindowPlatformInputEvent {
    /// 路由元数据与平台缓存；规范化只保留元数据。
    pub context: UiWindowInputContext,
    /// 拥有的输入内容；各工厂不校验事件序列，范围检查仅由显式可失败入口提供。
    pub kind: UiWindowPlatformInputEventKind,
}

impl UiWindowPlatformInputEvent {
    /// 原样接收拥有的上下文和内容；不校验字段或事件顺序。
    pub const fn new(context: UiWindowInputContext, kind: UiWindowPlatformInputEventKind) -> Self {
        Self { context, kind }
    }

    /// 保存指针事件和可选精确滚动值；位置与滚动单位由来源负责。
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

    /// 构造一个位置不经换算的指针移动。
    pub const fn mouse_move(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Move, point),
            None,
        )
    }

    /// 以当前位置的移动事件表示进入；没有独立的输入进入种类。
    pub const fn cursor_entered(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::mouse_move(context, point)
    }

    /// 以当前位置的取消事件表示离开。
    pub const fn cursor_left(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Cancel, point),
            None,
        )
    }

    /// 沿用离开窗口的取消表示，不另外生成按钮抬起事件。
    pub const fn mouse_capture_lost(context: UiWindowInputContext, point: UiPoint) -> Self {
        Self::cursor_left(context, point)
    }

    /// 以行单位保留竖向滚动量，并同时填入兼容的标量滚动值。
    pub const fn mouse_wheel(context: UiWindowInputContext, point: UiPoint, delta: f32) -> Self {
        Self::pointer(
            context,
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
            Some(UiPreciseScrollDelta::lines(0.0, delta)),
        )
    }

    /// 保留行/像素精确值，兼容标量取竖向分量；不进行单位换算。
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

    /// 保存原始相对位移，不套用窗口 DPI 或指针位置。
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

    /// 构造给定按钮在当前位置的按下，不更新按钮状态表。
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

    /// 构造给定按钮在当前位置的抬起。
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

    /// 生成点击次数为 2 的抬起事件，不补发按下或第一次点击。
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

    /// 保存键状态、数值代码及拥有的物理/逻辑键名和可选文本。
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

    /// `is_repeat` 决定重复或首次按下；保留键码与键名，文本字段固定为 `None`。
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

    /// 生成释放状态，保留调用方提供的键名和扫描码，文本字段固定为 `None`。
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

    /// 将字符编码为 Unicode 数值、字符文本和逻辑键名；物理键名为通用字符标签。
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

    /// 将手柄按钮名映射为键盘式按下或重复；数值键码为零，扫描码与文本为空。
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

    /// 将手柄按钮名映射为键盘式释放；不推导设备身份。
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

    /// 接收拥有的提交文本，不解析或插入文档。
    pub fn text(context: UiWindowInputContext, text: impl Into<String>) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Text { text: text.into() },
        )
    }

    /// 保存 IME 种类和拥有的文本，初始无光标范围、分句或周边删除请求。
    pub fn ime(
        context: UiWindowInputContext,
        kind: UiImeInputEventKind,
        text: impl Into<String>,
    ) -> Self {
        Self::ime_with_cursor_range(context, kind, text, None)
    }

    /// 保存可选 UTF-8 字节范围但不检查边界；无分句与周边删除请求。
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
                preedit_clauses: Vec::new(),
                delete_surrounding: None,
            },
        )
    }

    /// 在构造带格式片段的预编辑事件前校验 UTF-8 字节范围及片段顺序。
    /// 接收平台分段样式，在发布预编辑事件前完成局部范围准入。
    ///
    /// # 参数与返回
    /// `context` 随事件移交；`text` 是预编辑文本，范围使用其 UTF-8 字节偏移。
    /// 可选 `cursor_range` 与各子句端点须位于有效字符边界内；`preedit_clauses`
    /// 按起点排列且互不重叠，允许零长度和间隙，不要求覆盖全文。
    /// 返回拥有文本和子句的预编辑事件，或首个范围错误；不重排或修改输入内容。
    ///
    /// # 约束与参考
    /// 此处显式调用 [`UiImeInputEvent::validate_preedit_payload`]；其他构造器和反序列化
    /// 不会自动执行本项检查。校验不涉及字素簇、子句样式语义或实际文档，也不执行分发。
    pub fn ime_with_preedit_clauses(
        context: UiWindowInputContext,
        text: impl Into<String>,
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: Vec<UiImePreeditClause>,
    ) -> Result<Self, UiImePreeditClauseError> {
        let text = text.into();
        UiImeInputEvent::validate_preedit_payload(&text, cursor_range, &preedit_clauses)?;
        Ok(Self::new(
            context,
            UiWindowPlatformInputEventKind::Ime {
                kind: UiImeInputEventKind::Preedit,
                text,
                cursor_range,
                preedit_clauses,
                delete_surrounding: None,
            },
        ))
    }

    /// 以字节数记录光标前后删除请求，不在此访问文档或检查可删除范围。
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
                preedit_clauses: Vec::new(),
                delete_surrounding: Some(UiImeDeleteSurrounding::new(before_bytes, after_bytes)),
            },
        )
    }

    /// 记录导航意图，不直接改变焦点。
    pub fn navigation(context: UiWindowInputContext, kind: UiNavigationEventKind) -> Self {
        Self::new(context, UiWindowPlatformInputEventKind::Navigation { kind })
    }

    /// 保存控制名称和浮点值，不裁剪、归一化或检查有限性。
    pub fn analog(context: UiWindowInputContext, control: impl Into<String>, value: f32) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Analog {
                control: control.into(),
                value,
            },
        )
    }

    /// 沿用模拟量表示；设备身份须由上下文提供。
    pub fn controller_analog(
        context: UiWindowInputContext,
        control: impl Into<String>,
        value: f32,
    ) -> Self {
        Self::analog(context, control, value)
    }

    /// 保存拖拽阶段、可选会话身份、位置和拥有的载荷，不验证拖拽状态机。
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

    /// 生成进入阶段，可附带拥有的载荷。
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

    /// 生成经过阶段，不附带载荷。
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

    /// 生成离开阶段，不附带载荷。
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

    /// 生成放下阶段，可附带拥有的载荷；实际接收由消费者决定。
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

    /// 生成结束阶段，不附带载荷或执行资源释放协议。
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

    /// 记录弹窗请求与可选所有者、锚点，不创建或关闭 UI。
    pub fn popup(
        context: UiWindowInputContext,
        kind: crate::ui::dispatch::UiPopupInputEventKind,
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

    /// 构造弹窗打开请求，保留可选所有者与锚点。
    pub fn popup_open_requested(
        context: UiWindowInputContext,
        popup_id: impl Into<String>,
        owner: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    ) -> Self {
        Self::popup(
            context,
            crate::ui::dispatch::UiPopupInputEventKind::OpenRequested,
            popup_id,
            owner,
            anchor,
        )
    }

    /// 构造弹窗关闭请求，保留可选所有者，锚点为空。
    pub fn popup_close_requested(
        context: UiWindowInputContext,
        popup_id: impl Into<String>,
        owner: Option<UiNodeId>,
    ) -> Self {
        Self::popup(
            context,
            crate::ui::dispatch::UiPopupInputEventKind::CloseRequested,
            popup_id,
            owner,
            None,
        )
    }

    /// 记录弹窗已被解除的通知，不携带所有者或锚点。
    pub fn popup_dismissed(context: UiWindowInputContext, popup_id: impl Into<String>) -> Self {
        Self::popup(
            context,
            crate::ui::dispatch::UiPopupInputEventKind::Dismissed,
            popup_id,
            None,
            None,
        )
    }

    /// 记录提示计时状态及可选所有者，不创建或取消定时器。
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

    /// 记录提示计时已启动的状态，保存可选所有者。
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

    /// 记录提示计时到期，保存可选所有者。
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

    /// 记录提示计时取消，不携带所有者。
    pub fn tooltip_canceled(context: UiWindowInputContext, tooltip_id: impl Into<String>) -> Self {
        Self::tooltip_timer(
            context,
            UiTooltipTimerInputEventKind::Canceled,
            tooltip_id,
            None,
        )
    }

    /// 接收拥有的无障碍请求，不验证或执行其中动作。
    pub const fn accessibility(
        context: UiWindowInputContext,
        request: UiAccessibilityActionRequest,
    ) -> Self {
        Self::new(
            context,
            UiWindowPlatformInputEventKind::Accessibility { request },
        )
    }

    /// 保存触摸相位、指针身份和位置；规范化时以此身份覆盖上下文并标为触摸来源。
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

    /// 记录触摸开始，后续规范化为主按钮按下。
    pub const fn touch_started(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Started, pointer_id, point)
    }

    /// 记录触摸移动，后续规范化为指针移动。
    pub const fn touch_moved(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Moved, pointer_id, point)
    }

    /// 当前表示只保留移动相位和位置，`_force` 不进入结果。
    pub const fn touch_force_changed(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
        _force: f32,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Moved, pointer_id, point)
    }

    /// 当前表示与普通触摸移动相同，`_force` 不进入结果。
    pub const fn touch_first_move(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
        _force: f32,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Moved, pointer_id, point)
    }

    /// 记录触摸结束，后续规范化为主按钮抬起。
    pub const fn touch_ended(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Ended, pointer_id, point)
    }

    /// 记录触摸取消，后续规范化为指针取消。
    pub const fn touch_canceled(
        context: UiWindowInputContext,
        pointer_id: UiPointerId,
        point: UiPoint,
    ) -> Self {
        Self::touch(context, UiWindowTouchPhase::Canceled, pointer_id, point)
    }
}
