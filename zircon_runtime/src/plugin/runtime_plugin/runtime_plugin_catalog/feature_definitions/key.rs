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
mod tests {
    use super::feature_definition_key;

    #[test]
    fn exact_feature_definition_key_preserves_both_identities() {
        assert_eq!(
            feature_definition_key("sound.timeline", "sound_core"),
            "sound.timeline@sound_core"
        );
        assert_eq!(feature_definition_key("", ""), "@");
        assert_eq!(feature_definition_key("feature", ""), "feature@");
        assert_eq!(feature_definition_key("", "provider"), "@provider");
    }
}
