use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::config_store::ConfigStore;
use crate::core::event_bus::EventBus;
use crate::core::job_scheduler::JobScheduler;

use super::{ModuleEntry, ServiceEntry};

pub(crate) struct CoreRuntimeInner {
    pub(crate) modules: Mutex<HashMap<String, ModuleEntry>>,
    pub(crate) services: Mutex<HashMap<String, ServiceEntry>>,
    pub(crate) event_bus: EventBus,
    pub(crate) config_store: ConfigStore,
    pub(crate) scheduler: JobScheduler,
}
