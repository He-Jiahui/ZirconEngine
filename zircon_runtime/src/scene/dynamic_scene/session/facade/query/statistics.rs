use super::super::super::query as session_query;
use super::super::super::*;

impl RuntimeSessionArchive {
    /// 取得与封存字节同代际的聚合统计；可供保存前预览，不重新捕获 World。
    pub fn statistics(
        &self,
    ) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
        session_query::statistics(self)
    }
}
