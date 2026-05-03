use crate::core::ModuleDescriptor;
use crate::{
    plugin::ExportPackagingStrategy, plugin::PluginFeatureBundleManifest, plugin::PluginModuleKind,
    plugin::ProjectPluginFeatureSelection, plugin::RuntimeExtensionRegistry,
};

use super::RuntimePluginFeature;

#[derive(Clone, Debug)]
pub struct RuntimePluginFeatureRegistrationReport {
    pub manifest: crate::plugin::PluginFeatureBundleManifest,
    pub provider_package_id: Option<String>,
    pub project_selection: ProjectPluginFeatureSelection,
    pub extensions: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
}

impl RuntimePluginFeatureRegistrationReport {
    pub fn from_feature(feature: &dyn RuntimePluginFeature) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        if let Err(error) = feature.register_runtime_extensions(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        let manifest = feature.manifest();
        Self {
            project_selection: project_selection_from_feature_manifest(&manifest),
            provider_package_id: None,
            manifest,
            extensions,
            diagnostics,
        }
    }

    pub fn from_native_feature_manifest(
        manifest: PluginFeatureBundleManifest,
        provider_package_id: Option<String>,
    ) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        for module in manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
        {
            if let Err(error) = extensions.register_module(ModuleDescriptor::new(
                module.name.clone(),
                format!(
                    "Native dynamic runtime plugin feature module provided by {}",
                    manifest.id
                ),
            )) {
                diagnostics.push(error.to_string());
            }
        }
        let mut project_selection = project_selection_from_feature_manifest(&manifest);
        project_selection.provider_package_id = provider_package_id.clone();
        Self {
            project_selection,
            provider_package_id,
            manifest,
            extensions,
            diagnostics,
        }
    }

    pub fn provider_package_id_or_owner(&self) -> &str {
        self.provider_package_id
            .as_deref()
            .unwrap_or(self.manifest.owner_plugin_id.as_str())
    }

    pub fn with_provider_package_id(mut self, package_id: impl Into<String>) -> Self {
        let package_id = package_id.into();
        self.project_selection.provider_package_id = Some(package_id.clone());
        self.provider_package_id = Some(package_id);
        self
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub(crate) fn project_selection_from_feature_manifest(
    feature: &crate::plugin::PluginFeatureBundleManifest,
) -> ProjectPluginFeatureSelection {
    let mut target_modes = Vec::new();
    for target_mode in feature
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    let packaging = if feature
        .default_packaging
        .contains(&ExportPackagingStrategy::LibraryEmbed)
    {
        ExportPackagingStrategy::LibraryEmbed
    } else {
        feature
            .default_packaging
            .first()
            .copied()
            .unwrap_or(ExportPackagingStrategy::LibraryEmbed)
    };
    let mut selection = ProjectPluginFeatureSelection::new(feature.id.clone())
        .enabled(feature.enabled_by_default)
        .with_packaging(packaging)
        .with_target_modes(target_modes);
    if let Some(crate_name) = feature
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .map(|module| module.crate_name.clone())
    {
        selection = selection.with_runtime_crate(crate_name);
    }
    selection.with_optional_editor_crate(
        feature
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .map(|module| module.crate_name.clone()),
    )
}

trait OptionalEditorCrate {
    fn with_optional_editor_crate(self, crate_name: Option<String>) -> Self;
}

impl OptionalEditorCrate for ProjectPluginFeatureSelection {
    fn with_optional_editor_crate(self, crate_name: Option<String>) -> Self {
        match crate_name {
            Some(crate_name) => self.with_editor_crate(crate_name),
            None => self,
        }
    }
}
