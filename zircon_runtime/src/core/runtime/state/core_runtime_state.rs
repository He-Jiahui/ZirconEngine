use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::sync::Barrier;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;

use crate::core::diagnostics::{DiagnosticStore, RuntimeDevtoolsPluginCatalogEntry};
use crate::core::framework::state::StateRegistry;
use crate::core::{CoreError, RuntimeModuleLifecycleObserver};

use super::super::config_store::ConfigStore;
use super::super::descriptors::{FrozenModuleGraph, RegistryName};
use super::super::events::EventBus;
use super::super::frame_clock::FrameClock;
use super::super::tasks::{JobScheduler, TaskPools};
use super::super::time::RuntimeTimeClocks;
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

#[derive(Clone, Debug)]
pub(crate) enum ModuleLifecycleTransitionPermit {
    Owner(ModuleLifecycleTransitionToken),
    Completed(Result<(), CoreError>),
    Wait,
}

#[derive(Clone, Debug)]
enum ModuleLifecycleTransition {
    InFlight {
        epoch: u64,
        command: ModuleLifecycleCommand,
        owner: ThreadId,
        waiters: usize,
    },
    Completed {
        command: ModuleLifecycleCommand,
        result: Result<(), CoreError>,
        waiters: usize,
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
    pub(crate) fn begin(
        &mut self,
        module_name: &str,
        command: ModuleLifecycleCommand,
        owner: ThreadId,
    ) -> Result<ModuleLifecycleTransitionPermit, CoreError> {
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
                    waiters,
                    ..
                } => {
                    if *active_command == command {
                        *waiters += 1;
                    }
                    return Ok(ModuleLifecycleTransitionPermit::Wait);
                }
                ModuleLifecycleTransition::Completed {
                    command: completed_command,
                    result,
                    waiters,
                } if *completed_command == command && *waiters > 0 => {
                    *waiters -= 1;
                    return Ok(ModuleLifecycleTransitionPermit::Completed(result.clone()));
                }
                ModuleLifecycleTransition::Completed { .. } => {}
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
                waiters: 0,
            },
        );
        Ok(ModuleLifecycleTransitionPermit::Owner(
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
        let waiter_count = match self.transitions.get(&token.module_name) {
            Some(ModuleLifecycleTransition::InFlight {
                epoch,
                command,
                waiters,
                ..
            }) if *epoch == token.epoch && *command == token.command => *waiters,
            _ => return,
        };
        self.transitions.insert(
            token.module_name.clone(),
            ModuleLifecycleTransition::Completed {
                command: token.command,
                result,
                waiters: waiter_count,
            },
        );
    }
}

pub(crate) struct CoreRuntimeInner {
    pub(crate) modules: Mutex<HashMap<String, ModuleEntry>>,
    pub(crate) services: Mutex<HashMap<RegistryName, ServiceEntry>>,
    pub(crate) frozen_module_graph: Mutex<Option<Arc<FrozenModuleGraph>>>,
    pub(crate) lifecycle_coordinator: Mutex<LifecycleCoordinator>,
    pub(crate) lifecycle_transition_changed: Condvar,
    pub(crate) service_resolution_changed: Condvar,
    pub(crate) service_call_changed: Condvar,
    pub(crate) service_resolution_waits: Mutex<HashMap<ThreadId, ThreadId>>,
    pub(crate) service_activation_reentries: Mutex<HashSet<(ThreadId, RegistryName)>>,
    #[cfg(test)]
    pub(crate) service_resolution_claim_barrier: Mutex<Option<(usize, Arc<Barrier>)>>,
    pub(crate) event_bus: EventBus,
    pub(crate) config_store: ConfigStore,
    pub(crate) scheduler: JobScheduler,
    pub(crate) task_pools: TaskPools,
    pub(crate) frame_clock: Mutex<FrameClock>,
    pub(crate) time: Mutex<RuntimeTimeClocks>,
    pub(crate) diagnostics: Mutex<DiagnosticStore>,
    pub(crate) states: Mutex<StateRegistry>,
    pub(crate) devtools_plugin_catalog_entries: Mutex<Vec<RuntimeDevtoolsPluginCatalogEntry>>,
    pub(crate) runtime_module_lifecycle_observer:
        Mutex<Option<Arc<dyn RuntimeModuleLifecycleObserver>>>,
}
