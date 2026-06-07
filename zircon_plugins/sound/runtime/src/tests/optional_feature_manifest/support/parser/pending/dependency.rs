mod append;
mod signature;

use super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super) fn push_optional_feature_dependency(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    let Some(dependency) =
        signature::take_optional_feature_dependency(plugin_id, capability, primary)
    else {
        return;
    };
    append::append_optional_feature_dependency(feature, dependency);
}
