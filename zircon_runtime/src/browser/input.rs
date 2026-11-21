use crate::core::framework::input::{InputButton, InputEvent, InputManager};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};

const INPUT_CAPACITY: usize = 1024;
const INPUT_TEXT_BYTES: usize = 128;

#[derive(Default)]
struct Queue {
    // Distinct FIFO gaps, at most one before/between/after each raw event.
    canvas_blurs: Vec<usize>,
    events: VecDeque<InputEvent>,
    omitted: u64,
}
#[derive(Default)]
pub(crate) struct BrowserInputFrame {
    events: Vec<InputEvent>,
    canvas_blurs: Vec<usize>,
}
impl BrowserInputFrame {
    pub(crate) fn clear(&mut self) {
        self.events.clear();
        self.canvas_blurs.clear();
    }
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
            && queue.canvas_blurs.last() != Some(&queue.events.len())
        {
            *queue.events.back_mut().expect("last cursor event exists") = event;
            return;
        }
        if queue.events.len() == INPUT_CAPACITY {
            queue.omitted = queue.omitted.saturating_add(queue.events.len() as u64);
            queue.events.clear();
            queue.canvas_blurs.clear();
            // Losing a release must not leave keys or pointers captured.
            queue.events.push_back(InputEvent::FocusLost);
        }
        queue.events.push_back(event);
    }
    /// End the owning canvas's keyboard input without losing window focus.
    pub fn canvas_keyboard_blurred(&self) {
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        if self.is_closed() {
            return;
        }
        // A local blur must not consume capacity or cause an all-input reset.
        let gap = queue.events.len();
        if queue.canvas_blurs.last() != Some(&gap) {
            queue.canvas_blurs.push(gap);
        }
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
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        queue.events.clear();
        queue.canvas_blurs.clear();
    }
    pub(crate) fn submit_frame(
        &self,
        frame: &mut BrowserInputFrame,
        manager: &dyn InputManager,
        mut visit_raw: impl FnMut(&InputEvent) -> Result<(), String>,
    ) -> Result<(), String> {
        if self.is_closed() {
            frame.clear();
            return Ok(());
        }
        manager.begin_frame();
        self.drain(frame);
        let mut canvas_blurs = frame.canvas_blurs.drain(..).peekable();
        for (index, event) in frame.events.drain(..).enumerate() {
            if canvas_blurs.next_if_eq(&index).is_some() {
                self.release_canvas_keyboard(manager);
            }
            visit_raw(&event)?;
            manager.submit_event(event);
        }
        // A blur can be the final item, including in a frame with no raw events.
        if canvas_blurs.next().is_some() {
            self.release_canvas_keyboard(manager);
        }
        Ok(())
    }
    fn release_canvas_keyboard(&self, manager: &dyn InputManager) {
        if self.is_closed() {
            return;
        }
        // Release the actual held identities; never send a synthetic key to UI.
        for button in manager.snapshot().pressed_buttons {
            if matches!(&button, InputButton::Key(_) | InputButton::KeyCode(_)) {
                manager.submit_event(InputEvent::ButtonReleased(button));
            }
        }
    }
    fn drain(&self, frame: &mut BrowserInputFrame) {
        let mut queue = self.queue.lock().unwrap_or_else(|e| e.into_inner());
        frame.clear();
        frame.events.extend(queue.events.drain(..));
        std::mem::swap(&mut frame.canvas_blurs, &mut queue.canvas_blurs);
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
        let mut frame = BrowserInputFrame::default();
        input.drain(&mut frame);
        input.close();
        input.canvas_keyboard_blurred();
        input.push(InputEvent::ButtonReleased(InputButton::MouseLeft));
        input.drain(&mut frame);
        assert!(frame.events.is_empty());
        assert!(frame.canvas_blurs.is_empty());
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
