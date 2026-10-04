use std::panic::{self, AssertUnwindSafe};
use std::time::Instant;

use crate::core::{CoreError, LifecycleState};

use super::super::descriptors::FrozenModuleGraph;
use super::super::state::{ModuleLifecycleCommand, ModuleLifecycleTransitionPermit};
use super::CoreHandle;

mod batch;
mod blocked_dependencies;
mod blocked_unload;
mod module_lifecycle;
mod service_lifecycle;
mod startup;
mod unload_mutation;

use self::batch::transaction_set::LifecycleTransactionSet;
use self::blocked_unload::first_blocked_unload;
use self::service_lifecycle::retire_service_objects;
use self::unload_mutation::unload_services;

fn acquire_activation_transition_reservation(
    handle: &CoreHandle,
    module_name: &str,
    deadline: Option<Instant>,
    allow_expired_owner: bool,
) -> Result<ModuleLifecycleTransitionPermit, CoreError> {
    loop {
        match handle.acquire_module_lifecycle_transition_until_mode(
            module_name,
            ModuleLifecycleCommand::Activate,
            deadline,
            allow_expired_owner,
        )? {
            // A joined success ends the older operation; reacquire a fresh owner
            // permit so the complete closure is reserved before callbacks run.
            ModuleLifecycleTransitionPermit::Completed(Ok(())) => continue,
            permit => return Ok(permit),
        }
    }
}

