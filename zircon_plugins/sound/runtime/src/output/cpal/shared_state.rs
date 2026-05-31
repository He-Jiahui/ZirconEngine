use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use super::super::ring_buffer::SoundOutputRingBuffer;

#[derive(Debug)]
pub(in crate::output::cpal) struct CpalOutputSharedState {
    pub(in crate::output::cpal) ring_buffer: Mutex<SoundOutputRingBuffer>,
    pub(in crate::output::cpal) producer_rendered_blocks: AtomicU64,
    pub(in crate::output::cpal) producer_rendered_frames: AtomicU64,
    pub(in crate::output::cpal) callback_count: AtomicU64,
    pub(in crate::output::cpal) last_callback_sequence: AtomicU64,
    pub(in crate::output::cpal) underrun_count: AtomicU64,
    last_error: Mutex<Option<String>>,
}

impl CpalOutputSharedState {
    pub(in crate::output::cpal) fn new(ring_buffer: SoundOutputRingBuffer) -> Self {
        Self {
            ring_buffer: Mutex::new(ring_buffer),
            producer_rendered_blocks: AtomicU64::new(0),
            producer_rendered_frames: AtomicU64::new(0),
            callback_count: AtomicU64::new(0),
            last_callback_sequence: AtomicU64::new(0),
            underrun_count: AtomicU64::new(0),
            last_error: Mutex::new(None),
        }
    }

    pub(in crate::output::cpal) fn record_error(&self, detail: String) {
        if let Ok(mut last_error) = self.last_error.lock() {
            *last_error = Some(detail);
        }
    }

    pub(in crate::output::cpal) fn clear_error(&self) {
        if let Ok(mut last_error) = self.last_error.lock() {
            *last_error = None;
        }
    }

    pub(in crate::output::cpal) fn last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|error| error.clone())
    }

    pub(in crate::output::cpal) fn ring_buffer_status(&self) -> (Option<usize>, Option<usize>) {
        self.ring_buffer
            .lock()
            .map(|buffer| {
                (
                    Some(buffer.available_samples()),
                    Some(buffer.capacity_samples()),
                )
            })
            .unwrap_or((None, None))
    }
}
