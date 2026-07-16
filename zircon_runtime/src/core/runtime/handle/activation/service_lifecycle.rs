use std::collections::HashMap;

use crate::core::{CoreError, LifecycleState};

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;
use super::super::CoreHandle;

impl CoreHandle {
    pub(super) fn prepare_module_services_for_reactivation(
        &self,
        service_names: &[RegistryName],
    ) -> Result<(), CoreError> {
        if service_names.is_empty() {
            return Ok(());
        }
        let mut services = self.lock_services();
        validate_reactivation_services(&services, service_names)?;
        prepare_reactivation_services(&mut services, service_names);
        drop(services);
        self.notify_service_resolution_changed();
        Ok(())
    }

    pub(super) fn rollback_module_services_after_failed_reactivation(
        &self,
        service_names: &[RegistryName],
    ) {
        let mut services = self.lock_services();
        let changed = rollback_reactivation_services(&mut services, service_names);
        drop(services);
        if changed {
            self.notify_service_resolution_changed();
        }
    }
}

pub(super) fn validate_reactivation_services(
    services: &HashMap<RegistryName, ServiceEntry>,
    service_names: &[RegistryName],
) -> Result<(), CoreError> {
    for service_name in service_names {
        let Some(entry) = services.get(service_name) else {
            return Err(CoreError::MissingService(service_name.to_string()));
        };
        if entry.lifecycle != LifecycleState::Unloaded
            || entry.instance.is_some()
            || entry.initialization_owner.is_some()
        {
            return Err(CoreError::ServiceUnavailable(service_name.to_string()));
        }
    }
    Ok(())
}

pub(super) fn prepare_reactivation_services(
    services: &mut HashMap<RegistryName, ServiceEntry>,
    service_names: &[RegistryName],
) {
    for service_name in service_names {
        let entry = services
            .get_mut(service_name)
            .expect("validated module service should remain registered");
        entry.prepare_for_reactivation();
    }
}

pub(super) fn rollback_reactivation_services(
    services: &mut HashMap<RegistryName, ServiceEntry>,
    service_names: &[RegistryName],
) -> bool {
    let mut changed = false;
    for service_name in service_names {
        let Some(entry) = services.get_mut(service_name) else {
            continue;
        };
        if entry.lifecycle != LifecycleState::Unloaded {
            entry.reset_after_failed_reactivation();
            changed = true;
        }
    }
    changed
}
