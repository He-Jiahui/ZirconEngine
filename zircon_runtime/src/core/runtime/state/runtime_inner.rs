use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::state::StateRegistry;

use super::super::config_store::ConfigStore;
use super::super::descriptors::RegistryName;
use super::super::events::EventBus;
use super::super::frame_clock::FrameClock;
use super::super::tasks::{JobScheduler, TaskPools};
use super::super::time::RuntimeTimeClocks;
use super::{ModuleEntry, SceneRuntimeHookSet, ServiceEntry, WorldRuntimeExtensionSet};

pub(crate) struct CoreRuntimeInner {
    pub(crate) modules: Mutex<HashMap<String, ModuleEntry>>,
    pub(crate) services: Mutex<HashMap<RegistryName, ServiceEntry>>,
    pub(crate) event_bus: EventBus,
    pub(crate) config_store: ConfigStore,
    pub(crate) scheduler: JobScheduler,
    pub(crate) task_pools: TaskPools,
    pub(crate) frame_clock: Mutex<FrameClock>,
    pub(crate) time: Mutex<RuntimeTimeClocks>,
    pub(crate) diagnostics: Mutex<DiagnosticStore>,
    pub(crate) states: Mutex<StateRegistry>,
    pub(crate) scene_hooks: Mutex<SceneRuntimeHookSet>,
    pub(crate) world_extensions: Mutex<WorldRuntimeExtensionSet>,
}
