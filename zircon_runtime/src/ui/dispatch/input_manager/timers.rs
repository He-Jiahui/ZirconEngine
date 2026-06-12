use zircon_runtime_interface::ui::dispatch::UiInputTimestamp;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiInputTimerState {
    last_tick: Option<UiInputTimestamp>,
}

impl UiInputTimerState {
    pub fn last_tick(&self) -> Option<UiInputTimestamp> {
        self.last_tick
    }

    pub fn record_tick(&mut self, now: UiInputTimestamp) {
        self.last_tick = Some(now);
    }
}
