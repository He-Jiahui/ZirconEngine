use super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
