use crate::core::CoreError;
use crate::core::LifecycleState;

use super::CoreHandle;

mod batch;
mod blocked_dependencies;
mod blocked_unload;
mod module_lifecycle;
mod startup;
mod unload_mutation;

use self::blocked_unload::first_blocked_unload;
use self::unload_mutation::unload_services;

impl CoreHandle {
    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.activate_module_with_ready_timeout(module_name, Default::default())
    }

    pub fn activate_module_with_ready_timeout(
        &self,
        module_name: &str,
        ready_timeout: std::time::Duration,
    ) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "activate_module");
        let startup_services = {
            let mut modules = self.lock_modules();
            let Some(entry) = modules.get_mut(module_name) else {
                return Err(CoreError::MissingModule(module_name.to_string()));
            };
            if entry.lifecycle == LifecycleState::Running {
                return Ok(());
            }
            entry.lifecycle = LifecycleState::Initializing;
            if entry.startup_service_names.is_empty() {
                Default::default()
            } else {
                entry.startup_service_names.clone()
            }
        };

        let mut built = false;
        let result = (|| {
            self.build_module(module_name)?;
            built = true;

            if !startup_services.is_empty() {
                self.resolve_startup_services(startup_services.as_ref())?;
            }

            self.wait_until_module_ready(module_name, ready_timeout)?;
            self.finish_module(module_name)?;
            self.finish_module_activation(module_name)?;
            self.notify_runtime_module_activated(module_name);
            Ok(())
        })();

        if let Err(activation_error) = result {
            let rollback_error = built
                .then(|| self.cleanup_module(module_name))
                .transpose()
                .err();
            self.reset_started_services(startup_services.as_ref());
            self.reset_initializing_module(module_name);
            return Err(CoreError::module_activation_failed(
                activation_error,
                rollback_error,
            ));
        }

        Ok(())
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "deactivate_module");
        let (previous_lifecycle, unload_order) = {
            let mut modules = self.lock_modules();
            let Some(entry) = modules.get_mut(module_name) else {
                return Err(CoreError::MissingModule(module_name.to_string()));
            };
            let previous_lifecycle = entry.lifecycle;
            entry.lifecycle = LifecycleState::Stopping;
            let unload_order = if entry.shutdown_service_names.is_empty() {
                Default::default()
            } else {
                entry.shutdown_service_names.clone()
            };
            (previous_lifecycle, unload_order)
        };

        let result = (|| {
            let unload_order = unload_order.as_ref();
            let blocked_unload = {
                let services = self.lock_services();
                first_blocked_unload(&services, unload_order)
            };
            if let Some((service_name, dependents)) = blocked_unload {
                return Err(CoreError::UnloadBlocked(service_name, dependents));
            }

            self.cleanup_module(module_name)?;
            self.notify_runtime_module_deactivating(module_name)?;

            if !unload_order.is_empty() {
                let mut services = self.lock_services();
                unload_services(&mut services, unload_order);
            }

            self.finish_module_deactivation(module_name)
        })();

        if result.is_err() {
            self.reset_stopping_module(module_name, previous_lifecycle);
        }

        result
    }

    fn reset_initializing_module(&self, module_name: &str) {
        let mut modules = self.lock_modules();
        if let Some(entry) = modules.get_mut(module_name) {
            if entry.lifecycle == LifecycleState::Initializing {
                entry.lifecycle = LifecycleState::Registered;
            }
        }
    }

    fn reset_stopping_module(&self, module_name: &str, previous_lifecycle: LifecycleState) {
        let mut modules = self.lock_modules();
        if let Some(entry) = modules.get_mut(module_name) {
            if entry.lifecycle == LifecycleState::Stopping {
                entry.lifecycle = previous_lifecycle;
            }
        }
    }

    fn finish_module_activation(&self, module_name: &str) -> Result<(), CoreError> {
        let mut modules = self.lock_modules();
        let Some(entry) = modules.get_mut(module_name) else {
            return Err(CoreError::MissingModule(module_name.to_string()));
        };
        entry.lifecycle = LifecycleState::Running;
        Ok(())
    }

    fn finish_module_deactivation(&self, module_name: &str) -> Result<(), CoreError> {
        let mut modules = self.lock_modules();
        let Some(entry) = modules.get_mut(module_name) else {
            return Err(CoreError::MissingModule(module_name.to_string()));
        };
        entry.lifecycle = LifecycleState::Unloaded;
        Ok(())
    }
}
