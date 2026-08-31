use crate::core::framework::platform::RuntimeTargetMode;

use super::ProjectPluginSelection;

fn default_runtime_crate_name(plugin_id: &str) -> String {
    const PREFIX: &str = "zircon_plugin_";
    const SUFFIX: &str = "_runtime";

    let mut name = String::with_capacity(PREFIX.len() + plugin_id.len() + SUFFIX.len());
    name.push_str(PREFIX);
    for character in plugin_id.chars() {
        name.push(if character == '-' { '_' } else { character });
    }
    name.push_str(SUFFIX);
    name
}

impl ProjectPluginSelection {
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
            .unwrap_or_else(|| default_runtime_crate_name(&self.id))
    }
}

#[cfg(test)]
#[path = "project_plugin_selection_access/single_buffer_runtime_crate_name_tests.rs"]
mod single_buffer_runtime_crate_name_tests;
