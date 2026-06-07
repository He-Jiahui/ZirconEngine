pub(super) fn event_catalog_version_from_plugin_toml(value: &str) -> u32 {
    value
        .parse::<u32>()
        .expect("sound event catalog version should be an integer")
}
