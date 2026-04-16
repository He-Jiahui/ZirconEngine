use std::collections::HashSet;

use crate::error::CoreError;
use crate::lifecycle::{LifecycleState, ServiceKind, StartupMode};

use super::CoreHandle;

impl CoreHandle {
    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        {
            let mut modules = self.inner.modules.lock().unwrap();
            let entry = modules
                .get_mut(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
            if entry.lifecycle == LifecycleState::Running {
                return Ok(());
            }
            entry.lifecycle = LifecycleState::Initializing;
        }

        let immediate_services = {
            let services = self.inner.services.lock().unwrap();
            let mut names: Vec<_> = services
                .values()
                .filter(|entry| {
                    entry.owner_module == module_name
                        && entry.startup_mode == StartupMode::Immediate
                })
                .map(|entry| (entry.kind, entry.name.to_string()))
                .collect();
            names.sort_by_key(|(kind, _)| match kind {
                ServiceKind::Driver => 0,
                ServiceKind::Manager => 1,
                ServiceKind::Plugin => 2,
            });
            names.into_iter().map(|(_, name)| name).collect::<Vec<_>>()
        };

        for service in immediate_services {
            self.resolve_named_service(&service, None)?;
        }

        let mut modules = self.inner.modules.lock().unwrap();
        let entry = modules
            .get_mut(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
        entry.lifecycle = LifecycleState::Running;
        Ok(())
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        {
            let mut modules = self.inner.modules.lock().unwrap();
            let entry = modules
                .get_mut(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
            entry.lifecycle = LifecycleState::Stopping;
        }

        let unload_order = {
            let services = self.inner.services.lock().unwrap();
            let mut names: Vec<_> = services
                .values()
                .filter(|entry| entry.owner_module == module_name)
                .map(|entry| (entry.kind, entry.name.to_string()))
                .collect();
            names.sort_by_key(|(kind, _)| match kind {
                ServiceKind::Plugin => 0,
                ServiceKind::Manager => 1,
                ServiceKind::Driver => 2,
            });
            names.into_iter().map(|(_, name)| name).collect::<Vec<_>>()
        };
        let unloading: HashSet<_> = unload_order.iter().cloned().collect();

        for service_name in unload_order {
            let dependents = self.running_dependents(&service_name, &unloading);
            if !dependents.is_empty() {
                return Err(CoreError::UnloadBlocked(service_name, dependents));
            }
            if let Some(entry) = self.inner.services.lock().unwrap().get_mut(&service_name) {
                entry.instance = None;
                entry.lifecycle = LifecycleState::Unloaded;
            }
        }

        let mut modules = self.inner.modules.lock().unwrap();
        let entry = modules
            .get_mut(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
        entry.lifecycle = LifecycleState::Unloaded;
        Ok(())
    }

    fn running_dependents(&self, service_name: &str, unloading: &HashSet<String>) -> Vec<String> {
        self.inner
            .services
            .lock()
            .unwrap()
            .values()
            .filter(|entry| {
                entry.name.as_str() != service_name
                    && !unloading.contains(entry.name.as_str())
                    && entry.instance.is_some()
                    && entry
                        .dependencies
                        .iter()
                        .any(|dependency| dependency.name.as_str() == service_name)
            })
            .map(|entry| entry.name.to_string())
            .collect()
    }
}
