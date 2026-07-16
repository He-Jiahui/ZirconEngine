use std::time::Duration;

use crate::core::{sort_module_activation_order, CoreError, LifecycleState};

use super::super::super::descriptors::{ModuleDescriptor, RegistryName};
use super::super::CoreHandle;
use super::service_lifecycle::{
    prepare_reactivation_services, rollback_reactivation_services, validate_reactivation_services,
};

struct BatchModuleActivation {
    module_name: String,
    previous_lifecycle: LifecycleState,
    service_names: Box<[RegistryName]>,
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

        let mut built_module_count = 0;
        let mut reactivation_services_prepared = false;
        let result = (|| {
            reactivation_services_prepared =
                self.prepare_batch_reactivation_services(&pending_modules)?;
            for pending_module in &pending_modules {
                self.build_module(&pending_module.module_name)?;
                built_module_count += 1;
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
                self.notify_runtime_module_activated(pending_module.module_name.as_str());
            }

            Ok(())
        })();

        if let Err(activation_error) = result {
            let rollback_errors =
                self.cleanup_built_batch_modules(&pending_modules[..built_module_count]);
            self.reset_batch_services(&pending_modules, reactivation_services_prepared);
            self.reset_batch_initializing_modules(&pending_modules);
            return Err(CoreError::module_batch_activation_failed(
                activation_error,
                rollback_errors,
            ));
        }

        Ok(())
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
            let previous_lifecycle = entry.lifecycle;
            entry.lifecycle = LifecycleState::Initializing;
            pending_modules.push(BatchModuleActivation {
                module_name: module_name.clone(),
                previous_lifecycle,
                service_names: entry.service_names.as_ref().into(),
                startup_service_names: entry.startup_service_names.as_ref().into(),
            });
        }
        Ok(pending_modules)
    }

    fn prepare_batch_reactivation_services(
        &self,
        pending_modules: &[BatchModuleActivation],
    ) -> Result<bool, CoreError> {
        let has_reactivation = pending_modules
            .iter()
            .any(|pending| pending.previous_lifecycle == LifecycleState::Unloaded);
        if !has_reactivation {
            return Ok(false);
        }

        let mut services = self.lock_services();
        for pending_module in pending_modules {
            if pending_module.previous_lifecycle == LifecycleState::Unloaded {
                validate_reactivation_services(&services, &pending_module.service_names)?;
            }
        }
        for pending_module in pending_modules {
            if pending_module.previous_lifecycle == LifecycleState::Unloaded {
                prepare_reactivation_services(&mut services, &pending_module.service_names);
            }
        }
        drop(services);
        self.notify_service_resolution_changed();
        Ok(true)
    }

    fn reset_batch_services(
        &self,
        pending_modules: &[BatchModuleActivation],
        reactivation_services_prepared: bool,
    ) {
        let mut services = self.lock_services();
        let mut changed = false;
        for pending_module in pending_modules {
            if pending_module.previous_lifecycle == LifecycleState::Unloaded {
                if reactivation_services_prepared {
                    changed |= rollback_reactivation_services(
                        &mut services,
                        &pending_module.service_names,
                    );
                }
                continue;
            }
            for service_name in &pending_module.startup_service_names {
                if let Some(entry) = services.get_mut(service_name) {
                    if entry.lifecycle == LifecycleState::Running
                        || entry.lifecycle == LifecycleState::Initializing
                    {
                        entry.reset_after_failed_activation();
                        changed = true;
                    }
                }
            }
        }
        drop(services);
        if changed {
            self.notify_service_resolution_changed();
        }
    }

    fn cleanup_built_batch_modules(
        &self,
        built_modules: &[BatchModuleActivation],
    ) -> Vec<(String, CoreError)> {
        built_modules
            .iter()
            .rev()
            .filter_map(|pending_module| {
                self.cleanup_module(&pending_module.module_name)
                    .err()
                    .map(|error| (pending_module.module_name.clone(), error))
            })
            .collect()
    }

    fn reset_batch_initializing_modules(&self, pending_modules: &[BatchModuleActivation]) {
        let mut modules = self.lock_modules();
        for pending_module in pending_modules {
            if let Some(entry) = modules.get_mut(&pending_module.module_name) {
                if entry.lifecycle == LifecycleState::Initializing {
                    entry.lifecycle = pending_module.previous_lifecycle;
                }
            }
        }
    }
}
