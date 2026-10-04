use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Owns one stable allocation while shared registry lookups borrow only pointer metadata.
///
/// The pointer is created once and never replaced. `PhantomData<Box<T>>` preserves the payload's
/// ownership and auto-trait bounds; this owner has no custom Send/Sync implementation.
pub(super) struct OwnedChannel<T> {
    pointer: AtomicPtr<T>,
    ownership: PhantomData<Box<T>>,
}

impl<T> OwnedChannel<T> {
    pub(super) fn new(value: T) -> Self {
        Self {
            pointer: AtomicPtr::new(Box::into_raw(Box::new(value))),
            ownership: PhantomData,
        }
    }

    pub(super) fn get(&self) -> &T {
        // Safe callers borrow the actual shared owner; raw grants separately enforce no writer.
        unsafe { &*self.as_ptr() }
    }

    pub(super) fn get_mut(&mut self) -> &mut T {
        // Safe callers hold the actual exclusive owner, excluding every active raw grant.
        let pointer = *self.pointer.get_mut();
        unsafe { &mut *pointer }
    }

    pub(super) fn as_ptr(&self) -> *mut T {
        // This is pointer metadata, not publication of a new payload or synchronization of T.
        self.pointer.load(Ordering::Relaxed)
    }
}

impl<T> Drop for OwnedChannel<T> {
    fn drop(&mut self) {
        // No Clone/Copy or pointer replacement exists. This is the unique Box ownership return.
        let pointer = *self.pointer.get_mut();
        unsafe { drop(Box::from_raw(pointer)) };
    }
}
