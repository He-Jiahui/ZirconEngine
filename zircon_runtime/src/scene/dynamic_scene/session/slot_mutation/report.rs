use super::super::{RuntimeSessionSlot, RuntimeSessionSlotMutationPreviewReport};

pub(super) fn slot_mutation_report(
    slot: &RuntimeSessionSlot,
    destination_slot_id: Option<String>,
) -> RuntimeSessionSlotMutationPreviewReport {
    RuntimeSessionSlotMutationPreviewReport {
        source_slot_id: slot.slot_id.clone(),
        destination_slot_id,
        metadata: slot.metadata.clone().normalized(),
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    }
}
