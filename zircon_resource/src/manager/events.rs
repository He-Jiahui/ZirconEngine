use crate::{ResourceEvent, ResourceId, RuntimeResourceState};

use super::{resource_manager::ResourceManager, runtime_slot::ResourceRuntimeSlot};

impl ResourceManager {
    pub(super) fn broadcast(&self, event: ResourceEvent) {
        let mut subscribers = self
            .subscribers
            .lock()
            .expect("resource subscribers lock poisoned");
        subscribers.retain(|sender| sender.send(event.clone()).is_ok());
    }

    pub(super) fn ensure_runtime_slot(&self, id: ResourceId) {
        let has_payload = self.get_untyped(id).is_some();
        let mut runtime = self
            .runtime
            .write()
            .expect("resource runtime lock poisoned");
        runtime.entry(id).or_insert_with(|| ResourceRuntimeSlot {
            ref_count: 0,
            state: if has_payload {
                RuntimeResourceState::Loaded
            } else {
                RuntimeResourceState::Unloaded
            },
        });
    }

    pub(super) fn mark_runtime_loaded(&self, id: ResourceId) {
        let mut runtime = self
            .runtime
            .write()
            .expect("resource runtime lock poisoned");
        let slot = runtime.entry(id).or_default();
        slot.state = RuntimeResourceState::Loaded;
    }

    pub(super) fn set_runtime_state(&self, id: ResourceId, state: RuntimeResourceState) {
        let mut runtime = self
            .runtime
            .write()
            .expect("resource runtime lock poisoned");
        let slot = runtime.entry(id).or_default();
        slot.state = state;
    }
}
