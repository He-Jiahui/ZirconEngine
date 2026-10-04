use super::super::{free_owned_bytes_v3, NativePluginOwnedByteBufferV3};
use super::error::{NativePluginOwnedBytesError, NativePluginOwnedBytesErrorKind};
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::{Mutex, MutexGuard, OnceLock};

pub(super) struct Registry {
    pub(super) next_id: Option<NonZeroU64>,
    allocations: HashMap<u64, Vec<u8>>,
}

impl Registry {
    pub(super) fn new() -> Self {
        Self {
            next_id: NonZeroU64::new(1),
            allocations: HashMap::new(),
        }
    }

    pub(super) fn register(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<NativePluginOwnedByteBufferV3, NativePluginOwnedBytesError> {
        self.register_with_reservation(bytes, 1)
    }

    pub(super) fn register_with_reservation(
        &mut self,
        mut bytes: Vec<u8>,
        additional: usize,
    ) -> Result<NativePluginOwnedByteBufferV3, NativePluginOwnedBytesError> {
        // A zero-capacity Vec owns no allocation. Retained-capacity empty Vecs do.
        if bytes.capacity() == 0 {
            return Ok(NativePluginOwnedByteBufferV3::empty());
        }
        let Some(id) = self.next_id else {
            return Err(NativePluginOwnedBytesError {
                kind: NativePluginOwnedBytesErrorKind::AllocationIdsExhausted,
                bytes,
            });
        };
        if self.allocations.try_reserve(additional).is_err() {
            return Err(NativePluginOwnedBytesError {
                kind: NativePluginOwnedBytesErrorKind::RegistryCapacity,
                bytes,
            });
        }
        let buffer = NativePluginOwnedByteBufferV3 {
            data: bytes.as_mut_ptr(),
            len: bytes.len(),
            capacity: bytes.capacity(),
            owner_token: id.get(),
            free: Some(free_owned_bytes_v3),
        };
        // Reserve the ID permanently before publication, including u64::MAX.
        self.next_id = id.get().checked_add(1).and_then(NonZeroU64::new);
        // u64 keys have no user code; reserve succeeded and IDs are never reused.
        self.allocations.insert(id.get(), bytes);
        Ok(buffer)
    }

    pub(super) fn consume(&mut self, buffer: NativePluginOwnedByteBufferV3) -> Option<Vec<u8>> {
        let bytes = self.allocations.get(&buffer.owner_token)?;
        if bytes.as_ptr() as usize != buffer.data as usize
            || bytes.len() != buffer.len
            || bytes.capacity() != buffer.capacity
        {
            return None;
        }
        // Validation and removal are one mutex operation. Never reconstruct a
        // Vec from foreign metadata and never dereference the foreign pointer.
        self.allocations.remove(&buffer.owner_token)
    }
}

pub(super) static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

pub(super) fn lock_registry(registry: &Mutex<Registry>) -> MutexGuard<'_, Registry> {
    match registry.lock() {
        Ok(guard) => guard,
        // Operations under this lock contain no foreign calls or byte drops.
        // ID reservation precedes insertion; poison cannot recycle an issued ID.
        // Retain authoritative Vecs and recover rather than permanently leaking
        // every live allocation after an unrelated unwinding thread.
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Registers bytes in this SDK image. Failure returns the unpublished Vec.
///
/// A zero-capacity empty Vec becomes empty(); a retained-capacity empty Vec
/// remains a real allocation and must be released exactly once. Allocation IDs
/// are never reused while this SDK image remains loaded. No buffer or callback
/// may outlive that image. The host's normal callback/image lease is still required.
pub(in super::super) fn register(
    bytes: Vec<u8>,
) -> Result<NativePluginOwnedByteBufferV3, NativePluginOwnedBytesError> {
    let registry = REGISTRY.get_or_init(|| Mutex::new(Registry::new()));
    lock_registry(registry).register(bytes)
}
