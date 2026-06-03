mod identity;
mod note;
mod references;
mod targets;

use crate::plugin::{CapabilityStatusManifest, PluginPackageManifest};

use self::{
    identity::validate_runtime_plugin_package_capability_status_row_identity,
    note::validate_runtime_plugin_package_capability_status_row_note,
    references::validate_runtime_plugin_package_capability_status_row_bevy_references,
    targets::validate_runtime_plugin_package_capability_status_row_targets,
};

pub(super) fn validate_runtime_plugin_package_capability_status_row<'a>(
    package_manifest: &PluginPackageManifest,
    status: &'a CapabilityStatusManifest,
    owned_capabilities: &[&str],
    seen_capabilities: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_row_identity(
        status,
        owned_capabilities,
        seen_capabilities,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_row_targets(
        package_manifest,
        status,
        diagnostics,
    );
    validate_runtime_plugin_package_capability_status_row_bevy_references(status, diagnostics);
    validate_runtime_plugin_package_capability_status_row_note(status, diagnostics);
}
