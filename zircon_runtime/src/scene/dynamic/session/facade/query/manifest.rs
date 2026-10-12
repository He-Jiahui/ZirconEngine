use super::super::super::query as session_query;
use super::super::super::*;

impl RuntimeSessionArchive {
    /// 从已封存代际取得轻量目录；若当前档案无法通过验证或字节上限，查询也会失败。
    pub fn manifest(&self) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        session_query::manifest(self)
    }
}
