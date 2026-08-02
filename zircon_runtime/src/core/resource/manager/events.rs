use crate::core::resource::{ResourceEvent, ResourceId, RuntimeResourceState};

use super::{resource_manager::ResourceManager, runtime_slot::ResourceRuntimeSlot};

impl ResourceManager {
    pub(super) fn broadcast(&self, event: ResourceEvent) {
        self.publish_event(event);
    }

    pub(super) fn ensure_runtime_slot(&self, id: ResourceId) {
        let has_payload = self.get_untyped(id).is_some();
        let mut runtime = self.lock_runtime_write();
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
        let mut runtime = self.lock_runtime_write();
        let slot = runtime.entry(id).or_default();
        slot.state = RuntimeResourceState::Loaded;
    }

    pub(super) fn set_runtime_state(&self, id: ResourceId, state: RuntimeResourceState) {
        let mut runtime = self.lock_runtime_write();
        let slot = runtime.entry(id).or_default();
        slot.state = state;
    }
}
