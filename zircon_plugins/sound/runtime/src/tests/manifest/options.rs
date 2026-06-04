use super::support::{
    option_keys_from_plugin_toml, option_manifest_tuple, option_manifests_from_plugin_toml,
    STATIC_SOUND_PLUGIN_MANIFEST,
};

#[test]
fn static_plugin_manifest_keeps_runtime_option_keys_in_sync() {
    let mut static_keys = option_keys_from_plugin_toml(STATIC_SOUND_PLUGIN_MANIFEST);
    let runtime_options = crate::sound_options();
    let mut runtime_keys = runtime_options
        .iter()
        .map(|option| option.key.clone())
        .collect::<Vec<_>>();
    static_keys.sort_unstable();
    runtime_keys.sort_unstable();

    assert_eq!(static_keys, runtime_keys);
}

#[test]
fn static_plugin_manifest_keeps_runtime_option_metadata_in_sync() {
    let mut static_options = option_manifests_from_plugin_toml(STATIC_SOUND_PLUGIN_MANIFEST)
        .into_iter()
        .map(option_manifest_tuple)
        .collect::<Vec<_>>();
    let mut runtime_options = crate::sound_options()
        .into_iter()
        .map(option_manifest_tuple)
        .collect::<Vec<_>>();
    static_options.sort_unstable_by_key(|option| option.0.clone());
    runtime_options.sort_unstable_by_key(|option| option.0.clone());

    assert_eq!(static_options, runtime_options);
}
