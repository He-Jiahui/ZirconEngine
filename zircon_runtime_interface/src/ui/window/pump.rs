use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiDispatchEffect, UiInputEvent};

use super::{UiWindowEvent, UiWindowPlatformInputEvent};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiWindowInputPumpEvent {
    Window(UiWindowEvent),
    Input(UiInputEvent),
}

impl UiWindowInputPumpEvent {
    pub const fn is_redraw_request(&self) -> bool {
        matches!(self, Self::Window(event) if event.is_redraw_request())
    }

    pub fn transient_dismissal_effect(&self) -> Option<UiDispatchEffect> {
        match self {
            Self::Window(event) => event.transient_dismissal_effect(),
            Self::Input(_) => None,
        }
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

    pub fn push_platform_input(&mut self, event: UiWindowPlatformInputEvent) {
        self.push(UiWindowInputPumpEvent::Input(event.normalize()));
    }

    pub fn transient_dismissal_effects(&self) -> impl Iterator<Item = UiDispatchEffect> + '_ {
        self.events
            .iter()
            .filter_map(UiWindowInputPumpEvent::transient_dismissal_effect)
    }
}
