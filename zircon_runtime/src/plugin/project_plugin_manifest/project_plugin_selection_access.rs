use crate::{RuntimePluginId, RuntimeTargetMode};

use super::ProjectPluginSelection;

impl ProjectPluginSelection {
    pub fn runtime_id(&self) -> Option<RuntimePluginId> {
        RuntimePluginId::parse_key(&self.id)
    }

    pub fn supports_target(&self, target: RuntimeTargetMode) -> bool {
        self.target_modes.is_empty() || self.target_modes.contains(&target)
    }

    pub fn is_runtime_builtin_domain(&self) -> bool {
        self.runtime_crate
            .as_deref()
            .is_some_and(|crate_name| crate_name.starts_with("builtin_"))
    }

    pub fn runtime_crate_name(&self) -> String {
        self.runtime_crate
            .clone()
            .unwrap_or_else(|| format!("zircon_plugin_{}_runtime", self.id.replace('-', "_")))
    }
}
