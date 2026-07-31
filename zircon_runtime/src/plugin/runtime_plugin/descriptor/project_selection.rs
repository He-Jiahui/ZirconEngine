mod features;
mod packaging;

use crate::core::framework::project::ProjectPluginSelection;

use self::{
    features::project_feature_selection, packaging::descriptor_project_selection_packaging,
};
use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn project_selection(&self) -> ProjectPluginSelection {
        let mut selection = ProjectPluginSelection::runtime_plugin(
            self.runtime_id.clone(),
            self.enabled_by_default,
            self.required_by_default,
        )
        .with_packaging(descriptor_project_selection_packaging(self))
        .with_runtime_crate(self.crate_name.clone())
        .with_target_modes(self.target_modes.iter().copied());
        for feature in &self.optional_features {
            selection = selection.with_feature(project_feature_selection(feature));
        }
        selection
    }
}
