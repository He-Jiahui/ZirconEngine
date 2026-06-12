use super::super::support::{option_keys_from_plugin_toml, STATIC_SOUND_PLUGIN_MANIFEST};

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
