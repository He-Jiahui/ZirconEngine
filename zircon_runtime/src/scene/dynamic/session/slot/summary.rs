use super::super::RuntimeSessionSlotSummary;
use super::RuntimeSessionSlot;

impl RuntimeSessionSlot {
    pub fn summary(&self) -> RuntimeSessionSlotSummary {
        RuntimeSessionSlotSummary {
            slot_id: self.slot_id.clone(),
            metadata: self.metadata.clone(),
            scene_format_version: self.scene.payload_header.schema_version,
            entity_count: self.scene.entities.len(),
            resource_count: self.scene.resources.len(),
        }
    }
}
