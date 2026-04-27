use crate::ProjectPluginSelection;

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn project_selection(&self) -> ProjectPluginSelection {
        ProjectPluginSelection::runtime_plugin(
            self.runtime_id,
            self.enabled_by_default,
            self.required_by_default,
        )
        .with_runtime_crate(self.crate_name.clone())
        .with_target_modes(self.target_modes.iter().copied())
    }
}
