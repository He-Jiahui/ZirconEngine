use crate::{ExportPackagingStrategy, ProjectPluginSelection};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn project_selection(&self) -> ProjectPluginSelection {
        ProjectPluginSelection::runtime_plugin(
            self.runtime_id,
            self.enabled_by_default,
            self.required_by_default,
        )
        .with_packaging(self.project_selection_packaging())
        .with_runtime_crate(self.crate_name.clone())
        .with_target_modes(self.target_modes.iter().copied())
    }

    fn project_selection_packaging(&self) -> ExportPackagingStrategy {
        if self
            .default_packaging
            .contains(&ExportPackagingStrategy::LibraryEmbed)
        {
            return ExportPackagingStrategy::LibraryEmbed;
        }
        self.default_packaging
            .first()
            .copied()
            .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
    }
}
