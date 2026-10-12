use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveStatistics,
};

// 统计与规范输出共用当前修订的封存结果，因此读取统计也可能触发验证、序列化或返回封存失败。
pub(in crate::scene::dynamic_scene::session) fn statistics(
    archive: &RuntimeSessionArchive,
) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
    Ok(archive.sealed_artifact()?.statistics().clone())
}
