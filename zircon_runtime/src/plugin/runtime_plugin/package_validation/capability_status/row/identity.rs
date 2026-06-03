use crate::plugin::CapabilityStatusManifest;

use super::super::identity::validate_runtime_plugin_package_capability_status_identity;

pub(super) fn validate_runtime_plugin_package_capability_status_row_identity<'a>(
    status: &'a CapabilityStatusManifest,
    owned_capabilities: &[&str],
    seen_capabilities: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_identity(
        status.capability.as_str(),
        owned_capabilities,
        seen_capabilities,
        diagnostics,
    );
}
