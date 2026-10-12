use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};

// 时间选择只需要当前修订通过校验后读取二级索引；不要求封存或生成规范文本。
pub(in crate::scene::dynamic_scene::session) fn latest_updated_slot_id(
    archive: &RuntimeSessionArchive,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    Ok(archive
        .indexed_latest_slot()
        .map(|slot| slot.slot_id.clone()))
}

pub(in crate::scene::dynamic_scene::session) fn oldest_updated_slot_id(
    archive: &RuntimeSessionArchive,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    Ok(archive
        .indexed_oldest_slot()
        .map(|slot| slot.slot_id.clone()))
}
