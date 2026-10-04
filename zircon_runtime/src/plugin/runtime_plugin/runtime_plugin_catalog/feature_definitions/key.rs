pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_definition_key(
    feature_id: &str,
    provider_package_id: &str,
) -> String {
    let capacity = feature_id.len() + 1 + provider_package_id.len();
    let mut key = String::with_capacity(capacity);
    key.push_str(feature_id);
    key.push('@');
    key.push_str(provider_package_id);
    key
}

#[cfg(test)]
#[path = "tests/key.rs"]
mod tests;
