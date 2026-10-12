use crate::core::framework::project::ProjectPluginFeatureSelection;
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageRole, RuntimeExtensionRegistry};

#[path = "registration/feature.rs"]
mod feature;
#[path = "registration/native.rs"]
mod native;
#[path = "registration/project_selection.rs"]
mod project_selection;
#[path = "registration/provider.rs"]
mod provider;
#[path = "registration/status.rs"]
mod status;

pub(super) use project_selection::project_selection_from_feature_manifest;

#[derive(Clone, Debug)]
pub struct RuntimePluginFeatureRegistrationReport {
    pub manifest: PluginFeatureBundleManifest,
    pub provider_package_id: Option<String>,
    pub provider_package_role: PluginPackageRole,
    pub project_selection: ProjectPluginFeatureSelection,
    pub extensions: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
}
