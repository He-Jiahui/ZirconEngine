use crate::plugin::runtime_plugin::{
    descriptor::validate_runtime_plugin_descriptor, RuntimePlugin,
};
use crate::plugin::RuntimeExtensionRegistry;

use super::{
    package_contributions::register_package_manifest_contributions,
    validation::validate_runtime_plugin_package_manifest, RuntimePluginRegistrationReport,
};

impl RuntimePluginRegistrationReport {
    pub fn from_plugin(plugin: &dyn RuntimePlugin) -> Self {
        let mut extensions = RuntimeExtensionRegistry::default();
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_descriptor(plugin, &mut diagnostics);
        if let Err(error) = plugin.register_runtime_extensions(&mut extensions) {
            diagnostics.push(error.to_string());
        }
        let package_manifest = plugin.package_manifest();
        validate_runtime_plugin_package_manifest(
            Some(plugin.descriptor()),
            &package_manifest,
            &mut diagnostics,
        );
        register_package_manifest_contributions(
            &package_manifest,
            &mut extensions,
            &mut diagnostics,
        );
        Self {
            package_manifest,
            project_selection: plugin.project_selection(),
            extensions,
            diagnostics,
        }
    }
}
