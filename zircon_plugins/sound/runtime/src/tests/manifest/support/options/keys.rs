pub(super) fn option_keys_from_plugin_toml(manifest: &str) -> Vec<String> {
    super::parser::option_manifests_from_plugin_toml(manifest)
        .into_iter()
        .map(|option| option.key)
        .collect()
}
