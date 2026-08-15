use serde::{Deserialize, Serialize};

use crate::ui::{
    dispatch::UiPointerEvent,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiWindowTouchPhase {
    Started,
    Moved,
    Ended,
    Canceled,
}

impl UiWindowTouchPhase {
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
