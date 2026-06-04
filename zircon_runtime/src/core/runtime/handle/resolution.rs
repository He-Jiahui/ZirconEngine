use std::any::Any;
use std::sync::Arc;

use crate::core::error::CoreError;
use crate::core::lifecycle::{LifecycleState, ServiceKind};
use crate::core::types::ServiceObject;

use super::super::contexts::PluginContext;
use super::super::descriptors::RegistryName;
use super::super::state::ServiceEntryFactory;
use super::CoreHandle;

impl CoreHandle {
    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Driver))?;
        Arc::downcast::<T>(service).map_err(|_| CoreError::ServiceDowncast(name.to_string()))
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Manager))?;
        Arc::downcast::<T>(service).map_err(|_| CoreError::ServiceDowncast(name.to_string()))
    }

    pub fn resolve_plugin<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Plugin))?;
        Arc::downcast::<T>(service).map_err(|_| CoreError::ServiceDowncast(name.to_string()))
    }

    pub(crate) fn resolve_named_service(
        &self,
        service_name: &str,
        expected_kind: Option<ServiceKind>,
    ) -> Result<ServiceObject, CoreError> {
        crate::profile_scope!("runtime", "core", "resolve_named_service");
        self.resolve_named_service_inner(service_name, expected_kind, &mut Vec::new())
    }

    fn resolve_named_service_inner(
        &self,
        service_name: &str,
        expected_kind: Option<ServiceKind>,
        stack: &mut Vec<RegistryName>,
    ) -> Result<ServiceObject, CoreError> {
        let service_key = {
            let services = self.inner.services.lock().unwrap();
            let entry = services
                .get(service_name)
                .ok_or_else(|| CoreError::MissingService(service_name.to_string()))?;
            if let Some(expected_kind) = expected_kind {
                if entry.kind != expected_kind {
                    return Err(CoreError::ServiceKindMismatch {
                        name: service_name.to_string(),
                        expected: expected_kind,
                        actual: entry.kind,
                    });
                }
            }
            if let Some(instance) = entry.instance.clone() {
                return Ok(instance);
            }
            entry.name.clone()
        };

        if stack.iter().any(|existing| existing == &service_key) {
            return Err(CoreError::DependencyCycle(service_key.to_string()));
        }
        stack.push(service_key.clone());

        let (owner_module, dependency_names, factory) = {
            let mut services = self.inner.services.lock().unwrap();
            let entry = services
                .get_mut(service_key.as_str())
                .ok_or_else(|| CoreError::MissingService(service_name.to_string()))?;
            entry.lifecycle = LifecycleState::Initializing;
            (
                entry.owner_module.clone(),
                entry
                    .dependencies
                    .iter()
                    .map(|dependency| dependency.name.clone())
                    .collect::<Vec<_>>(),
                entry.factory.clone(),
            )
        };

        let should_activate = {
            let modules = self.inner.modules.lock().unwrap();
            modules
                .get(&owner_module)
                .is_some_and(|module| module.lifecycle == LifecycleState::Registered)
        };
        if should_activate {
            self.activate_module(&owner_module)?;
        }

        for dependency_name in &dependency_names {
            self.resolve_named_service_inner(dependency_name.as_str(), None, stack)?;
        }

        let instance = match factory {
            ServiceEntryFactory::Service(factory) => factory(self),
            ServiceEntryFactory::Plugin(factory) => {
                let context = PluginContext {
                    plugin_name: service_name.to_string(),
                    core: self.downgrade(),
                    package_root: None,
                    source_root: None,
                    data_root: None,
                };
                factory(&context)
            }
        }
        .map_err(|error| CoreError::Initialization(service_name.to_string(), error.to_string()))?;

        {
            let mut services = self.inner.services.lock().unwrap();
            let entry = services
                .get_mut(service_key.as_str())
                .ok_or_else(|| CoreError::MissingService(service_name.to_string()))?;
            entry.instance = Some(instance.clone());
            entry.lifecycle = LifecycleState::Running;
        }

        stack.pop();
        Ok(instance)
    }
}
