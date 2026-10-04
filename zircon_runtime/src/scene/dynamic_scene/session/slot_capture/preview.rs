use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot,
    RuntimeSessionSlotCapturePreviewReport,
};

// 捕获流程内部的快照与摘要配对；提交和带裁剪规划消费同一槽位，避免报告与再次捕获的场景不一致。
pub(in crate::scene::dynamic_scene::session) struct RuntimeSessionSlotCapturePreview {
    pub(in crate::scene::dynamic_scene::session) report: RuntimeSessionSlotCapturePreviewReport,
    pub(in crate::scene::dynamic_scene::session) slot: RuntimeSessionSlot,
}

// World/Level 预检入口已校验档案并构造槽位；此边界确认场景有效，再声明是否替换已有 ID。
// 只返回内存中的快照与摘要，文件预检和发布由路径层负责。
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
