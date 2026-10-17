use crate::ui::{
    dispatch::{UiInputEvent, UiPointerEvent},
    layout::UiPoint,
    surface::UiPointerEventKind,
};

use super::super::{UiWindowEvent, UiWindowEventKind};
use super::{UiWindowInputContext, UiWindowPlatformInputEvent};

impl UiWindowEvent {
    /// 仅从窗口元数据创建输入上下文；不从事件种类推导度量或光标缓存。
    pub fn input_context(&self) -> UiWindowInputContext {
        UiWindowInputContext::from_window_metadata(&self.metadata)
    }

    /// 仅将光标移动通知转成共享指针移动输入；位置原样保留，可选位移不传入结果。
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

    /// 离开或销毁窗口时沿用该窗口事件的元数据生成指针取消，供 Runtime 与 Editor 清理悬停和捕获。
    /// `point` 由调用方提供；只对光标离开、关闭和销毁生成取消输入，其他种类返回 `None`。
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
