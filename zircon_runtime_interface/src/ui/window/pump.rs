use serde::{Deserialize, Serialize};

use crate::ui::dispatch::UiInputEvent;

use super::UiWindowEvent;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiWindowInputPumpEvent {
    Window(UiWindowEvent),
    Input(UiInputEvent),
}

impl UiWindowInputPumpEvent {
    pub const fn is_redraw_request(&self) -> bool {
        matches!(self, Self::Window(event) if event.is_redraw_request())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWindowInputPumpBatch {
    pub events: Vec<UiWindowInputPumpEvent>,
}

impl UiWindowInputPumpBatch {
    pub fn push(&mut self, event: UiWindowInputPumpEvent) {
        self.events.push(event);
    }

    pub fn push_coalesced(&mut self, event: UiWindowInputPumpEvent) {
        if event.is_redraw_request()
            && self
                .events
                .last()
                .is_some_and(UiWindowInputPumpEvent::is_redraw_request)
        {
            return;
        }
        self.push(event);
    }
}
