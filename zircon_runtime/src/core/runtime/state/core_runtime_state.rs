use std::collections::{HashMap, HashSet};
#[cfg(test)]
use std::sync::Barrier;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;

use crate::core::diagnostics::{
    DiagnosticStore, RuntimeDevtoolsPluginCatalogEntry, RuntimeDevtoolsSceneHookSnapshot,
};
use crate::core::framework::state::StateRegistry;
use crate::core::RuntimeModuleLifecycleObserver;

use super::super::config_store::ConfigStore;
use super::super::descriptors::RegistryName;
use super::super::events::EventBus;
use super::super::frame_clock::FrameClock;
use super::super::tasks::{JobScheduler, TaskPools};
use super::super::time::RuntimeTimeClocks;
use super::{ModuleEntry, ServiceEntry};

pub(crate) struct CoreRuntimeInner {
    pub(crate) modules: Mutex<HashMap<String, ModuleEntry>>,
    pub(crate) services: Mutex<HashMap<RegistryName, ServiceEntry>>,
    pub(crate) service_resolution_changed: Condvar,
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
    pub(crate) scene_hook_snapshots: Mutex<Vec<RuntimeDevtoolsSceneHookSnapshot>>,
    pub(crate) devtools_plugin_catalog_entries: Mutex<Vec<RuntimeDevtoolsPluginCatalogEntry>>,
    pub(crate) runtime_module_lifecycle_observer:
        Mutex<Option<Arc<dyn RuntimeModuleLifecycleObserver>>>,
}
