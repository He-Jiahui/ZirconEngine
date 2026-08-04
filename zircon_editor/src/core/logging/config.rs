use super::EditorLogError;

const DEFAULT_ENTRY_CAPACITY: usize = 2_048;
const DEFAULT_RETAINED_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_EVENT_QUEUE_ENTRY_CAPACITY: usize = 256;
const DEFAULT_EVENT_QUEUE_RETAINED_BYTES: usize = 512 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorLogConfig {
    entry_capacity: usize,
    retained_bytes: usize,
    event_queue_entry_capacity: usize,
    event_queue_retained_bytes: usize,
}

impl Default for EditorLogConfig {
    fn default() -> Self {
        Self {
            entry_capacity: DEFAULT_ENTRY_CAPACITY,
            retained_bytes: DEFAULT_RETAINED_BYTES,
            event_queue_entry_capacity: DEFAULT_EVENT_QUEUE_ENTRY_CAPACITY,
            event_queue_retained_bytes: DEFAULT_EVENT_QUEUE_RETAINED_BYTES,
        }
    }
}

impl EditorLogConfig {
    pub fn new(entry_capacity: usize, retained_bytes: usize) -> Result<Self, EditorLogError> {
        if entry_capacity == 0 {
            return Err(EditorLogError::InvalidEntryCapacity);
        }
        if retained_bytes == 0 {
            return Err(EditorLogError::InvalidByteCapacity);
        }
        Ok(Self {
            entry_capacity,
            retained_bytes,
            event_queue_entry_capacity: entry_capacity.min(DEFAULT_EVENT_QUEUE_ENTRY_CAPACITY),
            event_queue_retained_bytes: retained_bytes.min(DEFAULT_EVENT_QUEUE_RETAINED_BYTES),
        })
    }

    pub fn with_event_queue_limits(
        mut self,
        entry_capacity: usize,
        retained_bytes: usize,
    ) -> Result<Self, EditorLogError> {
        if entry_capacity == 0 {
            return Err(EditorLogError::InvalidEventQueueEntryCapacity);
        }
        if retained_bytes == 0 {
            return Err(EditorLogError::InvalidEventQueueByteCapacity);
        }
        if entry_capacity > self.entry_capacity {
            return Err(EditorLogError::EventQueueEntryCapacityExceedsStore {
                maximum: self.entry_capacity,
                actual: entry_capacity,
            });
        }
        if retained_bytes > self.retained_bytes {
            return Err(EditorLogError::EventQueueByteCapacityExceedsStore {
                maximum: self.retained_bytes,
                actual: retained_bytes,
            });
        }
        self.event_queue_entry_capacity = entry_capacity;
        self.event_queue_retained_bytes = retained_bytes;
        Ok(self)
    }

    pub const fn entry_capacity(self) -> usize {
        self.entry_capacity
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    pub const fn event_queue_entry_capacity(self) -> usize {
        self.event_queue_entry_capacity
    }

    pub const fn event_queue_retained_bytes(self) -> usize {
        self.event_queue_retained_bytes
    }
}
