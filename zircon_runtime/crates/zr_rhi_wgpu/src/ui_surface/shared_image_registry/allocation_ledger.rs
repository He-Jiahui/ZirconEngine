use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in super::super) struct WgpuUiImageAllocationStats {
    pub(in super::super) allocation_count: u64,
    pub(in super::super) unique_allocation_bytes: u64,
    pub(in super::super) registry_evicted_pinned_bytes: u64,
    pub(in super::super) surface_pin_count: u64,
    pub(in super::super) in_flight_present_pin_count: u64,
    pub(in super::super) eviction_completion_count: u64,
}

#[derive(Default)]
pub(super) struct WgpuUiImageAllocationLedger {
    state: Mutex<WgpuUiImageAllocationLedgerState>,
    in_flight_present_pin_count: AtomicU64,
}

#[derive(Default)]
struct WgpuUiImageAllocationLedgerState {
    allocation_count: u64,
    unique_allocation_bytes: u64,
    registry_evicted_pinned_bytes: u64,
    surface_pin_count: u64,
    eviction_completion_count: u64,
}

struct WgpuUiImageAllocationRecord {
    ledger: Arc<WgpuUiImageAllocationLedger>,
    texture: Option<wgpu::Texture>,
    byte_size: u64,
    counted_as_evicted_pinned: AtomicBool,
}

pub(super) struct WgpuUiImageRegistryPin {
    allocation: Arc<WgpuUiImageAllocationRecord>,
}

pub(in super::super) struct WgpuUiImageSurfacePin {
    allocation: Arc<WgpuUiImageAllocationRecord>,
}

#[derive(Clone, Default)]
pub(in super::super) struct WgpuUiImageAllocationSet {
    inner: Option<Arc<WgpuUiImageAllocationSetInner>>,
}

struct WgpuUiImageAllocationSetInner {
    ledger: Arc<WgpuUiImageAllocationLedger>,
    _surface_pins: Vec<WgpuUiImageSurfacePin>,
}

pub(crate) struct WgpuUiImageInFlightPins {
    allocation_set: WgpuUiImageAllocationSet,
}

impl WgpuUiImageAllocationLedger {
    pub(super) fn try_allocate(
        self: &Arc<Self>,
        byte_size: u64,
        max_unique_allocation_bytes: u64,
        create_texture: impl FnOnce() -> wgpu::Texture,
    ) -> Option<WgpuUiImageRegistryPin> {
        let mut state = self.lock_state();
        let unique_allocation_bytes = state.unique_allocation_bytes.checked_add(byte_size)?;
        if unique_allocation_bytes > max_unique_allocation_bytes {
            return None;
        }
        state.allocation_count = state.allocation_count.saturating_add(1);
        state.unique_allocation_bytes = unique_allocation_bytes;
        drop(state);

        Some(WgpuUiImageRegistryPin {
            allocation: Arc::new(WgpuUiImageAllocationRecord {
                ledger: Arc::clone(self),
                texture: Some(create_texture()),
                byte_size,
                counted_as_evicted_pinned: AtomicBool::new(false),
            }),
        })
    }

    #[cfg(test)]
    fn try_allocate_for_test(
        self: &Arc<Self>,
        byte_size: u64,
        max_unique_allocation_bytes: u64,
    ) -> Option<WgpuUiImageRegistryPin> {
        let mut state = self.lock_state();
        let unique_allocation_bytes = state.unique_allocation_bytes.checked_add(byte_size)?;
        if unique_allocation_bytes > max_unique_allocation_bytes {
            return None;
        }
        state.allocation_count = state.allocation_count.saturating_add(1);
        state.unique_allocation_bytes = unique_allocation_bytes;
        drop(state);

        Some(WgpuUiImageRegistryPin {
            allocation: Arc::new(WgpuUiImageAllocationRecord {
                ledger: Arc::clone(self),
                texture: None,
                byte_size,
                counted_as_evicted_pinned: AtomicBool::new(false),
            }),
        })
    }

    pub(super) fn unique_allocation_bytes(&self) -> u64 {
        self.lock_state().unique_allocation_bytes
    }

    pub(super) fn stats(&self) -> WgpuUiImageAllocationStats {
        let state = self.lock_state();
        WgpuUiImageAllocationStats {
            allocation_count: state.allocation_count,
            unique_allocation_bytes: state.unique_allocation_bytes,
            registry_evicted_pinned_bytes: state.registry_evicted_pinned_bytes,
            surface_pin_count: state.surface_pin_count,
            in_flight_present_pin_count: self.in_flight_present_pin_count.load(Ordering::Relaxed),
            eviction_completion_count: state.eviction_completion_count,
        }
    }

    fn add_surface_pin(&self) {
        let mut state = self.lock_state();
        state.surface_pin_count = state.surface_pin_count.saturating_add(1);
    }

    fn remove_surface_pin(&self) {
        let mut state = self.lock_state();
        state.surface_pin_count = state.surface_pin_count.saturating_sub(1);
    }

    fn mark_registry_evicted(&self, byte_size: u64, remains_pinned: bool) {
        if remains_pinned {
            let mut state = self.lock_state();
            state.registry_evicted_pinned_bytes = state
                .registry_evicted_pinned_bytes
                .saturating_add(byte_size);
        }
    }

    fn remove_allocation(&self, byte_size: u64, was_evicted_pinned: bool) {
        let mut state = self.lock_state();
        state.allocation_count = state.allocation_count.saturating_sub(1);
        state.unique_allocation_bytes = state.unique_allocation_bytes.saturating_sub(byte_size);
        if was_evicted_pinned {
            state.registry_evicted_pinned_bytes = state
                .registry_evicted_pinned_bytes
                .saturating_sub(byte_size);
        }
        state.eviction_completion_count = state.eviction_completion_count.saturating_add(1);
    }