impl CoreHandle {
    /// 以默认的零等待预算激活请求模块及其传递依赖；每个模块仍会先检查一次就绪状态。
    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.activate_module_with_ready_deadline(module_name, Default::default(), None, None)
    }

    /// 按冻结模块图的依赖优先顺序激活请求模块及其传递依赖。
    /// `ready_timeout` 为每个模块单独计时；零预算仍会先调用一次 `ready`。
    pub fn activate_module_with_ready_timeout(
        &self,
        module_name: &str,
        ready_timeout: std::time::Duration,
    ) -> Result<(), CoreError> {
        let ready_deadline = Instant::now()
            .checked_add(ready_timeout)
            .unwrap_or_else(Instant::now);
        // Zero remains an immediate readiness probe; its owner may run while a
        // competing waiter is rejected immediately. Positive budgets bound
        // admission before callbacks can begin.
        let transition_deadline = Some(ready_deadline);
        self.activate_module_with_ready_deadline(
            module_name,
            ready_timeout,
            Some(ready_deadline),
            transition_deadline,
        )
    }

    fn activate_module_with_ready_deadline(
        &self,
        module_name: &str,
        ready_timeout: std::time::Duration,
        ready_deadline: Option<Instant>,
        transition_deadline: Option<Instant>,
    ) -> Result<(), CoreError> {
        crate::profile_scope!("runtime", "core", "activate_module");
        // One absolute budget covers graph freeze, transition admission, and readiness.
        let graph = self.frozen_module_graph()?;
        let activation_closure = graph.module_activation_closure(module_name)?;
        let mut transitions =
            LifecycleTransactionSet::new(self, activation_closure.len(), module_name);
        let mut owner_modules = Vec::with_capacity(activation_closure.len());

        // Reserve the stable dependency closure before invoking any module callback.
        for module_name in &activation_closure {
            let permit = match acquire_activation_transition_reservation(
                self,
                module_name,
                transition_deadline,
                ready_timeout.is_zero(),
            ) {
                Ok(permit) => permit,
                Err(error) => {
                    return transitions.finish(Err(error));
                }
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
                ModuleLifecycleTransitionPermit::Completed(Ok(())) => {
                    unreachable!("activation reservation retries joined successful receipts")
                }
                ModuleLifecycleTransitionPermit::Completed(Err(error)) => {
                    return transitions.finish(Err(error));
                }
                ModuleLifecycleTransitionPermit::Wait => {
                    let error = CoreError::ModuleLifecycleCoordinatorUnresolved {
                        module: module_name.clone(),
                        command: ModuleLifecycleCommand::Activate.as_str(),
                    };

                    return transitions.finish(Err(error));
                }
            }
        }

        if let Err(error) = self.validate_module_activation_states(&activation_closure) {
            return transitions.finish(Err(error));
        }

        let mut result = Ok(());
        for module_name in owner_modules {
            let module_result =
                self.activate_module_once(&graph, &module_name, ready_timeout, ready_deadline);
            transitions.complete_module(&module_name, module_result.clone());
            if let Err(error) = module_result {
                result = Err(error);
                break;
            }
        }
        transitions.finish(result)
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
        ready_deadline: Option<Instant>,
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
            entry.cleanup_completed = false;
            let service_names = module_services.service_names().clone();
            let startup_services = module_services.startup_service_names().clone();
            (previous_lifecycle, service_names, startup_services)
        };

        // 发布 Initializing 后释放模块表锁，再调用生命周期和服务工厂，避免回访内核时自锁。
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

            match ready_deadline {
                Some(deadline) => {
                    self.wait_until_module_ready_until(module_name, deadline, ready_timeout)?
                }
                None => self.wait_until_module_ready(module_name, ready_timeout)?,
            }
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
            if built {
                // Rollback is a stopping transaction too: fence new calls and
                // retain the module in shutdown order until cleanup succeeds.
                self.close_service_admission(module_services.shutdown_service_names().as_ref());
                self.mark_activation_rollback_stopping(module_name);
            }
            let rollback_error = built
                .then(|| {
                    panic::catch_unwind(AssertUnwindSafe(|| {
                        self.wait_for_service_calls_to_drain(
                            module_name,
                            module_services.shutdown_service_names().as_ref(),
                            Some(std::time::Duration::ZERO),
                        )?;
                        self.cleanup_module(module_name)?;
                        self.mark_module_cleanup_completed(module_name);
                        Ok(())
                    }))
                    .unwrap_or_else(|_| {
                        Err(CoreError::ModuleLifecycleCallbackPanicked {
                            module: module_name.to_owned(),
                            command: ModuleLifecycleCommand::Activate.as_str(),
                        })
                    })
                })
                .transpose()
                .err();
            // 清理失败时保留 Stopping 和关闭的调用入口供关闭流程重试；完整清理后才恢复前态。
            let rollback_error = if let Some(rollback_error) = rollback_error {
                Some(rollback_error)
            } else {
                let reset_result = if reactivation_services_prepared {
                    self.rollback_module_services_after_failed_reactivation(
                        module_name,
                        service_names.as_ref(),
                    )
                } else {
                    self.reset_started_services(module_name, service_names.as_ref())
                };
                match reset_result {
                    Some(error) => Some(error),
                    None => {
                        self.reset_initializing_module(module_name, previous_lifecycle);
                        None
                    }
                }
            };
            return Err(CoreError::module_activation_failed(
                activation_error,
                rollback_error,
            ));
        }

        Ok(())
    }

    /// 不设服务调用排空预算停用模块；仍有运行依赖模块或服务依赖时，首次预检会拒绝卸载。
    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.deactivate_module_with_drain_timeout_inner(module_name, None)
    }

    /// 让观察者、受保护调用排空和模块 `cleanup_until` 共用一个截止时刻，不在阶段间重置预算。
    /// 零预算仍执行即时排空探测；已处于 `Stopping` 的模块可再次调用以继续清理。
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
        let deadline = match drain_timeout {
            Some(timeout) => Some(Instant::now().checked_add(timeout).ok_or_else(|| {
                CoreError::ModuleCleanupTimeout {
                    module: module_name.to_owned(),
                    operation: "module_deactivation".to_owned(),
                    budget: timeout,
                    incomplete_entries: 1,
                    failed: 0,
                    cancelled: 0,
                }
            })?),
            None => None,
        };
        let transition_deadline = match drain_timeout {
            Some(timeout) if !timeout.is_zero() => deadline,
            _ => None,
        };
        self.deactivate_module_with_deadline(
            module_name,
            drain_timeout,
            deadline,
            transition_deadline,
        )
    }

    // 整机关闭传入共享绝对 deadline，避免逐模块重新起算清理预算。
    pub(crate) fn deactivate_module_until(
        &self,
        module_name: &str,
        deadline: Instant,
    ) -> Result<(), CoreError> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(CoreError::ModuleCleanupTimeout {
                module: module_name.to_owned(),
                operation: "module_deactivation".to_owned(),
                budget: std::time::Duration::ZERO,
                incomplete_entries: 1,
                failed: 0,
                cancelled: 0,
            });
        }
        self.deactivate_module_with_deadline(
            module_name,
            Some(remaining),
            Some(deadline),
            Some(deadline),
        )
    }

    fn deactivate_module_with_deadline(
        &self,
        module_name: &str,
        drain_timeout: Option<std::time::Duration>,
        deadline: Option<Instant>,
        transition_deadline: Option<Instant>,
    ) -> Result<(), CoreError> {
        let graph = self.frozen_module_graph()?;
        self.run_module_lifecycle_transition_until(
            module_name,
            ModuleLifecycleCommand::Deactivate,
            transition_deadline,
            || self.deactivate_module_with_graph(&graph, module_name, drain_timeout, deadline),
        )
    }

    pub(super) fn deactivate_module_with_graph(
        &self,
        graph: &FrozenModuleGraph,
        module_name: &str,
        drain_timeout: Option<std::time::Duration>,
        deadline: Option<Instant>,
    ) -> Result<(), CoreError> {
        let module_services = graph.module_services(module_name)?;
        let unload_order = module_services.shutdown_service_names().clone();
        let module_dependents = graph.module_dependent_closure(module_name)?;
        let (lifecycle, live_dependents) = {
            let modules = self.lock_modules();
            let Some(entry) = modules.get(module_name) else {
                return Err(CoreError::MissingModule(module_name.to_string()));
            };
            let live_dependents: Vec<String> = module_dependents
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
                .collect();
            (entry.lifecycle, live_dependents)
        };
        // Stopping 表示已选定 fail-closed 清理；重试继续排空和清理，不再重跑可撤销预检。
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
        // A failed dependent in Stopping still owns state that may call its
        // provider. Recheck this barrier on retries as well as the first stop.
        if !live_dependents.is_empty() {
            return Err(CoreError::ModuleUnloadBlocked {
                module: module_name.to_owned(),
                dependents: live_dependents,
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
                self.notify_runtime_module_deactivating(module_name, deadline)?;
                check_deactivation_deadline(module_name, drain_timeout, deadline)?;

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

            let remaining = deadline
                .map(|deadline| deadline.saturating_duration_since(Instant::now()))
                .or(drain_timeout);
            self.wait_for_service_calls_to_drain(module_name, unload_order, remaining)?;
            check_deactivation_deadline(module_name, drain_timeout, deadline)?;

            if !self.module_cleanup_completed(module_name) {
                if let Some(deadline) = deadline {
                    self.cleanup_module_until(module_name, deadline)?;
                } else {
                    self.cleanup_module(module_name)?;
                }
                self.mark_module_cleanup_completed(module_name);
            }
            check_deactivation_deadline(module_name, drain_timeout, deadline)?;

            if !unload_order.is_empty() {
                let retired = {
                    let mut services = self.lock_services();
                    unload_services(module_name, &mut services, unload_order)
                };
                self.notify_service_resolution_changed();
                if let Some(error) = retire_service_objects(retired, "deactivate") {
                    return Err(error);
                }
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

    // 仅在补偿清理成功后恢复前态，并从关机清单移除已回滚的模块。
    fn reset_initializing_module(&self, module_name: &str, previous_lifecycle: LifecycleState) {
        let mut active_module_order = self.lock_active_module_order();
        let mut modules = self.lock_modules();
        if let Some(entry) = modules.get_mut(module_name) {
            if matches!(
                entry.lifecycle,
                LifecycleState::Initializing | LifecycleState::Running | LifecycleState::Stopping
            ) {
                entry.lifecycle = previous_lifecycle;
                entry.cleanup_completed = false;
                active_module_order.retain(|active_module| active_module != module_name);
            }
        }
    }

    fn mark_activation_rollback_stopping(&self, module_name: &str) {
        let mut active_module_order = self.lock_active_module_order();
        let mut modules = self.lock_modules();
        if let Some(entry) = modules.get_mut(module_name) {
            entry.lifecycle = LifecycleState::Stopping;
            if !active_module_order
                .iter()
                .any(|active_module| active_module == module_name)
            {
                active_module_order.push(module_name.to_owned());
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
        entry.cleanup_completed = false;
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
        entry.cleanup_completed = false;
        active_module_order.retain(|active_module| active_module != module_name);
        Ok(())
    }

    fn mark_module_cleanup_completed(&self, module_name: &str) {
        let mut modules = self.lock_modules();
        if let Some(entry) = modules.get_mut(module_name) {
            entry.cleanup_completed = true;
        }
    }

    fn module_cleanup_completed(&self, module_name: &str) -> bool {
        self.lock_modules()
            .get(module_name)
            .is_some_and(|entry| entry.cleanup_completed)
    }
}

fn check_deactivation_deadline(
    module_name: &str,
    timeout: Option<std::time::Duration>,
    deadline: Option<Instant>,
) -> Result<(), CoreError> {
    if let (Some(timeout), Some(deadline)) = (timeout, deadline) {
        // Zero remains an immediate service-drain probe for existing callers.
        if !timeout.is_zero() && Instant::now() >= deadline {
            return Err(CoreError::ModuleCleanupTimeout {
                module: module_name.to_owned(),
                operation: "module_deactivation".to_owned(),
                budget: timeout,
                incomplete_entries: 1,
                failed: 0,
                cancelled: 0,
            });
        }
    }
    Ok(())
}
