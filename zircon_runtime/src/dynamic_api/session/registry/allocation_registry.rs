use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use zircon_runtime_interface::{
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeSessionHandle, ZrStatus,
};

use super::session_store::{begin_session_action, begin_session_release_action};
use crate::dynamic_api::session::status::{invalid_argument, not_found};

static RUNTIME_ALLOCATIONS: OnceLock<Mutex<RuntimeAllocationRegistry>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::dynamic_api::session) enum RuntimeAllocationKind {
    Frame,
    Accessibility,
    Profile,
    HostRequests,
    WorldSync,
    PluginEvents,
    Operation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RuntimeAllocationCensus {
    pub(super) outstanding_allocations: u64,
    pub(super) outstanding_bytes: u64,
    pub(super) high_water_allocations: u64,
    pub(super) high_water_bytes: u64,
}

struct RuntimeAllocationRecord {
    session: ZrRuntimeSessionHandle,
    _kind: RuntimeAllocationKind,
    bytes: Box<[u8]>,
}

struct RuntimeAllocationRegistry {
    next_id: u64,
    allocations: HashMap<u64, RuntimeAllocationRecord>,
    census: HashMap<u64, RuntimeAllocationCensus>,
}

impl Default for RuntimeAllocationRegistry {
    fn default() -> Self {
        Self {
            next_id: 1,
            allocations: HashMap::new(),
            census: HashMap::new(),
        }
    }
}

fn registry() -> &'static Mutex<RuntimeAllocationRegistry> {
    RUNTIME_ALLOCATIONS.get_or_init(|| Mutex::new(RuntimeAllocationRegistry::default()))
}

fn lock_registry() -> MutexGuard<'static, RuntimeAllocationRegistry> {
    registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(in crate::dynamic_api::session) fn register_runtime_allocation(
    session: ZrRuntimeSessionHandle,
    kind: RuntimeAllocationKind,
    bytes: Vec<u8>,
) -> Result<ZrOwnedResultV2, ZrStatus> {
    let _action = begin_session_action(session)?;
    register_runtime_allocation_in_action(session, kind, bytes)
}

// The caller must hold the session action lease through registration and ABI output publication.
pub(in crate::dynamic_api::session) fn register_runtime_allocation_in_action(
    session: ZrRuntimeSessionHandle,
    kind: RuntimeAllocationKind,
    bytes: Vec<u8>,
) -> Result<ZrOwnedResultV2, ZrStatus> {
    if bytes.is_empty() {
        return Ok(ZrOwnedResultV2::empty());
    }
    let len = u64::try_from(bytes.len())
        .map_err(|_| invalid_argument(b"runtime allocation exceeds ABI length range"))?;
    let bytes = bytes.into_boxed_slice();
    let data = bytes.as_ptr();

    let mut registry = lock_registry();
    if registry.next_id == u64::MAX {
        return Err(invalid_argument(b"runtime allocation ID space exhausted"));
    }
    let raw_id = registry.next_id;
    registry.next_id += 1;

    let previous = registry
        .census
        .get(&session.raw())
        .copied()
        .unwrap_or_default();
    let outstanding_allocations = previous
        .outstanding_allocations
        .checked_add(1)
        .ok_or_else(|| invalid_argument(b"runtime allocation census overflow"))?;
    let outstanding_bytes = previous
        .outstanding_bytes
        .checked_add(len)
        .ok_or_else(|| invalid_argument(b"runtime allocation byte census overflow"))?;
    registry.census.insert(
        session.raw(),
        RuntimeAllocationCensus {
            outstanding_allocations,
            outstanding_bytes,
            high_water_allocations: previous.high_water_allocations.max(outstanding_allocations),
            high_water_bytes: previous.high_water_bytes.max(outstanding_bytes),
        },
    );
    registry.allocations.insert(
        raw_id,
        RuntimeAllocationRecord {
            session,
            _kind: kind,
            bytes,
        },
    );

    Ok(ZrOwnedResultV2 {
        data,
        len,
        allocation: ZrRuntimeAllocationId::new(raw_id),
    })
}

pub(in crate::dynamic_api::session) fn release_runtime_allocation(
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    if !allocation.is_valid() {
        return not_found(b"runtime allocation not found");
    }
    let _action = match begin_session_release_action(session) {
        Ok(action) => action,
        Err(status) => return status,
    };
    let record = {
        let mut registry = lock_registry();
        let Some(record) = registry.allocations.get(&allocation.raw()) else {
            return not_found(b"runtime allocation not found");
        };
        if record.session != session {
            return not_found(b"runtime allocation not found");
        }
        let record = registry
            .allocations
            .remove(&allocation.raw())
            .expect("validated runtime allocation must remain present while registry is locked");
        let byte_len = record.bytes.len() as u64;
        if let Some(census) = registry.census.get_mut(&record.session.raw()) {
            census.outstanding_allocations = census.outstanding_allocations.saturating_sub(1);
            census.outstanding_bytes = census.outstanding_bytes.saturating_sub(byte_len);
        }
        record
    };
    drop(record);
    ZrStatus::ok()
}

pub(super) fn allocation_census(session: ZrRuntimeSessionHandle) -> RuntimeAllocationCensus {
    lock_registry()
        .census
        .get(&session.raw())
        .copied()
        .unwrap_or_default()
}

pub(super) fn session_has_outstanding_allocations(session: ZrRuntimeSessionHandle) -> bool {
    allocation_census(session).outstanding_allocations != 0
}

pub(super) fn forget_session_census(session: ZrRuntimeSessionHandle) {
    let mut registry = lock_registry();
    if registry
        .census
        .get(&session.raw())
        .is_some_and(|census| census.outstanding_allocations == 0)
    {
        registry.census.remove(&session.raw());
    }
}
