pub(in super::super) fn bool_from_plugin_toml(value: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        _ => panic!("unknown sound boolean value {value}"),
    }
}
