use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super::super) fn push_optional_feature_dependency(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    let Some(dependency) =
        super::signature::take_optional_feature_dependency(plugin_id, capability, primary)
    else {
        return;
    };
    super::append::append_optional_feature_dependency(feature, dependency);
}
