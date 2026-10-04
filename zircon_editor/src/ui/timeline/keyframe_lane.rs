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
    let mut visible = Vec::new();
    for key in keys {
        if !range.contains(key.time) {
            continue;
        }
        if visible.is_empty() {
            visible.reserve(keys.len());
        }
        visible.push(key);
    }
    visible
}

#[cfg(test)]
#[path = "tests/keyframe_lane.rs"]
mod tests;
