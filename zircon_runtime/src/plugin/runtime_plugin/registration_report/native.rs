mod runtime_modules;

use crate::plugin::{PluginPackageManifest, RuntimeExtensionRegistry};

use self::runtime_modules::register_native_package_runtime_modules;
use super::{
    native_package_projection::native_project_selection_from_package,
    package_contributions::register_package_manifest_contributions,
    validation::validate_runtime_plugin_package_manifest, RuntimePluginRegistrationReport,
};

impl RuntimePluginRegistrationReport {
    pub fn from_native_package_manifest(package_manifest: PluginPackageManifest) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        drop(validate_runtime_plugin_package_manifest(
            None,
            &package_manifest,
            &mut diagnostics,
        ));
        register_native_package_runtime_modules(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        Self {
            project_selection: native_project_selection_from_package(&package_manifest),
            package_manifest,
            extensions,
            diagnostics,
        }
    }
}