    fn lock_state(&self) -> MutexGuard<'_, WgpuUiImageAllocationLedgerState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl WgpuUiImageRegistryPin {
    pub(super) fn texture(&self) -> &wgpu::Texture {
        self.allocation
            .texture
            .as_ref()
            .expect("production UI allocations always own a texture")
    }

    pub(super) fn surface_pin(&self) -> WgpuUiImageSurfacePin {
        self.allocation.ledger.add_surface_pin();
        WgpuUiImageSurfacePin {
            allocation: Arc::clone(&self.allocation),
        }
    }

    pub(super) fn is_exclusively_registry_owned(&self) -> bool {
        Arc::strong_count(&self.allocation) == 1
    }
}

impl Drop for WgpuUiImageRegistryPin {
    fn drop(&mut self) {
        let remains_pinned = Arc::strong_count(&self.allocation) > 1;
        self.allocation
            .counted_as_evicted_pinned
            .store(remains_pinned, Ordering::Release);
        self.allocation
            .ledger
            .mark_registry_evicted(self.allocation.byte_size, remains_pinned);
    }
}

impl Clone for WgpuUiImageSurfacePin {
    fn clone(&self) -> Self {
        self.allocation.ledger.add_surface_pin();
        Self {
            allocation: Arc::clone(&self.allocation),
        }
    }
}

impl Drop for WgpuUiImageSurfacePin {
    fn drop(&mut self) {
        self.allocation.ledger.remove_surface_pin();
    }
}

impl Drop for WgpuUiImageAllocationRecord {
    fn drop(&mut self) {
        drop(self.texture.take());
        self.ledger.remove_allocation(
            self.byte_size,
            self.counted_as_evicted_pinned.load(Ordering::Acquire),
        );
    }
}

impl WgpuUiImageAllocationSet {
    pub(in super::super) fn from_surface_pins(surface_pins: Vec<WgpuUiImageSurfacePin>) -> Self {
        let Some(first) = surface_pins.first() else {
            return Self::default();
        };
        let ledger = Arc::clone(&first.allocation.ledger);
        debug_assert!(surface_pins
            .iter()
            .all(|pin| Arc::ptr_eq(&pin.allocation.ledger, &ledger)));
        Self {
            inner: Some(Arc::new(WgpuUiImageAllocationSetInner {
                ledger,
                _surface_pins: surface_pins,
            })),
        }
    }

    pub(in super::super) fn begin_in_flight(&self) -> Option<WgpuUiImageInFlightPins> {
        let inner = self.inner.as_ref()?;
        inner
            .ledger
            .in_flight_present_pin_count
            .fetch_add(1, Ordering::Relaxed);
        Some(WgpuUiImageInFlightPins {
            allocation_set: self.clone(),
        })
    }
}

impl Drop for WgpuUiImageInFlightPins {
    fn drop(&mut self) {
        if let Some(inner) = self.allocation_set.inner.as_ref() {
            inner
                .ledger
                .in_flight_present_pin_count
                .fetch_sub(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{WgpuUiImageAllocationLedger, WgpuUiImageAllocationSet};

    #[test]
    fn registry_eviction_remains_budgeted_until_the_surface_pin_drops() {
        let ledger = Arc::new(WgpuUiImageAllocationLedger::default());
        let registry_pin = ledger
            .try_allocate_for_test(64, 64)
            .expect("allocation fits");
        let surface_pin = registry_pin.surface_pin();

        drop(registry_pin);
        let pinned = ledger.stats();
        assert_eq!(pinned.unique_allocation_bytes, 64);
        assert_eq!(pinned.registry_evicted_pinned_bytes, 64);
        assert!(ledger.try_allocate_for_test(1, 64).is_none());

        drop(surface_pin);
        let released = ledger.stats();
        assert_eq!(released.unique_allocation_bytes, 0);
        assert_eq!(released.registry_evicted_pinned_bytes, 0);
        assert_eq!(released.eviction_completion_count, 1);
    }

    #[test]
    fn in_flight_set_releases_allocations_only_after_completion_guard_drops() {
        let ledger = Arc::new(WgpuUiImageAllocationLedger::default());
        let registry_pin = ledger
            .try_allocate_for_test(64, 64)
            .expect("allocation fits");
        let surface_pin = registry_pin.surface_pin();
        let allocation_set = WgpuUiImageAllocationSet::from_surface_pins(vec![surface_pin]);
        let in_flight = allocation_set.begin_in_flight().expect("non-empty set");

        drop(registry_pin);
        drop(allocation_set);
        assert_eq!(ledger.stats().in_flight_present_pin_count, 1);
        assert_eq!(ledger.stats().unique_allocation_bytes, 64);

        drop(in_flight);
        assert_eq!(ledger.stats().in_flight_present_pin_count, 0);
        assert_eq!(ledger.stats().unique_allocation_bytes, 0);
    }

    #[test]
    fn only_registry_owned_allocations_are_immediately_releasable() {
        let ledger = Arc::new(WgpuUiImageAllocationLedger::default());
        let registry_pin = ledger
            .try_allocate_for_test(64, 64)
            .expect("allocation fits");
        assert!(registry_pin.is_exclusively_registry_owned());

        let surface_pin = registry_pin.surface_pin();
        assert!(!registry_pin.is_exclusively_registry_owned());
        drop(surface_pin);
        assert!(registry_pin.is_exclusively_registry_owned());
    }
}
