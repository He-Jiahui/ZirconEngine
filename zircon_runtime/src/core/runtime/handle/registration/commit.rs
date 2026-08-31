use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::core::{CoreError, LifecycleState};

use super::super::super::descriptors::{ModuleDescriptor, RegistryName};
use super::super::super::state::{ModuleEntry, ServiceEntry};
use super::duplicates::duplicate_existing_pending_service_name;
use super::service_lists::ModuleServiceLists;

const FIRST_REGISTERED_SERVICE_INDEX: u32 = 1;

static NEXT_REGISTERED_SERVICE_INDEX: AtomicU32 = AtomicU32::new(FIRST_REGISTERED_SERVICE_INDEX);

pub(super) fn commit_module_registration<P>(
    modules: &mut HashMap<String, ModuleEntry>,
    services: &mut HashMap<RegistryName, ServiceEntry>,
    module_name: String,
    descriptor: ModuleDescriptor,
    module_service_lists: ModuleServiceLists,
    mut pending_services: P,
) -> Result<(), CoreError>
where
    P: AsMut<[(RegistryName, ServiceEntry)]> + IntoIterator<Item = (RegistryName, ServiceEntry)>,
{
    let service_count = pending_services.as_mut().len();
    debug_assert!(service_count > 0);

    if let Some(duplicate_name) =
        duplicate_existing_pending_service_name(services, pending_services.as_mut())
    {
        return Err(CoreError::DuplicateService(duplicate_name.to_string()));
    }
    assign_service_indices(
        pending_services.as_mut().iter_mut().map(|(_, entry)| entry),
        service_count,
    )?;
    for (key, entry) in pending_services {
        services.insert(key, entry);
    }
    modules.insert(
        module_name,
        ModuleEntry {
            descriptor,
            service_names: module_service_lists.service_names,
            startup_service_names: module_service_lists.startup_service_names,
            shutdown_service_names: module_service_lists.shutdown_service_names,
            lifecycle: LifecycleState::Registered,
        },
    );
    Ok(())
}

fn assign_service_indices<'a>(
    entries: impl IntoIterator<Item = &'a mut ServiceEntry>,
    service_count: usize,
) -> Result<(), CoreError> {
    let service_count =
        u32::try_from(service_count).map_err(|_| CoreError::ServiceIdentityIndexExhausted)?;
    let first_index = NEXT_REGISTERED_SERVICE_INDEX
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next_index| {
            next_index.checked_add(service_count)
        })
        .map_err(|_| CoreError::ServiceIdentityIndexExhausted)?;

    for (offset, entry) in entries.into_iter().enumerate() {
        let offset = u32::try_from(offset).expect("service identity offset should fit in u32");
        entry.assign_index(first_index + offset);
    }
    Ok(())
}
