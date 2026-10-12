use std::any::Any;
use std::sync::Arc;

use crate::core::manager::RegisteredManagerService;
use crate::core::{CoreError, CoreResult, LifecycleState, ServiceKind};

use super::super::{CoreHandle, RegisteredServiceIdentity};

/// Borrowed access to this stopping module's initialized service instances.
///
/// Core issues this capability only after closing admission and draining calls and
/// initialization. It does not initialize lazy services or expose an owning Core,
/// service Arc, or cloneable privileged handle. Service callbacks run outside the
/// registry locks and cannot return a borrowed service through this API.
pub struct ModuleCleanupContext<'a> {
    core: &'a CoreHandle,
    module_name: &'a str,
    identities: Vec<RegisteredServiceIdentity>,
}

impl<'a> ModuleCleanupContext<'a> {
    pub(in crate::core::runtime) fn new(
        core: &'a CoreHandle,
        module_name: &'a str,
    ) -> CoreResult<Self> {
        {
            let modules = core.lock_modules();
            let module = modules
                .get(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_owned()))?;
            if module.lifecycle != LifecycleState::Stopping {
                return Err(CoreError::ServiceUnavailable(module_name.to_owned()));
            }
        }
        let services = core.lock_services();
        let mut identities = Vec::new();
        for (name, entry) in services.iter() {
            if name.module_name() != module_name {
                continue;
            }
            if entry.admission_open
                || entry.in_flight_calls != 0
                || entry.initialization_owner.is_some()
                || !matches!(
                    entry.lifecycle,
                    LifecycleState::Stopping | LifecycleState::Unloaded
                )
            {
                return Err(CoreError::ServiceUnavailable(name.to_string()));
            }
            identities.push(RegisteredServiceIdentity::new(
                entry.index,
                entry.generation,
                name.clone(),
            ));
        }
        drop(services);
        Ok(Self {
            core,
            module_name,
            identities,
        })
    }

    pub fn module_name(&self) -> &str {
        self.module_name
    }

    /// Returns false when this registered owned driver was never initialized.
    pub fn with_existing_driver<T: Any + Send + Sync>(
        &self,
        name: &str,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        self.with_existing(name, ServiceKind::Driver, operation)
    }

    /// Returns false when this registered owned manager was never initialized.
    pub fn with_existing_manager<T: Any + Send + Sync>(
        &self,
        name: &str,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        self.with_existing(name, ServiceKind::Manager, operation)
    }

    /// Borrows the existing manager wrapper's service without exposing its Arc.
    pub fn with_existing_registered_manager<T: ?Sized + Send + Sync + 'static>(
        &self,
        name: &str,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        self.with_existing::<RegisteredManagerService<T>>(
            name,
            ServiceKind::Manager,
            |registered| operation(registered.borrowed()),
        )
    }

    fn with_existing<T: Any + Send + Sync>(
        &self,
        name: &str,
        kind: ServiceKind,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        let identity = self
            .identities
            .iter()
            .find(|identity| identity.service().as_str() == name)
            .ok_or_else(|| CoreError::ServiceUnavailable(name.to_owned()))?;
        if identity.service().service_kind() != kind {
            return Err(CoreError::ServiceKindMismatch {
                name: name.to_owned(),
                expected: kind,
                actual: identity.service().service_kind(),
            });
        }
        let instance = {
            let services = self.core.lock_services();
            let entry = services
                .get(identity.service())
                .ok_or_else(|| CoreError::MissingService(name.to_owned()))?;
            if entry.index != identity.index() || entry.generation != identity.generation() {
                return Err(CoreError::StaleServiceHandle {
                    name: name.to_owned(),
                    expected_index: identity.index(),
                    expected_generation: identity.generation(),
                    actual_index: entry.index,
                    actual_generation: entry.generation,
                });
            }
            if entry.admission_open
                || entry.in_flight_calls != 0
                || entry.initialization_owner.is_some()
                || !matches!(
                    entry.lifecycle,
                    LifecycleState::Stopping | LifecycleState::Unloaded
                )
            {
                return Err(CoreError::ServiceUnavailable(name.to_owned()));
            }
            entry.instance.clone()
        };
        let Some(instance) = instance else {
            return Ok(false);
        };
        // Cleanup is reserved for physically owned registrations. A declared
        // view does not grant access to its source after admission closes.
        if CoreHandle::is_service_alias(&instance) {
            return Err(CoreError::ServiceUnavailable(name.to_owned()));
        }
        let service = Arc::downcast::<T>(instance)
            .map_err(|_| CoreError::ServiceDowncast(name.to_owned()))?;
        operation(service.as_ref())?;
        Ok(true)
    }
}
