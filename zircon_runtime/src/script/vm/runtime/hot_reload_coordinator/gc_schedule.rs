use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::super::super::handles::PluginSlotId;

#[derive(Clone, Copy, Debug)]
struct GcScheduleEntry {
    interval_frames: u64,
    next_due_frame: u64,
}

#[derive(Debug, Default)]
pub(super) struct GcNextDueSchedule {
    entries: HashMap<PluginSlotId, GcScheduleEntry>,
    due_by_frame: BTreeMap<u64, BTreeSet<PluginSlotId>>,
}

impl GcNextDueSchedule {
    pub(super) fn replace(
        &mut self,
        slot: PluginSlotId,
        interval_frames: Option<u64>,
        next_frame: u64,
    ) {
        self.remove(slot);
        let Some(interval_frames) = interval_frames.filter(|interval| *interval > 0) else {
            return;
        };
        let Some(next_due_frame) = due_frame_at_or_after(next_frame, interval_frames) else {
            return;
        };
        self.entries.insert(
            slot,
            GcScheduleEntry {
                interval_frames,
                next_due_frame,
            },
        );
        self.due_by_frame
            .entry(next_due_frame)
            .or_default()
            .insert(slot);
    }

    pub(super) fn remove(&mut self, slot: PluginSlotId) -> bool {
        let Some(entry) = self.entries.remove(&slot) else {
            return false;
        };
        let remove_frame = if let Some(slots) = self.due_by_frame.get_mut(&entry.next_due_frame) {
            slots.remove(&slot);
            slots.is_empty()
        } else {
            false
        };
        if remove_frame {
            self.due_by_frame.remove(&entry.next_due_frame);
        }
        true
    }

    pub(super) fn take_due(&mut self, frame_index: u64) -> Vec<PluginSlotId> {
        let ready_capacity = self
            .due_by_frame
            .range(..=frame_index)
            .map(|(_, slots)| slots.len())
            .sum();
        let mut ready = Vec::with_capacity(ready_capacity);
        while self
            .due_by_frame
            .first_key_value()
            .is_some_and(|(due_frame, _)| *due_frame <= frame_index)
        {
            let (due_frame, slots) = self
                .due_by_frame
                .pop_first()
                .expect("a due GC frame was observed above");
            for slot in slots {
                let next_due_frame = {
                    let Some(entry) = self.entries.get_mut(&slot) else {
                        continue;
                    };
                    if entry.next_due_frame != due_frame {
                        continue;
                    }
                    ready.push(slot);
                    let next_due_frame = due_frame_after(frame_index, entry.interval_frames);
                    if let Some(next_due_frame) = next_due_frame {
                        entry.next_due_frame = next_due_frame;
                    }
                    next_due_frame
                };
                match next_due_frame {
                    Some(next_due_frame) => {
                        self.due_by_frame
                            .entry(next_due_frame)
                            .or_default()
                            .insert(slot);
                    }
                    None => {
                        self.entries.remove(&slot);
                    }
                }
            }
        }
        ready.sort_unstable();
        ready
    }
}

fn due_frame_at_or_after(frame: u64, interval: u64) -> Option<u64> {
    let remainder = frame % interval;
    if remainder == 0 {
        Some(frame)
    } else {
        frame.checked_add(interval - remainder)
    }
}

fn due_frame_after(frame: u64, interval: u64) -> Option<u64> {
    let remainder = frame % interval;
    frame.checked_add(if remainder == 0 {
        interval
    } else {
        interval - remainder
    })
}
