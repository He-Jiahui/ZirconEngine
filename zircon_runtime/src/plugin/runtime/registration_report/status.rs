use super::RuntimePluginRegistrationReport;

impl RuntimePluginRegistrationReport {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
