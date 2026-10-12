use super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector};
use super::preview::preview_single_slot_archive;

// 克隆已有槽位形成独立单槽档案；保留身份、场景和元数据，重新建立档案代际及发布谱系。
pub(in crate::scene::dynamic_scene::session) fn single_slot_archive(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let report = preview_single_slot_archive(archive, slot_id)?;
    RuntimeSessionArchive::from_slots(vec![archive.require_slot(&report.source_slot_id)?.clone()])
}

// 选择器只在源档案解析，复用命名导出；新档案不共享源档案的发布身份，源槽位不会移除。
pub(in crate::scene::dynamic_scene::session) fn selected_single_slot_archive(
    archive: &RuntimeSessionArchive,
    selector: RuntimeSessionSlotSelector,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let report = archive.select_slot(selector)?;
    single_slot_archive(archive, &report.selected_slot_id)
}
