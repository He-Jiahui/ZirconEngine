pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_definition_key(
    feature_id: &str,
    provider_package_id: &str,
) -> String {
    format!("{feature_id}@{provider_package_id}")
}
