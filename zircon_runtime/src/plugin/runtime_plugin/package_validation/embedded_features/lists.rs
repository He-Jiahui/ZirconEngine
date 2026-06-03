mod feature_extension;
mod optional;
mod state;

use crate::plugin::PluginPackageManifest;

use self::{
    feature_extension::validate_runtime_plugin_package_feature_extension_list,
    optional::validate_runtime_plugin_package_optional_feature_list,
    state::new_runtime_plugin_package_embedded_feature_provider_state,
};

pub(super) fn validate_runtime_plugin_package_feature_lists(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen_feature_providers = new_runtime_plugin_package_embedded_feature_provider_state();
    validate_runtime_plugin_package_optional_feature_list(
        package_manifest,
        &mut seen_feature_providers,
        diagnostics,
    );
    validate_runtime_plugin_package_feature_extension_list(
        package_manifest,
        &mut seen_feature_providers,
        diagnostics,
    );
}
