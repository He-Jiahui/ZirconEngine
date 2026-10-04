use std::path::Path;

use super::super::super::super::{
    io, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};

pub(in crate::scene::dynamic_scene::session) fn save_single_slot_archive_to_path_atomically(
    archive: &RuntimeSessionArchive,
    slot_id: &str,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let target_path = target_path.as_ref();
    let report = super::preview_save_single_slot_archive_to_path(archive, slot_id, target_path)?;
    // 预览确定的实际槽 ID 用于生成独立归档；返回目录取自该单槽归档，而非原多槽归档。
    let exported_archive = archive.single_slot_archive(&report.source_slot_id)?;
    io::save_to_path_atomically(&exported_archive, target_path)?;
    exported_archive.manifest()
}
