use super::super::RuntimeSessionSlotSummary;
use super::RuntimeSessionSlot;

impl RuntimeSessionSlot {
    pub fn summary(&self) -> RuntimeSessionSlotSummary {
        RuntimeSessionSlotSummary {
            slot_id: self.slot_id.clone(),
            metadata: self.metadata.clone().normalized(),
            scene_format_version: self.scene.format_version,
            entity_count: self.scene.entities.len(),
            resource_count: self.scene.resources.len(),
        }
    }
}
