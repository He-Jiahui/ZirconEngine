use std::time::{Duration, Instant};

use std::sync::Arc;

use crate::core::{CoreError, LifecycleState};

use super::super::super::descriptors::{FrozenModuleGraph, RegistryName};
use super::super::super::state::{ModuleLifecycleCommand, ModuleLifecycleTransitionPermit};
use super::super::CoreHandle;
use super::service_lifecycle::{
    prepare_reactivation_services, retire_service_objects, rollback_reactivation_services,
    validate_reactivation_services,
};

pub(super) mod transaction_set;

use self::transaction_set::LifecycleTransactionSet;

struct BatchModuleActivation {
    module_name: String,
    previous_lifecycle: LifecycleState,
    service_names: Arc<[RegistryName]>,
    startup_service_names: Arc<[RegistryName]>,
}

struct BatchCleanupOutcome {
    cleaned_modules: Vec<String>,
    // The callback itself completed even when service retirement/reset failed.
    // Keep this receipt on the module so a retry finishes teardown without
    // replaying user cleanup.
    cleanup_completed_modules: Vec<String>,
    failures: Vec<(String, CoreError)>,
}

impl CoreHandle {
    /// 按冻结图的依赖顺序批量激活；先取得转换令牌，再构建模块与解析即时服务。
    /// 失败时尝试清理本批已构建模块，并向等待同一批转换的调用者公布错误。
    pub fn activate_registered_modules(&self) -> Result<(), CoreError> {
        self.activate_registered_modules_with_ready_deadline(Default::default(), None)
    }

    pub fn activate_registered_modules_with_ready_timeout(
        &self,
        ready_timeout: Duration,
    ) -> Result<(), CoreError> {
        let ready_deadline = Instant::now()
            .checked_add(ready_timeout)
            .unwrap_or_else(Instant::now);
        self.activate_registered_modules_with_ready_deadline(ready_timeout, Some(ready_deadline))
    }

    fn activate_registered_modules_with_ready_deadline(
        &self,
        ready_timeout: Duration,
        transition_deadline: Option<Instant>,
    ) -> Result<(), CoreError> {
        // Capture one deadline before graph freeze and transition admission.
        let ready_deadline = transition_deadline.unwrap_or_else(Instant::now);
        crate::profile_scope!("runtime", "core", "activate_registered_modules");
        let graph = self.frozen_module_graph()?;
        let mut owner_modules = Vec::with_capacity(graph.module_activation_order().len());
        let mut transitions = LifecycleTransactionSet::new(
            self,
            graph.module_activation_order().len(),
            "batch activation",
        );
        for module_name in graph.module_activation_order() {
            let permit = match super::acquire_activation_transition_reservation(
                self,
                module_name,
                transition_deadline,
                ready_timeout.is_zero(),
            ) {
                Ok(permit) => permit,
                Err(error) => return transitions.finish(Err(error)),
            };
            match permit {
                ModuleLifecycleTransitionPermit::Owner(token) => {
                    let preexisting_running = self
                        .lock_modules()
                        .get(module_name.as_str())
                        .is_some_and(|entry| entry.lifecycle == LifecycleState::Running);
                    transitions.push(module_name, token, preexisting_running);
                    owner_modules.push(module_name.clone());
                }
                ModuleLifecycleTransitionPermit::Completed(Err(error)) => {
                    return transitions.finish(Err(error));
                }
                ModuleLifecycleTransitionPermit::Completed(Ok(())) => {
                    unreachable!("activation reservation retries joined successful receipts")
                }
                ModuleLifecycleTransitionPermit::Wait => {
                    return transitions.finish(Err(
                        CoreError::ModuleLifecycleCoordinatorUnresolved {
                            module: module_name.clone(),
                            command: ModuleLifecycleCommand::Activate.as_str(),
                        },
                    ));
                }
            }
        }

        let result = self.activate_owned_modules_with_ready_deadline(
            &graph,
            owner_modules.as_slice(),
            ready_timeout,
            ready_deadline,
        );
        transitions.finish(result)
    }

