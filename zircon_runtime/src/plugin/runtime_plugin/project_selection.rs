use crate::{
    plugin::ExportPackagingStrategy, plugin::PluginModuleKind, plugin::ProjectPluginFeatureSelection,
    plugin::ProjectPluginSelection,
};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn project_selection(&self) -> ProjectPluginSelection {
        let mut selection = ProjectPluginSelection::runtime_plugin(
            self.runtime_id,
            self.enabled_by_default,
            self.required_by_default,
        )
        .with_packaging(self.project_selection_packaging())
        .with_runtime_crate(self.crate_name.clone())
        .with_target_modes(self.target_modes.iter().copied());
        for feature in &self.optional_features {
            selection = selection.with_feature(project_feature_selection(feature));
        }
        selection
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

fn project_feature_selection(
    feature: &crate::plugin::PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut selection =
        ProjectPluginFeatureSelection::new(feature.id.clone()).enabled(feature.enabled_by_default);
    selection.packaging = feature
        .default_packaging
        .iter()
        .copied()
        .find(|packaging| *packaging == ExportPackagingStrategy::LibraryEmbed)
        .or_else(|| feature.default_packaging.first().copied())
        .unwrap_or(ExportPackagingStrategy::LibraryEmbed);
    for module in &feature.modules {
        match module.kind {
            PluginModuleKind::Runtime if selection.runtime_crate.is_none() => {
                selection.runtime_crate = Some(module.crate_name.clone());
            }
            PluginModuleKind::Editor if selection.editor_crate.is_none() => {
                selection.editor_crate = Some(module.crate_name.clone());
            }
            _ => {}
        }
    }
    selection
}
