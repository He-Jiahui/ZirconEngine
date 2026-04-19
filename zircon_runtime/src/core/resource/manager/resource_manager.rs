use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::core::resource::{ResourceData, ResourceEvent, ResourceId, ResourceRegistry};

use super::runtime_slot::ResourceRuntimeSlot;

#[derive(Clone, Debug, Default)]
pub struct ResourceManager {
    pub(super) registry: Arc<RwLock<ResourceRegistry>>,
    pub(super) payloads: Arc<RwLock<HashMap<ResourceId, Arc<dyn ResourceData>>>>,
    pub(super) runtime: Arc<RwLock<HashMap<ResourceId, ResourceRuntimeSlot>>>,
    pub(super) subscribers: Arc<Mutex<Vec<Sender<ResourceEvent>>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&self) -> Receiver<ResourceEvent> {
        let (sender, receiver) = unbounded();
        self.subscribers
            .lock()
            .expect("resource subscribers lock poisoned")
            .push(sender);
        receiver
    }

    pub fn registry(&self) -> std::sync::RwLockReadGuard<'_, ResourceRegistry> {
        self.registry
            .read()
            .expect("resource registry lock poisoned")
    }
}
