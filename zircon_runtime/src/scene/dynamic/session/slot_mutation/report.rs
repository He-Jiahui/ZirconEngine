use super::super::{RuntimeSessionSlot, RuntimeSessionSlotMutationPreviewReport};

// 为改名和删除预览投影已有槽位；有目标 ID 表示改名，删除则没有目标 ID，场景仅报告规模。
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
