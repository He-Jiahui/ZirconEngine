use std::collections::{HashMap, HashSet};

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;

const SMALL_PENDING_SERVICE_BATCH: usize = 5;

pub(super) fn duplicate_pending_service_name(
    pending_services: &[(RegistryName, ServiceEntry)],
) -> Option<&RegistryName> {
    debug_assert!(!pending_services.is_empty());
    if pending_services.len() <= SMALL_PENDING_SERVICE_BATCH {
        return duplicate_small_pending_service_name(pending_services);
    }

    let mut seen = HashSet::with_capacity(pending_services.len());
    for (name, _) in pending_services {
        if !seen.insert(name) {
            return Some(name);
        }
    }
    None
}

fn duplicate_small_pending_service_name(
    pending_services: &[(RegistryName, ServiceEntry)],
) -> Option<&RegistryName> {
    for (left_index, (left_name, _)) in pending_services.iter().enumerate() {
        for (right_name, _) in pending_services.iter().skip(left_index + 1) {
            if left_name == right_name {
                return Some(left_name);
            }
        }
    }
    None
}

pub(super) fn duplicate_existing_pending_service_name<'a>(
    services: &HashMap<RegistryName, ServiceEntry>,
    pending_services: &'a [(RegistryName, ServiceEntry)],
) -> Option<&'a RegistryName> {
    debug_assert!(!pending_services.is_empty());
    for (name, _) in pending_services {
        if services.contains_key(name) {
            return Some(name);
        }
    }
    None
}
