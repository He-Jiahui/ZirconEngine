use crate::plugin::PluginPackageManifest;

use super::super::super::{
    package_validation::{
        validate_runtime_plugin_package_field, validate_runtime_plugin_package_id,
    },
    RuntimePluginDescriptor,
};

pub(super) fn validate_runtime_plugin_registration_package_identity(
    descriptor: Option<&RuntimePluginDescriptor>,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_id(
        "runtime plugin package manifest",
        "id",
        &package_manifest.id,
        diagnostics,
    );
    if let Some(descriptor) = descriptor {
        if package_manifest.id != descriptor.package_id {
            diagnostics.push(format!(
                "runtime plugin package manifest id `{}` must match descriptor package_id `{}`",
                package_manifest.id, descriptor.package_id
            ));
        }
    }
    validate_runtime_plugin_package_field(
        "display_name",
        &package_manifest.display_name,
        diagnostics,
    );
}
