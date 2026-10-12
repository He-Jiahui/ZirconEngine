use super::super::super::validation;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn ensure_supported(&self) -> Result<(), RuntimeSessionArchiveError> {
        if self.has_current_validation_ticket() {
            return Ok(());
        }

        let _validation = self
            .state
            .validation_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.has_current_validation_ticket() {
            return Ok(());
        }

        validation::ensure_supported(self)?;
        self.record_validated();
        Ok(())
    }
}
