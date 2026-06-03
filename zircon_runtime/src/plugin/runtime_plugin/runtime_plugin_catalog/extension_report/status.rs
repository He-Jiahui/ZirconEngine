use super::RuntimeExtensionCatalogReport;

impl RuntimeExtensionCatalogReport {
    pub fn is_success(&self) -> bool {
        self.fatal_diagnostics.is_empty()
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.fatal_diagnostics.is_empty()
    }
}
