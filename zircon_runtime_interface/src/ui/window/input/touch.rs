use serde::{Deserialize, Serialize};

use crate::ui::{
    dispatch::UiPointerEvent,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
};

/// 触摸生命周期在规范化时映射为主按钮指针事件；不携带压力或手势状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowTouchPhase {
    /// 开始接触，对应主按钮按下。
    Started,
    /// 接触点移动，对应指针移动。
    Moved,
    /// 接触结束，对应主按钮抬起。
    Ended,
    /// 接触被取消，对应指针取消。
    Canceled,
}

impl UiWindowTouchPhase {
    /// 将触摸生命周期映射到主按钮指针协议，后续命中与组件事件可复用指针路由。
    pub(super) const fn pointer_event(self, point: UiPoint) -> UiPointerEvent {
        match self {
            Self::Started => UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
            Self::Moved => UiPointerEvent::new(UiPointerEventKind::Move, point),
            Self::Ended => UiPointerEvent::new(UiPointerEventKind::Up, point)
                .with_button(UiPointerButton::Primary),
            Self::Canceled => UiPointerEvent::new(UiPointerEventKind::Cancel, point),
        }
    }
}
