use super::{TimelineKey, TimelineRange};

/// Renderer-neutral keyframe lane data. It borrows the domain's keys rather than copying an
/// animation track into UI-owned state.
#[derive(Clone, Copy, Debug)]
pub struct TimelineKeyframeLane<'a> {
    keys: &'a [TimelineKey],
}

impl<'a> TimelineKeyframeLane<'a> {
    pub fn new(keys: &'a [TimelineKey]) -> Self {
        Self { keys }
    }

    pub fn visible_keys(&self, range: TimelineRange) -> Vec<&'a TimelineKey> {
        keyframes_in_range(self.keys, range)
    }
}

pub fn keyframes_in_range(keys: &[TimelineKey], range: TimelineRange) -> Vec<&TimelineKey> {
    keys.iter().filter(|key| range.contains(key.time)).collect()
}
