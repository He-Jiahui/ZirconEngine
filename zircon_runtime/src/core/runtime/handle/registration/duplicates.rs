use std::collections::HashMap;

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;

pub(super) fn duplicate_existing_pending_service_name<'a>(
    services: &HashMap<RegistryName, ServiceEntry>,
    pending_services: &'a [(RegistryName, ServiceEntry)],
) -> Option<&'a RegistryName> {
    debug_assert!(pending_services.len() >= 6);
    pending_services
        .iter()
        .find_map(|(name, _)| services.contains_key(name).then_some(name))
}
