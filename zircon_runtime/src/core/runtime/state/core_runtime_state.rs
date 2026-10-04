use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::sync::Barrier;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;
use std::time::Instant;

use crate::core::diagnostics::{DiagnosticStore, RuntimeDevtoolsPluginCatalogEntry};
use crate::core::{CoreError, RuntimeModuleLifecycleObserver};

use super::super::config_store::ConfigStore;
use super::super::descriptors::{FrozenModuleGraph, RegistryName};
use super::super::events::EventBus;
use super::super::frame_clock::FrameClock;
use super::super::random::RandomService;
use super::super::state_machine::StateRegistry;
use super::super::tasks::{EngineTaskGraph, JobScheduler};
use super::super::time::RuntimeTimeAuthority;
use super::{ModuleEntry, ServiceEntry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModuleLifecycleCommand {
    Activate,
    Deactivate,
}

impl ModuleLifecycleCommand {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Activate => "activate",
            Self::Deactivate => "deactivate",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ModuleLifecycleTransitionToken {
    module_name: String,
    epoch: u64,
    command: ModuleLifecycleCommand,
}

// 同一模块生命周期命令的取得结果：拥有执行权、复用完成值，或等待当前执行者。
#[derive(Clone, Debug)]
pub(crate) enum ModuleLifecycleTransitionPermit {
    Owner(ModuleLifecycleTransitionToken),
    Completed(Result<(), CoreError>),
    Wait,
}

#[derive(Debug)]
pub(crate) enum ModuleLifecycleTransitionAdmission {
    Owner(ModuleLifecycleTransitionToken),
    Wait(ModuleLifecycleTransitionWait),
}

impl ModuleLifecycleTransitionAdmission {
    pub(crate) fn resolve(self) -> ModuleLifecycleTransitionPermit {
        self.resolve_until(None)
            .expect("an unbounded lifecycle transition wait cannot time out")
    }

    pub(crate) fn resolve_until(
        self,
        deadline: Option<Instant>,
    ) -> Result<ModuleLifecycleTransitionPermit, ()> {
        match self {
            Self::Owner(token) => Ok(ModuleLifecycleTransitionPermit::Owner(token)),
            Self::Wait(wait) => {
                let result = wait.completion.wait_until(deadline).ok_or(())?;
                if wait.joined_command {
                    Ok(ModuleLifecycleTransitionPermit::Completed(result))
                } else {
                    // An opposite command waits for this owner, then competes for fresh admission.
                    Ok(ModuleLifecycleTransitionPermit::Wait)
                }
            }
        }
    }
}

/// 等待者绑定加入时的完成凭据，醒来后读取该轮结果而不重新争抢全局完成值。
#[derive(Clone, Debug)]
pub(crate) struct ModuleLifecycleTransitionWait {
    completion: Arc<ModuleLifecycleCompletion>,
    joined_command: bool,
}

#[derive(Debug, Default)]
struct ModuleLifecycleCompletion {
    result: Mutex<Option<Result<(), CoreError>>>,
    changed: Condvar,
    #[cfg(test)]
    spurious_wakes: AtomicUsize,
    #[cfg(test)]
    parked_waiters: AtomicUsize,
}

impl ModuleLifecycleCompletion {
    fn wait(&self) -> Result<(), CoreError> {
        self.wait_until(None)
            .expect("an unbounded lifecycle completion wait cannot time out")
    }

    fn wait_until(&self, deadline: Option<Instant>) -> Option<Result<(), CoreError>> {
        let mut result = self
            .result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            if let Some(result) = result.as_ref() {
                return Some(result.clone());
            }
            #[cfg(test)]
            self.parked_waiters.fetch_add(1, Ordering::SeqCst);
            let timed_out = if let Some(deadline) = deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    #[cfg(test)]
                    self.parked_waiters.fetch_sub(1, Ordering::SeqCst);
                    return None;
                }
                let (next_result, wait_result) = self
                    .changed
                    .wait_timeout(result, remaining)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                result = next_result;
                wait_result.timed_out()
            } else {
                result = self
                    .changed
                    .wait(result)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                false
            };
            #[cfg(test)]
            self.parked_waiters.fetch_sub(1, Ordering::SeqCst);
            #[cfg(test)]
            if result.is_none() {
                self.spurious_wakes.fetch_add(1, Ordering::SeqCst);
            }
            if timed_out && result.is_none() {
                return None;
            }
        }
    }

    fn complete(&self, result: Result<(), CoreError>) {
        *self
            .result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(result);
        self.changed.notify_all();
    }
}

