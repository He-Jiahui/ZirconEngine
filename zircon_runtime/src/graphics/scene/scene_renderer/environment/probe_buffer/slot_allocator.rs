use std::collections::HashMap;

use crate::core::resource::ResourceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ProbeCubemapSlot {
    pub(super) slot: u32,
    pub(super) revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ProbeCubemapSlotAllocation {
    pub(super) slot: u32,
    pub(super) requires_upload: bool,
    pub(super) evicted: Option<ResourceId>,
}

#[derive(Clone, Copy, Debug)]
struct ProbeCubemapSlotEntry {
    slot: u32,
    revision: u64,
    last_used: u64,
}

pub(super) struct ProbeCubemapSlotAllocator {
    capacity: usize,
    clock: u64,
    entries: HashMap<ResourceId, ProbeCubemapSlotEntry>,
    slot_owners: Vec<Option<ResourceId>>,
}

impl ProbeCubemapSlotAllocator {
    pub(super) fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1).next_power_of_two();
        Self {
            capacity,
            clock: 0,
            entries: HashMap::with_capacity(capacity),
            slot_owners: vec![None; capacity],
        }
    }

    pub(super) const fn capacity(&self) -> usize {
        self.capacity
    }

    pub(super) fn get(&self, cubemap: ResourceId) -> Option<ProbeCubemapSlot> {
        self.entries.get(&cubemap).map(|entry| ProbeCubemapSlot {
            slot: entry.slot,
            revision: entry.revision,
        })
    }

    pub(super) fn acquire(
        &mut self,
        cubemap: ResourceId,
        revision: u64,
    ) -> ProbeCubemapSlotAllocation {
        self.clock = self.clock.wrapping_add(1).max(1);
        if let Some(entry) = self.entries.get_mut(&cubemap) {
            let requires_upload = entry.revision != revision;
            entry.revision = revision;
            entry.last_used = self.clock;
            return ProbeCubemapSlotAllocation {
                slot: entry.slot,
                requires_upload,
                evicted: None,
            };
        }

        let (slot, evicted) = match self.slot_owners.iter().position(Option::is_none) {
            Some(slot) => (slot as u32, None),
            None => {
                let lru = self
                    .entries
                    .iter()
                    .min_by_key(|(_, entry)| (entry.last_used, entry.slot))
                    .map(|(id, entry)| (*id, entry.slot));
                match lru {
                    Some((evicted_id, slot)) => {
                        self.entries.remove(&evicted_id);
                        (slot, Some(evicted_id))
                    }
                    None => {
                        debug_assert!(false, "full probe slot allocator lost its ownership map");
                        self.slot_owners.fill(None);
                        (0, None)
                    }
                }
            }
        };

        self.slot_owners[slot as usize] = Some(cubemap);
        self.entries.insert(
            cubemap,
            ProbeCubemapSlotEntry {
                slot,
                revision,
                last_used: self.clock,
            },
        );
        ProbeCubemapSlotAllocation {
            slot,
            requires_upload: true,
            evicted,
        }
    }
}
