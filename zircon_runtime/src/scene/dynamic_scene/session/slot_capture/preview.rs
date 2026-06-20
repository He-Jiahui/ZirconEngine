use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot,
    RuntimeSessionSlotCapturePreviewReport,
};

pub(in crate::scene::dynamic_scene::session) struct RuntimeSessionSlotCapturePreview {
    pub(in crate::scene::dynamic_scene::session) report: RuntimeSessionSlotCapturePreviewReport,
    pub(in crate::scene::dynamic_scene::session) slot: RuntimeSessionSlot,
}

pub(in crate::scene::dynamic_scene::session::slot_capture) fn capture_preview(
    archive: &RuntimeSessionArchive,
    slot: RuntimeSessionSlot,
) -> Result<RuntimeSessionSlotCapturePreview, RuntimeSessionArchiveError> {
    slot.scene.ensure_supported()?;
    let report = RuntimeSessionSlotCapturePreviewReport {
        slot_id: slot.slot_id.clone(),
        will_replace_existing: archive.contains_slot(&slot.slot_id),
        metadata: slot.metadata.clone().normalized(),
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    };
    Ok(RuntimeSessionSlotCapturePreview { report, slot })
}
