use super::RuntimePluginFeatureBlock;

impl RuntimePluginFeatureBlock {
    pub fn to_diagnostic(&self) -> String {
        let severity = if self.required {
            "required feature"
        } else {
            "optional feature"
        };
        let mut details = Vec::new();
        if self.unknown_feature {
            details.push("feature is not declared by the plugin catalog".to_string());
        }
        if self.invalid_owner_dependency {
            details.push(
                "owner dependency is missing, not marked primary, or not the only primary dependency"
                    .to_string(),
            );
        }
        if self.provider_missing {
            details.push("concrete runtime feature provider registration is missing".to_string());
        }
        if self.target_unsupported {
            details.push("target mode is not supported".to_string());
        }
        if !self.missing_plugins.is_empty() {
            details.push(format!(
                "missing plugins: {}",
                self.missing_plugins.join(", ")
            ));
        }
        if !self.missing_capabilities.is_empty() {
            details.push(format!(
                "missing capabilities: {}",
                self.missing_capabilities.join(", ")
            ));
        }
        if self.cycle {
            details.push("feature capability dependencies form a cycle".to_string());
        }
        if details.is_empty() {
            details.push("dependency status is unresolved".to_string());
        }
        format!(
            "{severity} {} is blocked: {}",
            self.feature_id,
            details.join("; ")
        )
    }
}
