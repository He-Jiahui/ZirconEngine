use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{LifecycleState, ServiceKind};

use super::super::contexts::PluginContext;
use super::super::descriptors::RegistryName;
use super::super::state::ServiceEntryFactory;
use super::super::weak::CoreWeak;
use super::{CoreHandle, RegisteredServiceIdentity};

const RESOLUTION_STACK_FRAME_CAPACITY: usize = 1;

/// A generation-bound service reference. It cannot invoke the service until
/// [`Self::enter`] acquires a call guard from the owning runtime slot.
pub struct ServiceHandle<T> {
    core: CoreWeak,
    identity: RegisteredServiceIdentity,
    service: Arc<T>,
}

impl<T> ServiceHandle<T> {
    fn new(core: CoreWeak, identity: RegisteredServiceIdentity, service: Arc<T>) -> Self {
        Self {
            core,
            identity,
            service,
        }
    }

    pub fn enter(&self) -> Result<ServiceCallGuard<T>, CoreError> {
        let core = self.core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
        core.begin_service_call(&self.identity)?;
        Ok(ServiceCallGuard {
            core,
            identity: self.identity.clone(),
            service: Arc::clone(&self.service),
        })
    }
}

/// An admitted service invocation. Dropping the guard releases the slot's
/// in-flight count and allows an in-progress shutdown to continue draining.
pub struct ServiceCallGuard<T> {
    core: CoreHandle,
    identity: RegisteredServiceIdentity,
    service: Arc<T>,
}

impl<T> Deref for ServiceCallGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.service.as_ref()
    }
}

impl<T> Drop for ServiceCallGuard<T> {
    fn drop(&mut self) {
        self.core.release_service_call(&self.identity);
    }
}

enum NamedServiceResolution {
    Resolved(ServiceObject),
    Pending(RegistryName),
}

enum RegisteredServiceResolution {
    Resolved(ServiceObject),
    Pending,
}

