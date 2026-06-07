pub(in crate::tests::manifest::support) fn option_keys_from_plugin_toml(
    manifest: &str,
) -> Vec<String> {
    super::super::parser::option_manifests_from_plugin_toml(manifest)
        .into_iter()
        .map(|option| option.key)
        .collect()
}
