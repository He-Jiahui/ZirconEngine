use std::panic::{self, AssertUnwindSafe};

use crate::core::{CoreError, LifecycleState};

use super::super::descriptors::FrozenModuleGraph;
use super::super::state::ModuleLifecycleCommand;
use super::CoreHandle;

mod batch;
mod blocked_dependencies;
mod blocked_unload;
mod module_lifecycle;
mod service_lifecycle;
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
        let graph = self.frozen_module_graph()?;
        let activation_closure = graph.module_activation_closure(module_name)?;
        self.validate_module_activation_states(&activation_closure)?;
        for module_name in activation_closure {
            self.run_module_lifecycle_transition(
                &module_name,
                ModuleLifecycleCommand::Activate,
                || self.activate_module_once(&graph, &module_name, ready_timeout),
            )?;
        }
        Ok(())
    }

    fn validate_module_activation_states(&self, module_order: &[String]) -> Result<(), CoreError> {
        let modules = self.lock_modules();
        for module_name in module_order {
            let Some(entry) = modules.get(module_name) else {
                return Err(CoreError::MissingModule(module_name.clone()));
            };
            match entry.lifecycle {
                LifecycleState::Registered
                | LifecycleState::Running
                | LifecycleState::Unloaded
                | LifecycleState::Initializing => {}
                state => {
                    // Stopping is a committed or poisoned stop. Reject the whole
                    // activation closure here so an unloaded dependency is not
                    // rebuilt before the terminal target is refused.
                    return Err(CoreError::InvalidModuleLifecycleTransition {
                        module: module_name.clone(),
                        command: ModuleLifecycleCommand::Activate.as_str(),
                        state,
                    });
                }
            }
        }
        Ok(())
    }

    fn activate_module_once(
        &self,
        graph: &FrozenModuleGraph,
        module_name: &str,
        ready_timeout: std::time::Duration,
    ) -> Result<(), CoreError> {
        let module_services = graph.module_services(module_name)?;
        let (previous_lifecycle, service_names, startup_services) = {
            let mut modules = self.lock_modules();
            let Some(entry) = modules.get_mut(module_name) else {
                return Err(CoreError::MissingModule(module_name.to_string()));
            };
            match entry.lifecycle {
                LifecycleState::Running => return Ok(()),
                LifecycleState::Registered | LifecycleState::Unloaded => {}
                state => {
                    return Err(CoreError::InvalidModuleLifecycleTransition {
                        module: module_name.to_owned(),
                        command: ModuleLifecycleCommand::Activate.as_str(),
                        state,
                    });
                }
            }
            let previous_lifecycle = entry.lifecycle;
            entry.lifecycle = LifecycleState::Initializing;
            let service_names = module_services.service_names().clone();
            let startup_services = module_services.startup_service_names().clone();
            (previous_lifecycle, service_names, startup_services)
        };

        let mut built = false;
        let mut reactivation_services_prepared = false;
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            if previous_lifecycle == LifecycleState::Unloaded {
                self.prepare_module_services_for_reactivation(service_names.as_ref())?;
                reactivation_services_prepared = true;
            }
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
        }))
        .unwrap_or_else(|_| {
            Err(CoreError::ModuleLifecycleCallbackPanicked {
                module: module_name.to_owned(),
                command: ModuleLifecycleCommand::Activate.as_str(),
            })
        });

        if let Err(activation_error) = result {
            let rollback_error = built
                .then(|| {
                    panic::catch_unwind(AssertUnwindSafe(|| self.cleanup_module(module_name)))
                        .unwrap_or_else(|_| {
                            Err(CoreError::ModuleLifecycleCallbackPanicked {
                                module: module_name.to_owned(),
                                command: ModuleLifecycleCommand::Activate.as_str(),
                            })
                        })
                })
                .transpose()
                .err();
            if reactivation_services_prepared {
                self.rollback_module_services_after_failed_reactivation(service_names.as_ref());
            } else {
                self.reset_started_services(startup_services.as_ref());
            }
            self.reset_initializing_module(module_name, previous_lifecycle);
            return Err(CoreError::module_activation_failed(
                activation_error,
                rollback_error,
            ));
        }

        Ok(())
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.deactivate_module_with_drain_timeout_inner(module_name, None)
    }

    pub fn deactivate_module_with_drain_timeout(
        &self,
        module_name: &str,
        drain_timeout: std::time::Duration,
    ) -> Result<(), CoreError> {
        self.deactivate_module_with_drain_timeout_inner(module_name, Some(drain_timeout))
    }

    fn deactivate_module_with_drain_timeout_inner(
        &self,
        module_name: &str,
        drain_timeout: Option<std::time::Duration>,
    ) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "deactivate_module");
        let graph = self.frozen_module_graph()?;
        self.run_module_lifecycle_transition(
            module_name,
            ModuleLifecycleCommand::Deactivate,
            || self.deactivate_module_with_graph(&graph, module_name, drain_timeout),
        )
    }

    pub(super) fn deactivate_module_with_graph(
        &self,
        graph: &FrozenModuleGraph,
        module_name: &str,
        drain_timeout: Option<std::time::Duration>,
    ) -> Result<(), CoreError> {
        let module_services = graph.module_services(module_name)?;
        let unload_order = module_services.shutdown_service_names().clone();
        let module_dependents = graph.module_dependent_closure(module_name)?;
        let (lifecycle, running_dependents) = {
            let modules = self.lock_modules();
            let Some(entry) = modules.get(module_name) else {
                return Err(CoreError::MissingModule(module_name.to_string()));
            };
            let running_dependents: Vec<String> = module_dependents
                .iter()
                .filter(|dependent| {
                    modules
                        .get(dependent.as_str())
                        .is_some_and(|entry| entry.lifecycle == LifecycleState::Running)
                })
                .cloned()
                .collect();
            (entry.lifecycle, running_dependents)
        };
        let prepare_stop = match lifecycle {
            LifecycleState::Running => true,
            LifecycleState::Stopping => false,
            LifecycleState::Unloaded => return Ok(()),
            state => {
                return Err(CoreError::InvalidModuleLifecycleTransition {
                    module: module_name.to_owned(),
                    command: ModuleLifecycleCommand::Deactivate.as_str(),
                    state,
                });
            }
        };
        if prepare_stop && !running_dependents.is_empty() {
            return Err(CoreError::ModuleUnloadBlocked {
                module: module_name.to_owned(),
                dependents: running_dependents,
            });
        }

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let unload_order = unload_order.as_ref();
            if prepare_stop {
                let blocked_unload = {
                    let services = self.lock_services();
                    first_blocked_unload(&services, unload_order)
                };
                if let Some((service_name, dependents)) = blocked_unload {
                    return Err(CoreError::UnloadBlocked(service_name, dependents));
                }

                // Prepare/veto remains side-effect free: only a successful observer
                // callback may enter the fail-closed stopping transaction.
                self.notify_runtime_module_deactivating(module_name)?;

                {
                    let mut modules = self.lock_modules();
                    let Some(entry) = modules.get_mut(module_name) else {
                        return Err(CoreError::MissingModule(module_name.to_string()));
                    };
                    entry.lifecycle = LifecycleState::Stopping;
                }

                // New guarded calls stop before cleanup; active guards drain while
                // the module is stopping and before its service slots invalidate.
                self.close_service_admission(unload_order);
            }

            self.wait_for_service_calls_to_drain(module_name, unload_order, drain_timeout)?;

            self.cleanup_module(module_name)?;

            if !unload_order.is_empty() {
                let mut services = self.lock_services();
                unload_services(&mut services, unload_order);
                drop(services);
                self.notify_service_resolution_changed();
            }

            self.finish_module_deactivation(module_name)
        }))
        .unwrap_or_else(|_| {
            Err(CoreError::ModuleLifecycleCallbackPanicked {
                module: module_name.to_owned(),
                command: ModuleLifecycleCommand::Deactivate.as_str(),
            })
        });
        result
    }

    fn reset_initializing_module(&self, module_name: &str, previous_lifecycle: LifecycleState) {
        let mut modules = self.lock_modules();
        if let Some(entry) = modules.get_mut(module_name) {
            if entry.lifecycle == LifecycleState::Initializing {
                entry.lifecycle = previous_lifecycle;
            }
        }
    }

    fn finish_module_activation(&self, module_name: &str) -> Result<(), CoreError> {
        let mut active_module_order = self.lock_active_module_order();
        let mut modules = self.lock_modules();
        let Some(entry) = modules.get_mut(module_name) else {
            return Err(CoreError::MissingModule(module_name.to_string()));
        };
        entry.lifecycle = LifecycleState::Running;
        if !active_module_order
            .iter()
            .any(|active_module| active_module == module_name)
        {
            active_module_order.push(module_name.to_owned());
        }
        Ok(())
    }

    fn finish_module_deactivation(&self, module_name: &str) -> Result<(), CoreError> {
        let mut active_module_order = self.lock_active_module_order();
        let mut modules = self.lock_modules();
        let Some(entry) = modules.get_mut(module_name) else {
            return Err(CoreError::MissingModule(module_name.to_string()));
        };
        entry.lifecycle = LifecycleState::Unloaded;
        active_module_order.retain(|active_module| active_module != module_name);
        Ok(())
    }
}
