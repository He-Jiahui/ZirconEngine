use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, ThreadId};
use std::time::Instant;

use zircon_runtime::core::TaskGraphShutdownReport;

use super::{ProductCompositionFailure, RetainedOwner};

fn retained_owners() -> &'static Mutex<HashMap<ThreadId, Vec<Arc<RetainedOwner>>>> {
    static OWNERS: OnceLock<Mutex<HashMap<ThreadId, Vec<Arc<RetainedOwner>>>>> = OnceLock::new();
    OWNERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn insert(owner: &Arc<RetainedOwner>) {
    let mut registry = retained_owners()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let owners = registry.entry(owner.bucket).or_default();
    if !owners.iter().any(|existing| Arc::ptr_eq(existing, owner)) {
        owners.push(owner.clone());
    }
}

pub(super) fn release(owner: &Arc<RetainedOwner>) {
    let mut registry = retained_owners()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(owners) = registry.get_mut(&owner.bucket) {
        owners.retain(|existing| !Arc::ptr_eq(existing, owner));
        if owners.is_empty() {
            registry.remove(&owner.bucket);
        }
    }
}

pub(in crate::entry) fn pending_owner_count() -> usize {
    retained_owners()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&thread::current().id())
        .map_or(0, Vec::len)
}

#[derive(Debug)]
struct ProductAdmissionPending {
    count: usize,
}

impl fmt::Display for ProductAdmissionPending {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "product admission pending: {} original cleanup owner(s) remain",
            self.count
        )
    }
}
impl Error for ProductAdmissionPending {}

pub(in crate::entry) fn ensure_product_admission() -> Result<(), ProductCompositionFailure> {
    let count = pending_owner_count();
    if count == 0 {
        Ok(())
    } else {
        Err(ProductCompositionFailure::before_ownership(
            ProductAdmissionPending { count },
        ))
    }
}

pub(in crate::entry) fn runtime_owners() -> Vec<Arc<RetainedOwner>> {
    let owners = retained_owners()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&thread::current().id())
        .cloned()
        .unwrap_or_default();
    owners
        .into_iter()
        .filter(|owner| owner.has_runtime())
        .collect()
}

/// Retries each packet retained by this host thread under one cooperative Core deadline.
/// The registry lock is released before cleanup; exact Core-only tokens can also move threads.
/// Native packets retain their separate creator-thread guard and synchronous DLL timing.
pub fn retry_product_cleanup_until(
    deadline: Instant,
) -> Result<Vec<Option<TaskGraphShutdownReport>>, ProductCompositionFailure> {
    let owners = retained_owners()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&thread::current().id())
        .cloned()
        .unwrap_or_default();
    let mut reports = Vec::new();
    let mut first_failure = None;
    for owner in owners {
        match owner.retry_until(deadline) {
            Ok(report) => reports.push(report),
            Err(_) => {
                first_failure.get_or_insert_with(|| ProductCompositionFailure::from_owner(owner));
            }
        }
    }
    first_failure.map_or(Ok(reports), Err)
}
