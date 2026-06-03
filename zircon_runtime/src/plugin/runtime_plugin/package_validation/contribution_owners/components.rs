mod ownership;

use crate::plugin::PluginPackageManifest;

use self::ownership::validate_runtime_plugin_package_component_owner;

pub(super) fn validate_component_owners(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    for component in &package_manifest.components {
        validate_runtime_plugin_package_component_owner(
            component.type_id.as_str(),
            component.plugin_id.as_str(),
            package_manifest.id.as_str(),
            diagnostics,
        );
    }
}
