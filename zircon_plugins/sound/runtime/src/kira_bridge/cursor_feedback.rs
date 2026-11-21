//! Orders control-side cursor handoff against Kira's block-start position publication.
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub(super) struct CursorFeedback {
    requested: AtomicU64,
    published: AtomicU64,
}

impl CursorFeedback {
    /// Request only after enqueuing the control. Multiple voices may share one mixer block.
    pub(super) fn request(&self) -> u64 {
        self.requested
            .fetch_add(1, Ordering::Release)
            .wrapping_add(1)
    }

    pub(super) fn has_published(&self, request: u64) -> bool {
        self.published.load(Ordering::Acquire).wrapping_sub(request) < (1 << 63)
    }

    /// Kira publishes the *previous* block's position, then consumes controls for this block.
    /// A request racing this call is safe on either side of the acquire: at worst it waits
    /// one more block. No callback takes the service mutex or acknowledges device output.
    pub(super) fn start_block(&self, previous: &mut u64, kira_start: impl FnOnce()) {
        let current = self.requested.load(Ordering::Acquire);
        kira_start();
        // Only the render callback writes this stamp. Ordinary playback need not dirty
        // a shared cache line on every block when no handoff is pending.
        if self.published.load(Ordering::Relaxed) != *previous {
            self.published.store(*previous, Ordering::Release);
        }
        *previous = current;
    }
}
