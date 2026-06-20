use super::super::super::query as session_query;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn statistics(
        &self,
    ) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
        session_query::statistics(self)
    }
}
