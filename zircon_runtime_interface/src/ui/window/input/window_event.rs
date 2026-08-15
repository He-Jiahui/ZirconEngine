use crate::ui::{
    dispatch::{UiInputEvent, UiPointerEvent},
    layout::UiPoint,
    surface::UiPointerEventKind,
};

use super::super::{UiWindowEvent, UiWindowEventKind};
use super::{UiWindowInputContext, UiWindowPlatformInputEvent};

impl UiWindowEvent {
    pub fn input_context(&self) -> UiWindowInputContext {
        UiWindowInputContext::from_window_metadata(&self.metadata)
    }

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
