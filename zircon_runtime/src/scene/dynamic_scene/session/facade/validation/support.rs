use super::super::super::validation;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn ensure_supported(&self) -> Result<(), RuntimeSessionArchiveError> {
        validation::ensure_supported(self)
    }
}
