use super::super::super::query as session_query;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn manifest(&self) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        session_query::manifest(self)
    }
}