impl CoreHandle {
    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Driver))?;
        downcast_resolved_service(name, service)
    }

    pub fn resolve_driver_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.resolve_service_handle(name, ServiceKind::Driver)
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Manager))?;
        downcast_resolved_service(name, service)
    }

    pub fn resolve_manager_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.resolve_service_handle(name, ServiceKind::Manager)
    }

    pub(crate) fn registered_manager_identity(
        &self,
        service_name: &str,
    ) -> Result<RegisteredServiceIdentity, CoreError> {
        self.registered_service_identity(service_name, ServiceKind::Manager)
    }

    fn registered_service_identity(
        &self,
        service_name: &str,
        expected_kind: ServiceKind,
    ) -> Result<RegisteredServiceIdentity, CoreError> {
        let services = self.lock_services();
        let Some((name, entry)) = services.get_key_value(service_name) else {
            return Err(CoreError::MissingService(service_name.to_owned()));
        };
        let actual_kind = name.service_kind();
        if actual_kind != expected_kind {
            return Err(CoreError::ServiceKindMismatch {
                name: service_name.to_owned(),
                expected: expected_kind,
                actual: actual_kind,
            });
        }
        ensure_service_resolution_available(name.as_str(), entry.lifecycle)?;
        Ok(RegisteredServiceIdentity::new(
            entry.index,
            entry.generation,
            name.clone(),
        ))
    }

    pub(crate) fn resolve_registered_manager<T: Any + Send + Sync>(
        &self,
        identity: &RegisteredServiceIdentity,
    ) -> Result<Arc<T>, CoreError> {
        let service = match self
            .registered_service_resolution_for_identity(identity, ServiceKind::Manager)?
        {
            RegisteredServiceResolution::Resolved(instance) => instance,
            RegisteredServiceResolution::Pending => {
                let mut stack = Vec::with_capacity(RESOLUTION_STACK_FRAME_CAPACITY);
                let instance =
                    self.resolve_existing_service_inner(identity.service(), &mut stack)?;
                self.validate_registered_service_identity(identity, ServiceKind::Manager)?;
                instance
            }
        };
        downcast_resolved_service(identity.service().as_str(), service)
    }

    pub fn resolve_plugin<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Plugin))?;
        downcast_resolved_service(name, service)
    }

    pub fn resolve_plugin_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.resolve_service_handle(name, ServiceKind::Plugin)
    }

    fn resolve_service_handle<T: Any + Send + Sync>(
        &self,
        service_name: &str,
        expected_kind: ServiceKind,
    ) -> Result<ServiceHandle<T>, CoreError> {
        let service = self.resolve_named_service(service_name, Some(expected_kind))?;
        let service = downcast_resolved_service(service_name, service)?;
        let identity = self.registered_service_identity(service_name, expected_kind)?;
        Ok(ServiceHandle::new(self.downgrade(), identity, service))
    }

    fn begin_service_call(&self, identity: &RegisteredServiceIdentity) -> Result<(), CoreError> {
        let mut services = self.lock_services();
        let Some(entry) = services.get_mut(identity.service()) else {
            return Err(CoreError::MissingService(identity.service().to_string()));
        };
        validate_service_identity(
            identity,
            entry.index,
            entry.generation,
            identity.service().service_kind(),
        )?;
        entry.enter_call(identity.service().as_str())
    }

    fn release_service_call(&self, identity: &RegisteredServiceIdentity) {
        let mut services = self.lock_services();
        let became_idle = services
            .get_mut(identity.service())
            .filter(|entry| {
                entry.index == identity.index() && entry.generation == identity.generation()
            })
            .is_some_and(|entry| entry.leave_call());
        drop(services);
        if became_idle {
            self.inner.service_call_changed.notify_all();
        }
    }

    pub(super) fn close_service_admission(&self, service_names: &[RegistryName]) {
        let mut services = self.lock_services();
        for service_name in service_names {
            if let Some(entry) = services.get_mut(service_name) {
                entry.close_admission();
            }
        }
        drop(services);
        self.notify_service_resolution_changed();
    }

    pub(super) fn wait_for_service_calls_to_drain(
        &self,
        module_name: &str,
        service_names: &[RegistryName],
        drain_timeout: Option<Duration>,
    ) -> Result<(), CoreError> {
        let started_at = Instant::now();
        let mut services = self.lock_services();
        loop {
            let in_flight_calls = service_names
                .iter()
                .filter_map(|service_name| services.get(service_name))
                .fold(0usize, |total, entry| {
                    total.saturating_add(entry.in_flight_calls)
                });
            if in_flight_calls == 0 {
                return Ok(());
            }

            let Some(drain_timeout) = drain_timeout else {
                services = self
                    .inner
                    .service_call_changed
                    .wait(services)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                continue;
            };
            let remaining = drain_timeout.saturating_sub(started_at.elapsed());
            if remaining.is_zero() {
                return Err(CoreError::ServiceCallDrainTimeout {
                    module: module_name.to_owned(),
                    budget: drain_timeout,
                    in_flight_calls,
                });
            }
            let (next_services, _) = self
                .inner
                .service_call_changed
                .wait_timeout(services, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            services = next_services;
        }
    }

    pub(crate) fn resolve_named_service(
        &self,
        service_name: &str,
        expected_kind: Option<ServiceKind>,
    ) -> Result<ServiceObject, CoreError> {
        crate::profile_scope!("runtime", "core", "resolve_named_service");
        match self.named_service_resolution(service_name, expected_kind)? {
            NamedServiceResolution::Resolved(instance) => Ok(instance),
            NamedServiceResolution::Pending(service_key) => {
                let mut stack = Vec::with_capacity(RESOLUTION_STACK_FRAME_CAPACITY);
                self.resolve_existing_service_inner(&service_key, &mut stack)
            }
        }
    }

    pub(crate) fn resolve_registered_service(
        &self,
        service_key: &RegistryName,
        expected_kind: Option<ServiceKind>,
    ) -> Result<ServiceObject, CoreError> {
        crate::profile_scope!("runtime", "core", "resolve_registered_service");
        match self.registered_service_resolution(service_key, expected_kind)? {
            RegisteredServiceResolution::Resolved(instance) => Ok(instance),
            RegisteredServiceResolution::Pending => {
                let mut stack = Vec::with_capacity(RESOLUTION_STACK_FRAME_CAPACITY);
                self.resolve_existing_service_inner(service_key, &mut stack)
            }
        }
    }

    fn named_service_resolution(
        &self,
        service_name: &str,
        expected_kind: Option<ServiceKind>,
    ) -> Result<NamedServiceResolution, CoreError> {
        let services = self.lock_services();
        let Some((name, entry)) = services.get_key_value(service_name) else {
            return Err(CoreError::MissingService(service_name.to_string()));
        };
        if let Some(expected_kind) = expected_kind {
            let actual_kind = name.service_kind();
            if actual_kind != expected_kind {
                return Err(CoreError::ServiceKindMismatch {
                    name: service_name.to_string(),
                    expected: expected_kind,
                    actual: actual_kind,
                });
            }
        }
        ensure_service_resolution_available(name.as_str(), entry.lifecycle)?;
        if let Some(instance) = entry.instance.clone() {
            return Ok(NamedServiceResolution::Resolved(instance));
        }

        Ok(NamedServiceResolution::Pending(name.clone()))
    }

    fn resolve_registered_service_inner(
        &self,
        service_key: &RegistryName,
        expected_kind: Option<ServiceKind>,
        stack: &mut Vec<RegistryName>,
    ) -> Result<ServiceObject, CoreError> {
        match self.registered_service_resolution(service_key, expected_kind)? {
            RegisteredServiceResolution::Resolved(instance) => Ok(instance),
            RegisteredServiceResolution::Pending => {
                self.resolve_existing_service_inner(service_key, stack)
            }
        }
    }

    fn registered_service_resolution(
        &self,
        service_key: &RegistryName,
        expected_kind: Option<ServiceKind>,
    ) -> Result<RegisteredServiceResolution, CoreError> {
        if let Some(expected_kind) = expected_kind {
            let actual_kind = service_key.service_kind();
            if actual_kind != expected_kind {
                return Err(CoreError::ServiceKindMismatch {
                    name: service_key.to_string(),
                    expected: expected_kind,
                    actual: actual_kind,
                });
            }
        }
        {
            let services = self.lock_services();
            let Some(entry) = services.get(service_key) else {
                return Err(CoreError::MissingService(service_key.to_string()));
            };
            ensure_service_resolution_available(service_key.as_str(), entry.lifecycle)?;
            if let Some(instance) = entry.instance.clone() {
                return Ok(RegisteredServiceResolution::Resolved(instance));
            }
        }

        Ok(RegisteredServiceResolution::Pending)
    }

    fn registered_service_resolution_for_identity(
        &self,
        identity: &RegisteredServiceIdentity,
        expected_kind: ServiceKind,
    ) -> Result<RegisteredServiceResolution, CoreError> {
        let services = self.lock_services();
        let Some(entry) = services.get(identity.service()) else {
            return Err(CoreError::MissingService(identity.service().to_string()));
        };
        validate_service_identity(identity, entry.index, entry.generation, expected_kind)?;
        ensure_service_resolution_available(identity.service().as_str(), entry.lifecycle)?;
        if let Some(instance) = entry.instance.clone() {
            return Ok(RegisteredServiceResolution::Resolved(instance));
        }
        Ok(RegisteredServiceResolution::Pending)
    }

    fn validate_registered_service_identity(
        &self,
        identity: &RegisteredServiceIdentity,
        expected_kind: ServiceKind,
    ) -> Result<(), CoreError> {
        let services = self.lock_services();
        let Some(entry) = services.get(identity.service()) else {
            return Err(CoreError::MissingService(identity.service().to_string()));
        };
        validate_service_identity(identity, entry.index, entry.generation, expected_kind)
    }

    fn resolve_existing_service_inner(
        &self,
        service_key: &RegistryName,
        stack: &mut Vec<RegistryName>,
    ) -> Result<ServiceObject, CoreError> {
        if resolution_stack_contains(stack.as_slice(), service_key) {
            return Err(CoreError::DependencyCycle(service_key.to_string()));
        }
        stack.push(service_key.clone());
        let current_thread = thread::current().id();
        let mut claimed_initialization = false;

        let result = (|| {
            let owner_module = service_key.module_name();
            let canonical_service_name = service_key.as_str();
            let (dependency_names, factory, resolution_index, resolution_generation) = {
                let mut services = self.lock_services();
                loop {
                    let Some(entry) = services.get_mut(service_key) else {
                        return Err(CoreError::MissingService(service_key.to_string()));
                    };
                    if let Some(instance) = entry.instance.clone() {
                        return Ok(instance);
                    }
                    ensure_service_resolution_available(service_key.as_str(), entry.lifecycle)?;
                    if entry.lifecycle != LifecycleState::Initializing {
                        entry.lifecycle = LifecycleState::Initializing;
                        entry.initialization_owner = Some(current_thread);
                        claimed_initialization = true;
                        break (
                            entry.dependencies.clone(),
                            entry.factory.clone(),
                            entry.index,
                            entry.generation,
                        );
                    }
                    let Some(initialization_owner) = entry.initialization_owner else {
                        return Err(CoreError::ServiceUnavailable(service_key.to_string()));
                    };
                    if initialization_owner == current_thread
                        && self.take_service_activation_reentry(current_thread, service_key)
                    {
                        claimed_initialization = true;
                        break (
                            entry.dependencies.clone(),
                            entry.factory.clone(),
                            entry.index,
                            entry.generation,
                        );
                    }
                    if !self
                        .try_register_service_resolution_wait(current_thread, initialization_owner)
                    {
                        return Err(CoreError::DependencyCycle(service_key.to_string()));
                    }
                    services = self.wait_for_service_resolution_change(services);
                    self.clear_service_resolution_wait(current_thread);
                }
            };

            #[cfg(test)]
            self.wait_on_service_resolution_claim_barrier();

            let should_activate = {
                let modules = self.lock_modules();
                match modules.get(owner_module) {
                    Some(module) => module.lifecycle == LifecycleState::Registered,
                    None => false,
                }
            };
            if should_activate {
                self.register_service_activation_reentry(current_thread, service_key.clone());
                let activation_result = self.activate_module(owner_module);
                self.clear_service_activation_reentry(current_thread, service_key);
                activation_result?;
                if let Some(instance) = self.resolved_service_instance(service_key) {
                    return Ok(instance);
                }
            }

            if !dependency_names.is_empty() {
                self.resolve_dependency_services(dependency_names.as_ref(), stack)?;
            }

            let factory_result = match factory {
                ServiceEntryFactory::Service(factory) => factory(&self.downgrade()),
                ServiceEntryFactory::Plugin(factory) => {
                    let context = PluginContext {
                        plugin_name: canonical_service_name.to_owned(),
                        core: self.downgrade(),
                        package_root: None,
                        source_root: None,
                        data_root: None,
                    };
                    factory(&context)
                }
            };
            let instance = match factory_result {
                Ok(instance) => instance,
                Err(error) => {
                    return Err(CoreError::Initialization(
                        canonical_service_name.to_owned(),
                        error.to_string(),
                    ));
                }
            };

            let committed = (|| {
                let mut services = self.lock_services();
                let Some(entry) = services.get_mut(service_key) else {
                    return Err(CoreError::MissingService(service_key.to_string()));
                };
                if entry.index != resolution_index
                    || entry.generation != resolution_generation
                    || matches!(
                        entry.lifecycle,
                        LifecycleState::Stopping | LifecycleState::Unloaded
                    )
                {
                    return Err(CoreError::ServiceUnavailable(service_key.to_string()));
                }
                if entry.initialization_owner != Some(current_thread) {
                    return Err(CoreError::ServiceUnavailable(service_key.to_string()));
                }
                if let Some(existing) = entry.instance.clone() {
                    return Ok(existing);
                }
                if entry.lifecycle != LifecycleState::Initializing {
                    return Err(CoreError::ServiceUnavailable(service_key.to_string()));
                }
                entry.instance = Some(instance.clone());
                entry.initialization_owner = None;
                entry.lifecycle = LifecycleState::Running;
                entry.open_admission();
                Ok(instance)
            })();
            self.notify_service_resolution_changed();
            committed
        })();

        if result.is_err() && claimed_initialization {
            self.reset_initializing_service(service_key, current_thread);
        }

        stack.pop();
        result
    }

    fn resolve_dependency_services(
        &self,
        dependency_names: &[RegistryName],
        stack: &mut Vec<RegistryName>,
    ) -> Result<(), CoreError> {
        reserve_dependency_resolution_frame(stack);
        if let [dependency_name] = dependency_names {
            self.resolve_registered_service_inner(dependency_name, None, stack)?;
            return Ok(());
        }
        if let [first_dependency_name, second_dependency_name] = dependency_names {
            self.resolve_registered_service_inner(first_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(second_dependency_name, None, stack)?;
            return Ok(());
        }
        if let [first_dependency_name, second_dependency_name, third_dependency_name] =
            dependency_names
        {
            self.resolve_registered_service_inner(first_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(second_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(third_dependency_name, None, stack)?;
            return Ok(());
        }
        if let [first_dependency_name, second_dependency_name, third_dependency_name, fourth_dependency_name] =
            dependency_names
        {
            self.resolve_registered_service_inner(first_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(second_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(third_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(fourth_dependency_name, None, stack)?;
            return Ok(());
        }
        if let [first_dependency_name, second_dependency_name, third_dependency_name, fourth_dependency_name, fifth_dependency_name] =
            dependency_names
        {
            self.resolve_registered_service_inner(first_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(second_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(third_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(fourth_dependency_name, None, stack)?;
            self.resolve_registered_service_inner(fifth_dependency_name, None, stack)?;
            return Ok(());
        }

        for dependency_name in dependency_names {
            self.resolve_registered_service_inner(dependency_name, None, stack)?;
        }

        Ok(())
    }

    fn reset_initializing_service(
        &self,
        service_key: &RegistryName,
        initialization_owner: thread::ThreadId,
    ) {
        let mut services = self.lock_services();
        let mut reset = false;
        if let Some(entry) = services.get_mut(service_key) {
            if entry.lifecycle == LifecycleState::Initializing
                && entry.initialization_owner == Some(initialization_owner)
                && entry.instance.is_none()
            {
                entry.initialization_owner = None;
                entry.lifecycle = LifecycleState::Registered;
                reset = true;
            }
        }
        drop(services);
        if reset {
            self.notify_service_resolution_changed();
        }
    }

    fn resolved_service_instance(&self, service_key: &RegistryName) -> Option<ServiceObject> {
        let services = self.lock_services();
        let Some(entry) = services.get(service_key) else {
            return None;
        };
        entry.instance.clone()
    }
}

fn validate_service_identity(
    identity: &RegisteredServiceIdentity,
    actual_index: u32,
    actual_generation: u32,
    expected_kind: ServiceKind,
) -> Result<(), CoreError> {
    let actual_kind = identity.service().service_kind();
    if actual_kind != expected_kind {
        return Err(CoreError::ServiceKindMismatch {
            name: identity.service().to_string(),
            expected: expected_kind,
            actual: actual_kind,
        });
    }
    if actual_index != identity.index() || actual_generation != identity.generation() {
        return Err(CoreError::StaleServiceHandle {
            name: identity.service().to_string(),
            expected_index: identity.index(),
            expected_generation: identity.generation(),
            actual_index,
            actual_generation,
        });
    }
    Ok(())
}

fn ensure_service_resolution_available(
    service_name: &str,
    lifecycle: LifecycleState,
) -> Result<(), CoreError> {
    if matches!(
        lifecycle,
        LifecycleState::Stopping | LifecycleState::Unloaded
    ) {
        return Err(CoreError::ServiceUnavailable(service_name.to_owned()));
    }
    Ok(())
}

fn downcast_resolved_service<T: Any + Send + Sync>(
    name: &str,
    service: ServiceObject,
) -> Result<Arc<T>, CoreError> {
    match Arc::downcast::<T>(service) {
        Ok(service) => Ok(service),
        Err(_) => Err(CoreError::ServiceDowncast(name.to_string())),
    }
}

fn reserve_dependency_resolution_frame(stack: &mut Vec<RegistryName>) {
    stack.reserve(RESOLUTION_STACK_FRAME_CAPACITY);
}

fn resolution_stack_contains(stack: &[RegistryName], service_key: &RegistryName) -> bool {
    match stack {
        [] => false,
        [existing] => existing == service_key,
        [first_existing, second_existing] => {
            first_existing == service_key || second_existing == service_key
        }
        [first_existing, second_existing, third_existing] => {
            first_existing == service_key
                || second_existing == service_key
                || third_existing == service_key
        }
        [first_existing, second_existing, third_existing, fourth_existing] => {
            first_existing == service_key
                || second_existing == service_key
                || third_existing == service_key
                || fourth_existing == service_key
        }
        [first_existing, second_existing, third_existing, fourth_existing, fifth_existing] => {
            first_existing == service_key
                || second_existing == service_key
                || third_existing == service_key
                || fourth_existing == service_key
                || fifth_existing == service_key
        }
        _ => {
            for existing in stack {
                if existing == service_key {
                    return true;
                }
            }
            false
        }
    }
}
