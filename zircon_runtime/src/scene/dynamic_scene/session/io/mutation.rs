use std::path::Path;

use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};
use super::{load_from_path, save_to_path_atomically};

pub(in crate::scene::dynamic_scene::session) fn mutate_archive_at_path_atomically(
    path: impl AsRef<Path>,
    mutate: impl FnOnce(&mut RuntimeSessionArchive) -> Result<(), RuntimeSessionArchiveError>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_with_report_atomically(path, |archive| {
        mutate(archive)?;
        archive.manifest()
    })
}

// BUG: [CR-DYNAMIC-SESSION-FACADE-0001] 两个独立调用先后读取同一旧档案后分别修改，后提交者可覆盖先提交者的槽位；证据：本函数读取发生于 atomic::reserve_archive_path_write 之前，独立加载得到不同 lineage，保存票据只保护提交阶段。
// 路径级事务调用方复用此入口；成功返回表示该次替换已发布，不表示读改写期间未有其他提交。
pub(in crate::scene::dynamic_scene::session) fn mutate_archive_at_path_with_report_atomically<T>(
    path: impl AsRef<Path>,
    mutate: impl FnOnce(&mut RuntimeSessionArchive) -> Result<T, RuntimeSessionArchiveError>,
) -> Result<T, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let mut archive = load_from_path(path)?;
    let report = mutate(&mut archive)?;
    save_to_path_atomically(&archive, path)?;
    Ok(report)
}
