use super::missing::RuntimeRequiredPluginMissing;
use super::report::RuntimeModuleLoadReport;

impl RuntimeModuleLoadReport {
    pub fn effective_required_missing(&self) -> Vec<RuntimeRequiredPluginMissing> {
        let mut missing = self.owned_required_missing();
        for entry in &self.runtime_plugin_availability.missing_required {
            let structured_missing = RuntimeRequiredPluginMissing {
                id: entry.runtime_id,
                reason: entry.reason.clone(),
            };
            if !missing
                .iter()
                .any(|existing| existing.id == structured_missing.id)
            {
                missing.push(structured_missing);
            }
        }
        missing
    }

    pub fn required_missing_summary(&self) -> String {
        self.effective_required_missing()
            .into_iter()
            .map(|missing| {
                format!(
                    "required runtime plugin {} is unavailable: {}",
                    missing.id.label(),
                    missing.reason
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn effective_errors(&self) -> Vec<String> {
        let mut errors = self.errors.clone();
        for missing in self.effective_required_missing() {
            let diagnostic = format!(
                "required runtime plugin {} is unavailable: {}",
                missing.id.label(),
                missing.reason
            );
            if !errors.iter().any(|existing| existing == &diagnostic) {
                errors.push(diagnostic);
            }
        }
        errors
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.effective_errors().is_empty()
    }
}
