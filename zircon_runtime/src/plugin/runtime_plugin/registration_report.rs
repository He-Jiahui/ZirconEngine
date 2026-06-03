use crate::plugin::{PluginPackageManifest, ProjectPluginSelection, RuntimeExtensionRegistry};

mod native;
mod native_package_projection;
mod package_contributions;
mod plugin;
mod status;
mod validation;

#[derive(Clone, Debug)]
pub struct RuntimePluginRegistrationReport {
    pub package_manifest: PluginPackageManifest,
    pub project_selection: ProjectPluginSelection,
    pub extensions: RuntimeExtensionRegistry,
    pub diagnostics: Vec<String>,
}
