use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot,
    RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
};

pub(in crate::scene::dynamic_scene::session) fn empty() -> RuntimeSessionArchive {
    let archive =
        RuntimeSessionArchive::from_payload(RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION, Vec::new());
    archive.record_normalized();
    archive
}

// 验证票据必须对应规范化后的修订；元数据与索引更新完成后，才能缓存验证成功状态。
pub(in crate::scene::dynamic_scene::session) fn from_slots(
    slots: Vec<RuntimeSessionSlot>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    let mut archive =
        RuntimeSessionArchive::from_payload(RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION, slots);
    archive.normalize_slot_metadata();
    archive.record_normalized();
    archive.ensure_supported()?;
    archive.record_validated();
    Ok(archive)
}
