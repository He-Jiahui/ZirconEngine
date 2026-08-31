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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct ProbeCubemapSlotReservation {
    pub(super) cubemap: ResourceId,
    pub(super) revision: u64,
    pub(super) slot: u32,
    pub(super) prepare_epoch: u64,
    replaces_existing: bool,
}

impl ProbeCubemapSlotReservation {
    pub(in crate::graphics) const fn cubemap(self) -> ResourceId {
        self.cubemap
    }

    pub(in crate::graphics) const fn revision(self) -> u64 {
        self.revision
    }

    pub(in crate::graphics) const fn slot(self) -> u32 {
        self.slot
    }

    pub(in crate::graphics) const fn prepare_epoch(self) -> u64 {
        self.prepare_epoch
    }
}

#[derive(Clone, Copy, Debug)]
struct ProbeCubemapSlotEntry {
    slot: u32,
    revision: u64,
    upload_state: ProbeCubemapSlotUploadState,
    previous: Option<ResourceId>,
    next: Option<ResourceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeCubemapSlotUploadState {
    Ready,
    Pending(u64),
}

pub(super) struct ProbeCubemapSlotAllocator {
    capacity: usize,
    entries: HashMap<ResourceId, ProbeCubemapSlotEntry>,
    free_slots: Vec<u32>,
    capture_reservation: Option<ProbeCubemapSlotReservation>,
    oldest: Option<ResourceId>,
    newest: Option<ResourceId>,
}

impl ProbeCubemapSlotAllocator {
    pub(super) fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1).next_power_of_two();
        let physical_slot_count = capacity
            .checked_add(1)
            .expect("probe cubemap slot capacity must leave room for capture");
        Self {
            capacity,
            entries: HashMap::with_capacity(capacity),
            free_slots: (0..physical_slot_count as u32).rev().collect(),
            capture_reservation: None,
            oldest: None,
            newest: None,
        }
    }

    pub(super) const fn capacity(&self) -> usize {
        self.capacity
    }

    #[cfg(test)]
    pub(super) const fn physical_slot_count(&self) -> usize {
        self.capacity + 1
    }

    pub(super) fn available(
        &self,
        cubemap: ResourceId,
        revision: u64,
        prepare_epoch: u64,
    ) -> Option<ProbeCubemapSlot> {
        self.entries.get(&cubemap).and_then(|entry| {
            let upload_available = match entry.upload_state {
                ProbeCubemapSlotUploadState::Ready => true,
                ProbeCubemapSlotUploadState::Pending(epoch) => epoch == prepare_epoch,
            };
            (entry.revision == revision && upload_available).then_some(ProbeCubemapSlot {
                slot: entry.slot,
                revision: entry.revision,
            })
        })
    }

    pub(super) fn acquire(
        &mut self,
        cubemap: ResourceId,
        revision: u64,
        prepare_epoch: u64,
    ) -> Option<ProbeCubemapSlotAllocation> {
        if let Some(entry) = self.entries.get_mut(&cubemap) {
            if self
                .capture_reservation
                .is_some_and(|reservation| reservation.cubemap == cubemap)
            {
                if !matches!(entry.upload_state, ProbeCubemapSlotUploadState::Ready) {
                    return None;
                }
                let entry = *entry;
                self.touch(cubemap, entry);
                return Some(ProbeCubemapSlotAllocation {
                    slot: entry.slot,
                    requires_upload: false,
                    evicted: None,
                });
            }
            let upload_available = match entry.upload_state {
                ProbeCubemapSlotUploadState::Ready => true,
                ProbeCubemapSlotUploadState::Pending(epoch) => epoch == prepare_epoch,
            };
            let requires_upload = entry.revision != revision || !upload_available;
            entry.revision = revision;
            if requires_upload {
                entry.upload_state = ProbeCubemapSlotUploadState::Pending(prepare_epoch);
            }
            let entry = *entry;
            self.touch(cubemap, entry);
            return Some(ProbeCubemapSlotAllocation {
                slot: entry.slot,
                requires_upload,
                evicted: None,
            });
        }
        if self
            .capture_reservation
            .is_some_and(|reservation| reservation.cubemap == cubemap)
        {
            return None;
        }

        let reserved_new_entry_count = usize::from(
            self.capture_reservation
                .is_some_and(|reservation| !reservation.replaces_existing),
        );
        let (slot, evicted) = if self.entries.len() + reserved_new_entry_count < self.capacity {
            (
                self.free_slots
                    .pop()
                    .expect("probe allocator must retain its physical capture spare"),
                None,
            )
        } else {
            let (evicted, slot) = self.evict_oldest()?;
            (slot, Some(evicted))
        };

        self.insert_newest(cubemap, slot, revision, prepare_epoch);
        Some(ProbeCubemapSlotAllocation {
            slot,
            requires_upload: true,
            evicted,
        })
    }

    pub(super) fn reserve_for_capture(
        &mut self,
        cubemap: ResourceId,
        revision: u64,
        prepare_epoch: u64,
    ) -> Option<ProbeCubemapSlotReservation> {
        if self.capture_reservation.is_some() {
            return None;
        }
        let replaces_existing = self.entries.contains_key(&cubemap);
        if !replaces_existing && self.entries.len() >= self.capacity {
            return None;
        }
        let reservation = ProbeCubemapSlotReservation {
            cubemap,
            revision,
            slot: self.free_slots.pop()?,
            prepare_epoch,
            replaces_existing,
        };
        self.capture_reservation = Some(reservation);
        Some(reservation)
    }

    pub(super) fn capture_pending(&self, cubemap: ResourceId) -> bool {
        self.capture_reservation
            .is_some_and(|reservation| reservation.cubemap == cubemap)
    }

    pub(super) fn commit(
        &mut self,
        cubemap: ResourceId,
        revision: u64,
        slot: u32,
        prepare_epoch: u64,
    ) {
        if self.capture_reservation.is_some_and(|reservation| {
            reservation.cubemap == cubemap
                && reservation.revision == revision
                && reservation.slot == slot
                && reservation.prepare_epoch == prepare_epoch
        }) {
            let reservation = self
                .capture_reservation
                .take()
                .expect("matched capture reservation must remain owned");
            if let Some(entry) = self.entries.get_mut(&cubemap) {
                let released_slot = entry.slot;
                entry.slot = reservation.slot;
                entry.revision = reservation.revision;
                entry.upload_state = ProbeCubemapSlotUploadState::Ready;
                let entry = *entry;
                self.free_slots.push(released_slot);
                self.touch(cubemap, entry);
            } else {
                debug_assert!(!reservation.replaces_existing);
                debug_assert!(self.entries.len() < self.capacity);
                self.insert_newest_ready(cubemap, reservation.slot, reservation.revision);
            }
            return;
        }

        let Some(entry) = self.entries.get_mut(&cubemap) else {
            return;
        };
        if entry.slot == slot
            && entry.revision == revision
            && matches!(
                entry.upload_state,
                ProbeCubemapSlotUploadState::Pending(epoch) if epoch == prepare_epoch
            )
        {
            entry.upload_state = ProbeCubemapSlotUploadState::Ready;
        }
    }

    pub(super) fn cancel(&mut self, reservation: ProbeCubemapSlotReservation) {
        if self.capture_reservation == Some(reservation) {
            self.capture_reservation = None;
            self.free_slots.push(reservation.slot);
        }
    }

    pub(super) fn invalidate_pending_epochs(&mut self) {
        for entry in self.entries.values_mut() {
            if matches!(entry.upload_state, ProbeCubemapSlotUploadState::Pending(_)) {
                entry.upload_state = ProbeCubemapSlotUploadState::Pending(0);
            }
        }
    }

    fn touch(&mut self, cubemap: ResourceId, entry: ProbeCubemapSlotEntry) {
        if self.newest == Some(cubemap) {
            return;
        }

        if let Some(previous) = entry.previous {
            self.entries
                .get_mut(&previous)
                .expect("probe LRU previous link must exist")
                .next = entry.next;
        } else {
            self.oldest = entry.next;
        }
        let next = entry
            .next
            .expect("a non-newest probe LRU entry must have a successor");
        self.entries
            .get_mut(&next)
            .expect("probe LRU next link must exist")
            .previous = entry.previous;

        let newest = self
            .newest
            .expect("a non-empty probe LRU must have a newest entry");
        self.entries
            .get_mut(&newest)
            .expect("probe LRU newest link must exist")
            .next = Some(cubemap);
        let entry = self
            .entries
            .get_mut(&cubemap)
            .expect("touched probe LRU entry must exist");
        entry.previous = Some(newest);
        entry.next = None;
        self.newest = Some(cubemap);
    }

    fn evict_oldest(&mut self) -> Option<(ResourceId, u32)> {
        let protected = self
            .capture_reservation
            .filter(|reservation| reservation.replaces_existing)
            .map(|reservation| reservation.cubemap);
        let mut cursor = self.oldest;
        let evicted = loop {
            let candidate = cursor?;
            let entry = self
                .entries
                .get(&candidate)
                .expect("probe LRU candidate must exist");
            if Some(candidate) != protected {
                break candidate;
            }
            cursor = entry.next;
        };
        let entry = self
            .entries
            .remove(&evicted)
            .expect("probe LRU entry must exist");
        if let Some(previous) = entry.previous {
            self.entries
                .get_mut(&previous)
                .expect("probe LRU previous link must exist")
                .next = entry.next;
        } else {
            self.oldest = entry.next;
        }
        if let Some(next) = entry.next {
            self.entries
                .get_mut(&next)
                .expect("probe LRU next link must exist")
                .previous = entry.previous;
        } else {
            self.newest = entry.previous;
        }
        Some((evicted, entry.slot))
    }

    fn insert_newest(&mut self, cubemap: ResourceId, slot: u32, revision: u64, prepare_epoch: u64) {
        let previous = self.newest;
        if let Some(previous) = previous {
            self.entries
                .get_mut(&previous)
                .expect("probe LRU newest link must exist")
                .next = Some(cubemap);
        } else {
            self.oldest = Some(cubemap);
        }
        self.entries.insert(
            cubemap,
            ProbeCubemapSlotEntry {
                slot,
                revision,
                upload_state: ProbeCubemapSlotUploadState::Pending(prepare_epoch),
                previous,
                next: None,
            },
        );
        self.newest = Some(cubemap);
    }

    fn insert_newest_ready(&mut self, cubemap: ResourceId, slot: u32, revision: u64) {
        let previous = self.newest;
        if let Some(previous) = previous {
            self.entries
                .get_mut(&previous)
                .expect("probe LRU newest link must exist")
                .next = Some(cubemap);
        } else {
            self.oldest = Some(cubemap);
        }
        self.entries.insert(
            cubemap,
            ProbeCubemapSlotEntry {
                slot,
                revision,
                upload_state: ProbeCubemapSlotUploadState::Ready,
                previous,
                next: None,
            },
        );
        self.newest = Some(cubemap);
    }
}
