use crate::input::{InputEvent, InputEventQueueStatus};

#[derive(Debug, Default)]
pub(in crate::input::runtime) struct FrameEventBuffer {
    events: Vec<InputEvent>,
    coalesced_events: u64,
}

impl FrameEventBuffer {
    pub(in crate::input::runtime) fn begin_frame(&mut self) {
        self.events.clear();
        self.coalesced_events = 0;
    }

    pub(in crate::input::runtime) fn push(&mut self, event: InputEvent) {
        if let Some(previous) = self.events.last_mut() {
            match (previous, &event) {
                (
                    InputEvent::CursorMoved { x, y },
                    InputEvent::CursorMoved {
                        x: next_x,
                        y: next_y,
                    },
                ) => {
                    *x = *next_x;
                    *y = *next_y;
                    self.coalesced_events = self.coalesced_events.saturating_add(1);
                    return;
                }
                (
                    InputEvent::MouseMotion { delta_x, delta_y },
                    InputEvent::MouseMotion {
                        delta_x: next_x,
                        delta_y: next_y,
                    },
                ) => {
                    *delta_x += *next_x;
                    *delta_y += *next_y;
                    self.coalesced_events = self.coalesced_events.saturating_add(1);
                    return;
                }
                _ => {}
            }
        }
        self.events.push(event);
    }

    pub(in crate::input::runtime) fn drain(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.events)
    }

    pub(in crate::input::runtime) fn status(&self) -> InputEventQueueStatus {
        InputEventQueueStatus {
            retained_events: self.events.len() as u32,
            coalesced_events: self.coalesced_events,
        }
    }
}
