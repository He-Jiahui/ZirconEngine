pub(super) const READBACK_FRAME_SLOTS: usize = 3;
pub(super) const READBACK_OFFSET_ALIGNMENT: u64 = 256;
pub(super) const MIN_STAGING_CAPACITY: u64 = 256 * 1024;
pub(super) const LOW_UTILIZATION_FRAME_LIMIT: u16 = 240;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StagingCapacityPolicy {
    capacity: u64,
    low_utilization_frames: u16,
}

impl Default for StagingCapacityPolicy {
    fn default() -> Self {
        Self {
            capacity: 0,
            low_utilization_frames: 0,
        }
    }
}

impl StagingCapacityPolicy {
    pub(super) const fn capacity(self) -> u64 {
        self.capacity
    }

    #[cfg(test)]
    pub(super) fn capacity_for_frame(&mut self, used_bytes: u64) -> Option<u64> {
        self.capacity_for_elapsed_frames(used_bytes, 1)
    }

    pub(super) fn capacity_for_elapsed_frames(
        &mut self,
        used_bytes: u64,
        elapsed_frames: u16,
    ) -> Option<u64> {
        let elapsed_frames = elapsed_frames.max(1);
        if used_bytes == 0 {
            if self.capacity == 0 {
                return None;
            }
            self.low_utilization_frames =
                self.low_utilization_frames.saturating_add(elapsed_frames);
            if self.low_utilization_frames < LOW_UTILIZATION_FRAME_LIMIT
                || self.capacity <= MIN_STAGING_CAPACITY
            {
                return None;
            }
            self.capacity = (self.capacity / 2).max(MIN_STAGING_CAPACITY);
            self.low_utilization_frames = 0;
            return Some(self.capacity);
        }

        let required = used_bytes.max(MIN_STAGING_CAPACITY);
        if required > self.capacity {
            self.capacity = required.checked_next_power_of_two()?;
            self.low_utilization_frames = 0;
            return Some(self.capacity);
        }

        if used_bytes < self.capacity / 4 {
            self.low_utilization_frames =
                self.low_utilization_frames.saturating_add(elapsed_frames);
        } else {
            self.low_utilization_frames = 0;
        }

        if self.low_utilization_frames >= LOW_UTILIZATION_FRAME_LIMIT
            && self.capacity > MIN_STAGING_CAPACITY
        {
            let required_capacity = required.checked_next_power_of_two()?;
            self.capacity = (self.capacity / 2)
                .max(MIN_STAGING_CAPACITY)
                .max(required_capacity);
            self.low_utilization_frames = 0;
            return Some(self.capacity);
        }

        None
    }
}

pub(super) fn align_readback_offset(value: u64) -> Option<u64> {
    value
        .checked_add(READBACK_OFFSET_ALIGNMENT - 1)
        .map(|value| value & !(READBACK_OFFSET_ALIGNMENT - 1))
}
