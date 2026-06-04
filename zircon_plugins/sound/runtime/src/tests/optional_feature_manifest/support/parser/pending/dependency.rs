use super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super) fn push_optional_feature_dependency(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    plugin_id: &mut Option<String>,
    capability: &mut Option<String>,
    primary: &mut Option<bool>,
) {
    let Some(plugin_id) = plugin_id.take() else {
        return;
    };
    feature
        .as_mut()
        .expect("optional feature dependency should have a parent feature")
        .dependencies
        .push((
            plugin_id,
            capability
                .take()
                .expect("optional feature dependency should declare capability"),
            primary
                .take()
                .expect("optional feature dependency should declare primary"),
        ));
}
