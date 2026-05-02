use serde::{Deserialize, Serialize};

use crate::ui::layout::UiPoint;
use crate::ui::surface::{UiPointerButton, UiPointerEventKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerEvent {
    pub kind: UiPointerEventKind,
    pub button: Option<UiPointerButton>,
    pub point: UiPoint,
    pub scroll_delta: f32,
}

impl UiPointerEvent {
    pub const fn new(kind: UiPointerEventKind, point: UiPoint) -> Self {
        Self {
            kind,
            button: None,
            point,
            scroll_delta: 0.0,
        }
    }

    pub const fn with_button(mut self, button: UiPointerButton) -> Self {
        self.button = Some(button);
        self
    }

    pub const fn with_scroll_delta(mut self, scroll_delta: f32) -> Self {
        self.scroll_delta = scroll_delta;
        self
    }
}
