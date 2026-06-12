use zircon_runtime_interface::ui::{
    dispatch::{UiInputDispatchResult, UiInputEvent, UiInputTimestamp},
    tree::UiTreeError,
    window::{UiWindowInputPumpBatch, UiWindowInputPumpEvent},
};

use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::{input, UiSurface},
};

use super::{
    outcome::UiInputDispatchOutcome, pointer_table::UiActivePointerTable, timers::UiInputTimerState,
};

#[derive(Default)]
pub struct UiInputManager {
    pointer: UiPointerDispatcher,
    navigation: UiNavigationDispatcher,
    pointers: UiActivePointerTable,
    timers: UiInputTimerState,
}

impl UiInputManager {
    pub fn pointer_dispatcher(&self) -> &UiPointerDispatcher {
        &self.pointer
    }

    pub fn pointer_dispatcher_mut(&mut self) -> &mut UiPointerDispatcher {
        &mut self.pointer
    }

    pub fn navigation_dispatcher(&self) -> &UiNavigationDispatcher {
        &self.navigation
    }

    pub fn navigation_dispatcher_mut(&mut self) -> &mut UiNavigationDispatcher {
        &mut self.navigation
    }

    pub fn active_pointers(&self) -> &UiActivePointerTable {
        &self.pointers
    }

    pub fn active_pointers_mut(&mut self) -> &mut UiActivePointerTable {
        &mut self.pointers
    }

    pub fn timers(&self) -> &UiInputTimerState {
        &self.timers
    }

    pub fn dispatch_input_event(
        &mut self,
        surface: &mut UiSurface,
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        input::dispatch_input_event(surface, &self.pointer, &self.navigation, event)
    }

    pub fn dispatch_window_event(
        &mut self,
        surface: &mut UiSurface,
        event: UiWindowInputPumpEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        input::dispatch_window_input_pump_event(surface, &self.pointer, &self.navigation, event)
    }

    pub fn dispatch_window_batch(
        &mut self,
        surface: &mut UiSurface,
        batch: UiWindowInputPumpBatch,
    ) -> Result<UiInputDispatchOutcome, UiTreeError> {
        let mut results = Vec::with_capacity(batch.events.len());
        for event in batch.events {
            results.push(self.dispatch_window_event(surface, event)?);
        }
        Ok(UiInputDispatchOutcome::from_results(surface, results))
    }

    pub fn tick(
        &mut self,
        _surface: &mut UiSurface,
        now: UiInputTimestamp,
    ) -> Result<Vec<UiInputDispatchResult>, UiTreeError> {
        self.timers.record_tick(now);
        Ok(Vec::new())
    }
}
