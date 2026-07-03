use std::time::Duration;

use crate::core::{sort_module_activation_order, CoreError, LifecycleState};

use super::super::super::descriptors::{ModuleDescriptor, RegistryName};
use super::super::CoreHandle;

struct BatchModuleActivation {
    module_name: String,
    startup_service_names: Box<[RegistryName]>,
}

impl CoreHandle {
    pub fn activate_registered_modules(&self) -> Result<(), CoreError> {
        self.activate_registered_modules_with_ready_timeout(Default::default())
    }

    pub fn activate_registered_modules_with_ready_timeout(
        &self,
        ready_timeout: Duration,
    ) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "activate_registered_modules");
        let module_order = self.sorted_registered_module_order()?;
        let pending_modules = self.begin_batch_module_activation(module_order.as_slice())?;
        if pending_modules.is_empty() {
            return Ok(());
        }

        let result = (|| {
            for pending_module in &pending_modules {
                self.build_module(&pending_module.module_name)?;
            }

            for pending_module in &pending_modules {
                if !pending_module.startup_service_names.is_empty() {
                    self.resolve_startup_services(&pending_module.startup_service_names)?;
                }
            }

            for pending_module in &pending_modules {
                self.wait_until_module_ready(&pending_module.module_name, ready_timeout)?;
            }

            for pending_module in &pending_modules {
                self.finish_module(&pending_module.module_name)?;
            }

            for pending_module in &pending_modules {
                self.finish_module_activation(&pending_module.module_name)?;
            }

            for pending_module in &pending_modules {
                self.activate_plugin_bridge_provider_for_runtime_module(
                    pending_module.module_name.as_str(),
                );
            }

            Ok(())
        })();

        if result.is_err() {
            self.reset_batch_started_services(&pending_modules);
            self.reset_batch_initializing_modules(&pending_modules);
        }

        result
    }

    fn sorted_registered_module_order(&self) -> Result<Vec<String>, CoreError> {
        let mut descriptors = self.registered_module_descriptors();
        descriptors.sort_by(|left, right| left.name.cmp(&right.name));
        sort_module_activation_order(descriptors.as_slice())
    }

    fn registered_module_descriptors(&self) -> Vec<ModuleDescriptor> {
        let modules = self.lock_modules();
        modules
            .values()
            .map(|entry| entry.descriptor().clone())
            .collect()
    }

    fn begin_batch_module_activation(
        &self,
        module_order: &[String],
    ) -> Result<Vec<BatchModuleActivation>, CoreError> {
        let mut modules = self.lock_modules();
        let mut pending_modules = Vec::with_capacity(module_order.len());
        for module_name in module_order {
            let Some(entry) = modules.get_mut(module_name) else {
                return Err(CoreError::MissingModule(module_name.clone()));
            };
            if entry.lifecycle == LifecycleState::Running {
                continue;
            }
            entry.lifecycle = LifecycleState::Initializing;
            pending_modules.push(BatchModuleActivation {
                module_name: module_name.clone(),
                startup_service_names: entry.startup_service_names.as_ref().into(),
            });
        }
        Ok(pending_modules)
    }

    fn reset_batch_started_services(&self, pending_modules: &[BatchModuleActivation]) {
        let mut services = self.lock_services();
        for pending_module in pending_modules {
            for service_name in &pending_module.startup_service_names {
                if let Some(entry) = services.get_mut(service_name) {
                    if entry.lifecycle == LifecycleState::Running
                        || entry.lifecycle == LifecycleState::Initializing
                    {
                        entry.instance = None;
                        entry.lifecycle = LifecycleState::Registered;
                    }
                }
            }
        }
    }

    fn reset_batch_initializing_modules(&self, pending_modules: &[BatchModuleActivation]) {
        let mut modules = self.lock_modules();
        for pending_module in pending_modules {
            if let Some(entry) = modules.get_mut(&pending_module.module_name) {
                if entry.lifecycle == LifecycleState::Initializing {
                    entry.lifecycle = LifecycleState::Registered;
                }
            }
        }
    }
}
