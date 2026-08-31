use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorJobEventJournalSnapshot {
    pub(super) depth: usize,
    pub(super) retained_bytes: usize,
    pub(super) oldest_age: Option<Duration>,
    pub(super) high_water_depth: usize,
    pub(super) high_water_retained_bytes: usize,
    pub(super) coalesced_progress_events: u64,
    pub(super) dropped_progress_events: u64,
    pub(super) dropped_lifecycle_events: u64,
    pub(super) sequence_exhausted: bool,
}

impl EditorJobEventJournalSnapshot {
    pub const fn depth(self) -> usize {
        self.depth
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    pub const fn oldest_age(self) -> Option<Duration> {
        self.oldest_age
    }

    pub const fn high_water_depth(self) -> usize {
        self.high_water_depth
    }

    pub const fn high_water_retained_bytes(self) -> usize {
        self.high_water_retained_bytes
    }

    pub const fn coalesced_progress_events(self) -> u64 {
        self.coalesced_progress_events
    }

    pub const fn dropped_progress_events(self) -> u64 {
        self.dropped_progress_events
    }

    pub const fn dropped_lifecycle_events(self) -> u64 {
        self.dropped_lifecycle_events
    }

    pub const fn sequence_exhausted(self) -> bool {
        self.sequence_exhausted
    }
}
