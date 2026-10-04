use std::path::Path;

use super::super::{
    target_path, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotSelector,
};

// 预览先验证整个源档案，再报告实际槽位的规范元数据和负载数量；成功不代表已生成独立导出档案。
pub(in crate::scene::dynamic_scene::session) fn preview_single_slot_archive(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;

    let slot = archive
        .slot(slot_id)
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: slot_id.to_string(),
        })?;

    Ok(RuntimeSessionSlotExportPreviewReport {
        source_slot_id: slot.slot_id.clone(),
        target_path: None,
        will_replace_target: false,
        metadata: slot.metadata.clone().normalized(),
        entity_count: slot.scene.entities.len(),
        resource_count: slot.scene.resources.len(),
    })
}

pub(in crate::scene::dynamic_scene::session) fn preview_selected_single_slot_archive(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    preview_single_slot_archive(archive, &report.selected_slot_id)
}

// 目标状态只是当次文件系统观察；预览不创建目录或预留写入，保存仍须重新检查并处理错误。
pub(in crate::scene::dynamic_scene::session) fn preview_single_slot_archive_to_path(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionSlotExportPreviewReport, RuntimeSessionArchiveError> {
    let target_path = target_path.as_ref();
    let mut report = preview_single_slot_archive(archive, slot_id)?;
    report.will_replace_target = target_path::target_file_will_replace(
        target_path,
        "runtime session single-slot archive target",
    )?;
    report.target_path = Some(target_path.to_path_buf());
    Ok(report)
}