#[derive(Clone, Debug)]
enum ModuleLifecycleTransition {
    InFlight {
        epoch: u64,
        command: ModuleLifecycleCommand,
        owner: ThreadId,
        completion: Arc<ModuleLifecycleCompletion>,
        #[cfg(test)]
        waiters: usize,
        #[cfg(test)]
        opposite_waiters: usize,
    },
}

/// Serializes public module lifecycle commands while callbacks run without registry locks.
#[derive(Debug)]
pub(crate) struct LifecycleCoordinator {
    next_epoch: u64,
    transitions: HashMap<String, ModuleLifecycleTransition>,
}

impl Default for LifecycleCoordinator {
    fn default() -> Self {
        Self {
            next_epoch: 1,
            transitions: HashMap::new(),
        }
    }
}

impl LifecycleCoordinator {
    // 在调用回调前决定执行者；等待者由 CoreHandle 在锁外通过 Condvar 重试。
    pub(crate) fn begin(
        &mut self,
        module_name: &str,
        command: ModuleLifecycleCommand,
        owner: ThreadId,
    ) -> Result<ModuleLifecycleTransitionAdmission, CoreError> {
        if let Some(transition) = self.transitions.get_mut(module_name) {
            match transition {
                ModuleLifecycleTransition::InFlight {
                    owner: active_owner,
                    ..
                } if *active_owner == owner => {
                    return Err(CoreError::ModuleLifecycleCommandReentrant {
                        module: module_name.to_owned(),
                        command: command.as_str(),
                    });
                }
                ModuleLifecycleTransition::InFlight {
                    command: active_command,
                    completion,
                    #[cfg(test)]
                    waiters,
                    #[cfg(test)]
                    opposite_waiters,
                    ..
                } => {
                    let joined_command = *active_command == command;
                    #[cfg(test)]
                    {
                        if joined_command {
                            *waiters += 1;
                        } else {
                            *opposite_waiters += 1;
                        }
                    }
                    // Historical audit text is retained; the caller now holds its exact receipt.
                    // BUG: [CR-RUNTIME-LIFECYCLE-0001] 完成值按计数而非等待者身份分发；迟到调用可抢走结果，原等待者重试后再次成为 Owner。
                    // 当前实现已消解此历史问题：等待者保留加入时的 completion Arc 与 joined_command，醒后读取该轮回执；
                    // 异命令等待者则在原操作完成后重新申请执行权。对应并发回归见 tests/activation/behavior/activation/contention.rs 的
                    // lifecycle_joiner_keeps_its_result_after_unrelated_and_spurious_wakes、
                    // opposite_lifecycle_command_waits_for_owner_then_runs_its_own_operation 和
                    // late_lifecycle_retry_cannot_consume_a_previous_joiners_receipt。
                    return Ok(ModuleLifecycleTransitionAdmission::Wait(
                        ModuleLifecycleTransitionWait {
                            completion: Arc::clone(completion),
                            joined_command,
                        },
                    ));
                }
            }
        }

        let epoch = self.next_epoch;
        self.next_epoch = self
            .next_epoch
            .checked_add(1)
            .ok_or(CoreError::ModuleLifecycleEpochExhausted)?;
        self.transitions.insert(
            module_name.to_owned(),
            ModuleLifecycleTransition::InFlight {
                epoch,
                command,
                owner,
                completion: Arc::new(ModuleLifecycleCompletion::default()),
                #[cfg(test)]
                waiters: 0,
                #[cfg(test)]
                opposite_waiters: 0,
            },
        );
        Ok(ModuleLifecycleTransitionAdmission::Owner(
            ModuleLifecycleTransitionToken {
                module_name: module_name.to_owned(),
                epoch,
                command,
            },
        ))
    }

    pub(crate) fn complete(
        &mut self,
        token: &ModuleLifecycleTransitionToken,
        result: Result<(), CoreError>,
    ) {
        let completion = match self.transitions.get(&token.module_name) {
            Some(ModuleLifecycleTransition::InFlight {
                epoch,
                command,
                completion,
                ..
            }) if *epoch == token.epoch && *command == token.command => Arc::clone(completion),
            _ => return,
        };
        // Joined callers retain the completed receipt after this admission slot is removed.
        completion.complete(result);
        self.transitions.remove(&token.module_name);
    }

