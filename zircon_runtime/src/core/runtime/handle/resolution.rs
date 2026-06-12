use std::any::Any;
use std::sync::Arc;

use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{LifecycleState, ServiceKind};

use super::super::contexts::PluginContext;
use super::super::descriptors::RegistryName;
use super::super::state::ServiceEntryFactory;
use super::CoreHandle;

const RESOLUTION_STACK_FRAME_CAPACITY: usize = 1;

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

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Manager))?;
        downcast_resolved_service(name, service)
    }

    pub fn resolve_plugin<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Plugin))?;
        downcast_resolved_service(name, service)
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
        let services = self.inner.services.lock().unwrap();
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
            let services = self.inner.services.lock().unwrap();
            let Some(entry) = services.get(service_key) else {
                return Err(CoreError::MissingService(service_key.to_string()));
            };
            if let Some(instance) = entry.instance.clone() {
                return Ok(RegisteredServiceResolution::Resolved(instance));
            }
        }

        Ok(RegisteredServiceResolution::Pending)
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

        let result = (|| {
            let owner_module = service_key.module_name();
            let canonical_service_name = service_key.as_str();
            let (dependency_names, factory) = {
                let mut services = self.inner.services.lock().unwrap();
                let Some(entry) = services.get_mut(service_key) else {
                    return Err(CoreError::MissingService(service_key.to_string()));
                };
                if let Some(instance) = entry.instance.clone() {
                    return Ok(instance);
                }
                entry.lifecycle = LifecycleState::Initializing;
                (entry.dependencies.clone(), entry.factory.clone())
            };

            let should_activate = {
                let modules = self.inner.modules.lock().unwrap();
                match modules.get(owner_module) {
                    Some(module) => module.lifecycle == LifecycleState::Registered,
                    None => false,
                }
            };
            if should_activate {
                self.activate_module(owner_module)?;
                if let Some(instance) = self.resolved_service_instance(service_key) {
                    return Ok(instance);
                }
            }

            if !dependency_names.is_empty() {
                self.resolve_dependency_services(dependency_names.as_ref(), stack)?;
            }

            let factory_result = match factory {
                ServiceEntryFactory::Service(factory) => factory(self),
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

            {
                let mut services = self.inner.services.lock().unwrap();
                let Some(entry) = services.get_mut(service_key) else {
                    return Err(CoreError::MissingService(service_key.to_string()));
                };
                entry.instance = Some(instance.clone());
                entry.lifecycle = LifecycleState::Running;
            }

            Ok(instance)
        })();

        if result.is_err() {
            self.reset_initializing_service(service_key);
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

    fn reset_initializing_service(&self, service_key: &RegistryName) {
        let mut services = self.inner.services.lock().unwrap();
        if let Some(entry) = services.get_mut(service_key) {
            if entry.lifecycle == LifecycleState::Initializing && entry.instance.is_none() {
                entry.lifecycle = LifecycleState::Registered;
            }
        }
    }

    fn resolved_service_instance(&self, service_key: &RegistryName) -> Option<ServiceObject> {
        let services = self.inner.services.lock().unwrap();
        let Some(entry) = services.get(service_key) else {
            return None;
        };
        entry.instance.clone()
    }
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
