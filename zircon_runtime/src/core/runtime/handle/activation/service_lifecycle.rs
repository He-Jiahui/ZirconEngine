use std::collections::HashMap;

use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, LifecycleState};

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;
use super::super::CoreHandle;

impl CoreHandle {
    // 重激活复用原服务槽位，只允许已卸载且无实例的槽位重新进入注册态。
    // 同一把服务锁覆盖整组校验与复位，解析者不会观察到部分恢复的槽位。
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
        module_name: &str,
        service_names: &[RegistryName],
    ) -> Option<CoreError> {
        let mut services = self.lock_services();
        let mut retired = Vec::new();
        let changed =
            rollback_reactivation_services(&mut services, module_name, service_names, &mut retired);
        drop(services);
        if changed {
            self.notify_service_resolution_changed();
        }
        retire_service_objects(retired, "activate")
    }
}

// Retired service objects are user-owned `Drop` code. Consume each object under
// its own panic boundary so one destructor cannot unwind through the public
// activation API or trigger a second panic while the remaining objects drop.
pub(super) fn retire_service_objects(
    retired: Vec<(String, String, ServiceObject)>,
    command: &'static str,
) -> Option<CoreError> {
    let mut first_error = None;
    for (module_name, service, instance) in retired {
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(instance))).is_err()
            && first_error.is_none()
        {
            first_error = Some(CoreError::ServiceRetirementPanicked {
                module: module_name,
                service,
                command,
            });
        }
    }
    first_error
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
    module_name: &str,
    service_names: &[RegistryName],
    retired: &mut Vec<(String, String, crate::core::runtime::ServiceObject)>,
) -> bool {
    let mut changed = false;
    for service_name in service_names {
        let Some(entry) = services.get_mut(service_name) else {
            continue;
        };
        if entry.lifecycle != LifecycleState::Unloaded {
            if let Some(instance) = entry.reset_after_failed_reactivation() {
                retired.push((module_name.to_owned(), service_name.to_string(), instance));
            }
            changed = true;
        }
    }
    changed
}
