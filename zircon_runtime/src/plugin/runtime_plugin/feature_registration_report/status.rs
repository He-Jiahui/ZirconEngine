use super::RuntimePluginFeatureRegistrationReport;

impl RuntimePluginFeatureRegistrationReport {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
