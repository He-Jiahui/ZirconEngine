use crate::core::framework::input::{InputButton, InputEvent};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

const INPUT_CAPACITY: usize = 1024;
const INPUT_TEXT_BYTES: usize = 128;

#[derive(Default)]
struct Queue {
    events: VecDeque<InputEvent>,
    omitted: u64,
}
/// Browser callbacks retain this bounded ingress, never the engine or its World.
pub struct BrowserInput {
    queue: Mutex<Queue>,
    visible: AtomicBool,
    closed: AtomicBool,
    lifecycle_generation: AtomicU64,
}
impl Default for BrowserInput {
    fn default() -> Self {
        Self {
            queue: Mutex::new(Queue::default()),
            visible: AtomicBool::new(true),
            closed: AtomicBool::new(false),
            lifecycle_generation: AtomicU64::new(0),
        }
    }
}
impl BrowserInput {
    pub fn push(&self, mut event: InputEvent) {
        if self.closed.load(Ordering::Acquire) {
            return;
        }
        match &mut event {
            InputEvent::KeyboardInput {
                logical_key, text, ..
            } => {
                if let Some(key) = logical_key {
                    truncate(key);
                }
                if let Some(text) = text {
                    truncate(text);
                }
            }
            InputEvent::ButtonPressed(InputButton::Key(key))
            | InputEvent::ButtonReleased(InputButton::Key(key)) => truncate(key),
            InputEvent::CursorMoved { .. }
            | InputEvent::CursorEntered
            | InputEvent::CursorLeft
            | InputEvent::MouseMotion { .. }
            | InputEvent::ButtonPressed(_)
            | InputEvent::ButtonReleased(_)
            | InputEvent::WheelScrolled { .. }
            | InputEvent::MouseWheel(_)
            | InputEvent::FocusLost
            | InputEvent::Touch { .. } => {}
            _ => return,
        }
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        if self.closed.load(Ordering::Acquire) {
            return;
        }
        if matches!(event, InputEvent::CursorMoved { .. })
            && matches!(queue.events.back(), Some(InputEvent::CursorMoved { .. }))
        {
            *queue.events.back_mut().expect("last cursor event exists") = event;
            return;
        }
        if queue.events.len() == INPUT_CAPACITY {
            queue.omitted = queue.omitted.saturating_add(queue.events.len() as u64);
            queue.events.clear();
            // Losing a release must not leave keys or pointers captured.
            queue.events.push_back(InputEvent::FocusLost);
        }
        queue.events.push_back(event);
    }
    pub fn set_visible(&self, visible: bool) {
        if self.visible.swap(visible, Ordering::AcqRel) != visible {
            self.lifecycle_generation.fetch_add(1, Ordering::AcqRel);
            if !visible {
                self.push(InputEvent::FocusLost);
            }
        }
    }
    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::Acquire)
    }
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
    pub fn lifecycle_generation(&self) -> u64 {
        self.lifecycle_generation.load(Ordering::Acquire)
    }
    pub fn omitted_events(&self) -> u64 {
        self.queue.lock().unwrap_or_else(|e| e.into_inner()).omitted
    }
    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .events
            .clear();
    }
    pub(crate) fn drain(&self, events: &mut Vec<InputEvent>) {
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        events.clear();
        events.extend(queue.events.drain(..));
    }
}
fn truncate(text: &mut String) {
    if text.len() > INPUT_TEXT_BYTES {
        let mut boundary = INPUT_TEXT_BYTES;
        while !text.is_char_boundary(boundary) {
            boundary -= 1;
        }
        text.truncate(boundary);
        // Callback input can carry a much larger backing allocation.
        text.shrink_to_fit();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stalled_browser_input_is_bounded_and_resets_lost_release_state() {
        let input = BrowserInput::default();
        for _ in 0..20_000 {
            input.push(InputEvent::ButtonPressed(InputButton::MouseLeft));
        }
        let queue = input.queue.lock().unwrap();
        assert!(queue.events.len() <= INPUT_CAPACITY);
        assert!(queue.omitted > 0);
        assert!(matches!(queue.events.front(), Some(InputEvent::FocusLost)));
        drop(queue);
        let mut frame = Vec::new();
        input.drain(&mut frame);
        input.close();
        input.push(InputEvent::ButtonReleased(InputButton::MouseLeft));
        input.drain(&mut frame);
        assert!(frame.is_empty());
    }
    #[test]
    fn browser_lifecycle_resets_clock_boundary_and_coalesces_cursor_motion() {
        let input = BrowserInput::default();
        for i in 0..20_000 {
            input.push(InputEvent::CursorMoved {
                x: i as f32,
                y: 0.0,
            });
        }
        assert_eq!(input.queue.lock().unwrap().events.len(), 1);
        input.set_visible(false);
        input.set_visible(true);
        assert_eq!(input.lifecycle_generation(), 2);
        let mut text = "字".repeat(10_000);
        truncate(&mut text);
        assert!(text.len() <= INPUT_TEXT_BYTES);
        assert!(text.capacity() <= INPUT_TEXT_BYTES);
    }
}