    #[cfg(test)]
    pub(crate) fn waiter_count(&self, module_name: &str, command: ModuleLifecycleCommand) -> usize {
        match self.transitions.get(module_name) {
            Some(ModuleLifecycleTransition::InFlight {
                command: active_command,
                waiters,
                ..
            }) if *active_command == command => *waiters,
            _ => 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn notify_transition_waiters_for_test(&self, module_name: &str) {
        if let Some(ModuleLifecycleTransition::InFlight { completion, .. }) =
            self.transitions.get(module_name)
        {
            let _result = completion
                .result
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            completion.changed.notify_all();
        }
    }

    #[cfg(test)]
    pub(crate) fn spurious_wake_count_for_test(&self, module_name: &str) -> usize {
        match self.transitions.get(module_name) {
            Some(ModuleLifecycleTransition::InFlight { completion, .. }) => {
                completion.spurious_wakes.load(Ordering::SeqCst)
            }
            _ => 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn parked_waiter_count_for_test(&self, module_name: &str) -> usize {
        match self.transitions.get(module_name) {
            Some(ModuleLifecycleTransition::InFlight { completion, .. }) => {
                completion.parked_waiters.load(Ordering::SeqCst)
            }
            _ => 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn opposite_waiter_count_for_test(
        &self,
        module_name: &str,
        active_command: ModuleLifecycleCommand,
    ) -> usize {
        match self.transitions.get(module_name) {
            Some(ModuleLifecycleTransition::InFlight {
                command,
                opposite_waiters,
                ..
            }) if *command == active_command => *opposite_waiters,
            _ => 0,
        }
    }
}

// CoreHandle 共享的运行时权威状态；注册表和生命周期门禁在这里协调。
// 工厂、模块回调与状态钩子应在相关锁释放后运行，避免重入死锁。
pub(crate) struct CoreRuntimeInner {
    pub(crate) modules: Mutex<HashMap<String, ModuleEntry>>,
    pub(crate) services: Mutex<HashMap<RegistryName, ServiceEntry>>,
    pub(crate) frozen_module_graph: Mutex<Option<Arc<FrozenModuleGraph>>>,
    pub(crate) active_module_order: Mutex<Vec<String>>,
    pub(crate) lifecycle_coordinator: Mutex<LifecycleCoordinator>,
    pub(crate) service_resolution_changed: Condvar,
    pub(crate) service_call_changed: Condvar,
    pub(crate) service_resolution_waits: Mutex<HashMap<ThreadId, ThreadId>>,
    pub(crate) service_activation_reentries: Mutex<HashSet<(ThreadId, RegistryName)>>,
    #[cfg(test)]
    pub(crate) service_resolution_claim_barrier: Mutex<Option<(usize, Arc<Barrier>)>>,
    pub(crate) event_bus: EventBus,
    pub(crate) config_store: ConfigStore,
    pub(crate) scheduler: JobScheduler,
    pub(crate) task_graph: EngineTaskGraph,
    pub(crate) frame_clock: Mutex<FrameClock>,
    random_service: RandomService,
    pub(crate) time: Mutex<RuntimeTimeAuthority>,
    pub(crate) diagnostics: Mutex<DiagnosticStore>,
    pub(crate) states: Mutex<StateRegistry>,
    pub(crate) devtools_plugin_catalog_entries: Mutex<Vec<RuntimeDevtoolsPluginCatalogEntry>>,
    pub(crate) runtime_module_lifecycle_observer:
        Mutex<Option<Arc<dyn RuntimeModuleLifecycleObserver>>>,
}

impl CoreRuntimeInner {
    pub(crate) fn new(
        frame_clock: FrameClock,
        random_service: RandomService,
        task_graph: EngineTaskGraph,
    ) -> Self {
        Self {
            modules: Default::default(),
            services: Default::default(),
            frozen_module_graph: Default::default(),
            active_module_order: Default::default(),
            lifecycle_coordinator: Default::default(),
            service_resolution_changed: Default::default(),
            service_call_changed: Default::default(),
            service_resolution_waits: Default::default(),
            service_activation_reentries: Default::default(),
            #[cfg(test)]
            service_resolution_claim_barrier: Default::default(),
            event_bus: EventBus::default(),
            config_store: ConfigStore::default(),
            scheduler: task_graph.scheduler(crate::core::TaskPoolKind::Compute),
            task_graph,
            frame_clock: Mutex::new(frame_clock),
            random_service,
            time: Default::default(),
            diagnostics: Default::default(),
            states: Default::default(),
            devtools_plugin_catalog_entries: Default::default(),
            runtime_module_lifecycle_observer: Default::default(),
        }
    }

    pub(crate) const fn random_service(&self) -> &RandomService {
        &self.random_service
    }
}