    fn activate_owned_modules_with_ready_deadline(
        &self,
        graph: &FrozenModuleGraph,
        module_order: &[String],
        ready_timeout: Duration,
        ready_deadline: Instant,
    ) -> Result<(), CoreError> {
        let pending_modules = self.begin_batch_module_activation(graph, module_order)?;
        if pending_modules.is_empty() {
            return Ok(());
        }

        let mut built_module_count = 0;
        let mut panic_module = "batch activation".to_owned();
        let mut reactivation_services_prepared = false;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            reactivation_services_prepared =
                self.prepare_batch_reactivation_services(&pending_modules)?;
            for pending_module in &pending_modules {
                panic_module = pending_module.module_name.clone();
                // A build callback may resolve lazy services before returning an
                // error. Include that module in the compensation set as soon as
                // its callback starts.
                built_module_count += 1;
                self.build_module(&pending_module.module_name)?;
            }

            for pending_module in &pending_modules {
                panic_module = pending_module.module_name.clone();
                if !pending_module.startup_service_names.is_empty() {
                    self.resolve_startup_services(&pending_module.startup_service_names)?;
                }
            }

            for pending_module in &pending_modules {
                panic_module = pending_module.module_name.clone();
                self.wait_until_module_ready_until(
                    &pending_module.module_name,
                    ready_deadline,
                    ready_timeout,
                )?;
            }

            for pending_module in &pending_modules {
                panic_module = pending_module.module_name.clone();
                self.finish_module(&pending_module.module_name)?;
            }

            for pending_module in &pending_modules {
                panic_module = pending_module.module_name.clone();
                self.finish_module_activation(&pending_module.module_name)?;
            }

            for pending_module in &pending_modules {
                panic_module = pending_module.module_name.clone();
                self.notify_runtime_module_activated(pending_module.module_name.as_str());
            }

            Ok(())
        }))
        .unwrap_or_else(|_| {
            Err(CoreError::ModuleLifecycleCallbackPanicked {
                module: panic_module,
                command: ModuleLifecycleCommand::Activate.as_str(),
            })
        });

        if let Err(activation_error) = result {
            self.close_batch_service_admission(&pending_modules);
            // A build callback may have materialized a lazy service owned by a
            // later module.  Retire those unstarted instances before touching
            // a provider so the provider can be compensated without leaving a
            // dependent in Initializing state.
            let unstarted_reset = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for pending_module in pending_modules.iter().skip(built_module_count) {
                    let remaining = ready_deadline.saturating_duration_since(Instant::now());
                    self.wait_for_service_calls_to_drain(
                        &pending_module.module_name,
                        pending_module.service_names.as_ref(),
                        Some(remaining),
                    )?;
                }
                if let Some(error) = self.reset_batch_services(
                    &pending_modules,
                    reactivation_services_prepared,
                    built_module_count,
                    &[],
                ) {
                    return Err(error);
                }
                Ok::<(), CoreError>(())
            }));
            let unstarted_reset_error = match &unstarted_reset {
                Ok(Ok(())) => None,
                Ok(Err(error)) => Some(error.clone()),
                Err(_) => Some(CoreError::ModuleLifecycleCallbackPanicked {
                    module: "batch activation".to_owned(),
                    command: ModuleLifecycleCommand::Activate.as_str(),
                }),
            };
            let unstarted_reset_succeeded = unstarted_reset_error.is_none();
            if unstarted_reset_succeeded {
                self.reset_batch_modules(&pending_modules, built_module_count, &[], true);
            } else {
                for pending_module in pending_modules.iter().skip(built_module_count) {
                    self.mark_activation_rollback_stopping(&pending_module.module_name);
                }
            }
            let mut cleanup = self.cleanup_built_batch_modules(
                graph,
                &pending_modules[..built_module_count],
                ready_deadline,
                reactivation_services_prepared,
            );
            for module_name in &cleanup.cleanup_completed_modules {
                self.mark_module_cleanup_completed(module_name);
            }
            if let Some(error) = unstarted_reset_error {
                cleanup
                    .failures
                    .push(("batch activation".to_owned(), error));
            }
            // Restore every module whose cleanup completed.  A failure in a
            // sibling must not replay successful cleanup callbacks on retry.
            let mut restorable_cleaned = cleanup.cleaned_modules.clone();
            let reset_built_module_count = if unstarted_reset_succeeded {
                built_module_count
            } else {
                pending_modules.len()
            };
            let reset_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.reset_batch_services(
                    &pending_modules,
                    reactivation_services_prepared,
                    reset_built_module_count,
                    restorable_cleaned.as_slice(),
                )
                .map_or(Ok(()), Err)
            }));
            let reset_error = match reset_result {
                Ok(Ok(())) => None,
                Ok(Err(error)) => Some(error),
                Err(_) => Some(CoreError::ModuleLifecycleCallbackPanicked {
                    module: "batch activation".to_owned(),
                    command: ModuleLifecycleCommand::Activate.as_str(),
                }),
            };
            if reset_error.is_some() {
                restorable_cleaned.clear();
            }
            self.reset_batch_modules(
                &pending_modules,
                reset_built_module_count,
                restorable_cleaned.as_slice(),
                unstarted_reset_succeeded,
            );
            if let Some(error) = reset_error {
                cleanup
                    .failures
                    .push(("batch activation".to_owned(), error));
            }
            return Err(CoreError::module_batch_activation_failed(
                activation_error,
                cleanup.failures,
            ));
        }

        Ok(())
    }

    fn begin_batch_module_activation(
        &self,
        graph: &FrozenModuleGraph,
        module_order: &[String],
    ) -> Result<Vec<BatchModuleActivation>, CoreError> {
        let mut modules = self.lock_modules();
        let mut pending_modules = Vec::with_capacity(module_order.len());
        for module_name in module_order {
            let Some(entry) = modules.get(module_name) else {
                return Err(CoreError::MissingModule(module_name.clone()));
            };
            if !matches!(
                entry.lifecycle,
                LifecycleState::Registered | LifecycleState::Running | LifecycleState::Unloaded
            ) {
                return Err(CoreError::InvalidModuleLifecycleTransition {
                    module: module_name.clone(),
                    command: ModuleLifecycleCommand::Activate.as_str(),
                    state: entry.lifecycle,
                });
            }
        }

        for module_name in module_order {
            let entry = modules
                .get_mut(module_name)
                .expect("prevalidated module should remain registered");
            if entry.lifecycle == LifecycleState::Running {
                continue;
            }
            let module_services = graph.module_services(module_name)?;
            let previous_lifecycle = entry.lifecycle;
            entry.lifecycle = LifecycleState::Initializing;
            pending_modules.push(BatchModuleActivation {
                module_name: module_name.clone(),
                previous_lifecycle,
                service_names: module_services.service_names().clone(),
                startup_service_names: module_services.startup_service_names().clone(),
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
        built_module_count: usize,
        cleaned_modules: &[String],
    ) -> Option<CoreError> {
        let mut services = self.lock_services();
        let mut changed = false;
        let mut retired = Vec::new();
        for (index, pending_module) in pending_modules.iter().enumerate() {
            let cleaned = cleaned_modules
                .iter()
                .any(|module| module == &pending_module.module_name);
            let unstarted = index >= built_module_count;
            if pending_module.previous_lifecycle == LifecycleState::Unloaded {
                if reactivation_services_prepared && (cleaned || unstarted) {
                    changed |= rollback_reactivation_services(
                        &mut services,
                        &pending_module.module_name,
                        &pending_module.service_names,
                        &mut retired,
                    );
                }
                continue;
            }
            if !cleaned && !unstarted {
                continue;
            }
            for service_name in pending_module.service_names.iter() {
                if let Some(entry) = services.get_mut(service_name) {
                    if entry.lifecycle == LifecycleState::Running
                        || entry.lifecycle == LifecycleState::Initializing
                    {
                        if let Some(instance) = entry.reset_after_failed_activation() {
                            retired.push((
                                pending_module.module_name.clone(),
                                service_name.to_string(),
                                instance,
                            ));
                        }
                        changed = true;
                    }
                }
            }
        }
        drop(services);
        if changed {
            self.notify_service_resolution_changed();
        }
        retire_service_objects(retired, "activate")
    }

    fn cleanup_built_batch_modules(
        &self,
        graph: &FrozenModuleGraph,
        built_modules: &[BatchModuleActivation],
        deadline: Instant,
        reactivation_services_prepared: bool,
    ) -> BatchCleanupOutcome {
        // Retain every built module in active shutdown order before any cleanup
        // callback can run. Admission was fenced for all pending entries above,
        // including lazy entries resolved from a build callback.
        for pending_module in built_modules {
            self.mark_activation_rollback_stopping(&pending_module.module_name);
        }

        let mut cleaned_modules = Vec::new();
        let mut cleanup_completed_modules = Vec::new();
        let mut failures = Vec::new();
        for pending_module in built_modules.iter().rev() {
            let live_dependents =
                match self.batch_live_module_dependents(graph, &pending_module.module_name) {
                    Ok(dependents) => dependents,
                    Err(error) => {
                        failures.push((pending_module.module_name.clone(), error));
                        continue;
                    }
                };
            if !live_dependents.is_empty() {
                failures.push((
                    pending_module.module_name.clone(),
                    CoreError::ModuleUnloadBlocked {
                        module: pending_module.module_name.clone(),
                        dependents: live_dependents,
                    },
                ));
                continue;
            }
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let remaining = deadline.saturating_duration_since(Instant::now());
                self.wait_for_service_calls_to_drain(
                    &pending_module.module_name,
                    pending_module.service_names.as_ref(),
                    Some(remaining),
                )?;
                self.cleanup_module_until(&pending_module.module_name, deadline)
            }))
            .unwrap_or_else(|_| {
                Err(CoreError::ModuleLifecycleCallbackPanicked {
                    module: pending_module.module_name.clone(),
                    command: ModuleLifecycleCommand::Activate.as_str(),
                })
            });
            match outcome {
                Ok(()) => {
                    cleanup_completed_modules.push(pending_module.module_name.clone());
                    let reset_result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            self.reset_batch_services(
                                std::slice::from_ref(pending_module),
                                reactivation_services_prepared,
                                1,
                                std::slice::from_ref(&pending_module.module_name),
                            )
                            .map_or(Ok(()), Err)
                        }));
                    let reset_error = match reset_result {
                        Ok(Ok(())) => None,
                        Ok(Err(error)) => Some(error),
                        Err(_) => Some(CoreError::ModuleLifecycleCallbackPanicked {
                            module: pending_module.module_name.clone(),
                            command: ModuleLifecycleCommand::Activate.as_str(),
                        }),
                    };
                    if reset_error.is_none() {
                        self.reset_batch_modules(
                            std::slice::from_ref(pending_module),
                            1,
                            std::slice::from_ref(&pending_module.module_name),
                            true,
                        );
                        cleaned_modules.push(pending_module.module_name.clone());
                    } else if let Some(error) = reset_error {
                        failures.push((pending_module.module_name.clone(), error));
                    }
                }
                Err(error) => failures.push((pending_module.module_name.clone(), error)),
            }
        }
        BatchCleanupOutcome {
            cleaned_modules,
            cleanup_completed_modules,
            failures,
        }
    }

    fn batch_live_module_dependents(
        &self,
        graph: &FrozenModuleGraph,
        module_name: &str,
    ) -> Result<Vec<String>, CoreError> {
        let module_dependents = graph.module_dependent_closure(module_name)?;
        let modules = self.lock_modules();
        Ok(module_dependents
            .iter()
            .filter(|dependent| {
                modules.get(dependent.as_str()).is_some_and(|entry| {
                    matches!(
                        entry.lifecycle,
                        LifecycleState::Initializing
                            | LifecycleState::Running
                            | LifecycleState::Stopping
                    )
                })
            })
            .cloned()
            .collect())
    }

    fn close_batch_service_admission(&self, pending_modules: &[BatchModuleActivation]) {
        let mut services = self.lock_services();
        for pending_module in pending_modules {
            for service_name in pending_module.service_names.iter() {
                if let Some(entry) = services.get_mut(service_name) {
                    entry.close_admission();
                }
            }
        }
        drop(services);
        self.notify_service_resolution_changed();
    }

    fn reset_batch_modules(
        &self,
        pending_modules: &[BatchModuleActivation],
        built_module_count: usize,
        cleaned_modules: &[String],
        restore_unstarted: bool,
    ) {
        let mut active_module_order = self.lock_active_module_order();
        let mut modules = self.lock_modules();
        for (index, pending_module) in pending_modules.iter().enumerate() {
            if let Some(entry) = modules.get_mut(&pending_module.module_name) {
                let cleaned = cleaned_modules
                    .iter()
                    .any(|module| module == &pending_module.module_name);
                let not_started = index >= built_module_count;
                if cleaned
                    || (restore_unstarted
                        && not_started
                        && entry.lifecycle == LifecycleState::Initializing)
                {
                    entry.lifecycle = pending_module.previous_lifecycle;
                    entry.cleanup_completed = false;
                    if cleaned {
                        active_module_order.retain(|active| active != &pending_module.module_name);
                    }
                }
            }
        }
    }
}
