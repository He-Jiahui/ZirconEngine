use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::config_store::ConfigStore;
use crate::core::diagnostics::DiagnosticStore;
use crate::core::event_bus::EventBus;
use crate::core::frame_clock::FrameClock;
use crate::core::job_scheduler::JobScheduler;
use crate::core::state::StateRegistry;
use crate::core::tasks::TaskPools;
use crate::core::time::RuntimeTimeClocks;
use crate::plugin::SceneRuntimeHookRegistration;

use super::{ModuleEntry, ServiceEntry};

pub(crate) struct CoreRuntimeInner {
    pub(crate) modules: Mutex<HashMap<String, ModuleEntry>>,
    pub(crate) services: Mutex<HashMap<String, ServiceEntry>>,
    pub(crate) event_bus: EventBus,
    pub(crate) config_store: ConfigStore,
    pub(crate) scheduler: JobScheduler,
    pub(crate) task_pools: TaskPools,
    pub(crate) frame_clock: Mutex<FrameClock>,
    pub(crate) time: Mutex<RuntimeTimeClocks>,
    pub(crate) diagnostics: Mutex<DiagnosticStore>,
    pub(crate) states: Mutex<StateRegistry>,
    pub(crate) scene_hooks: Mutex<Vec<SceneRuntimeHookRegistration>>,
}
