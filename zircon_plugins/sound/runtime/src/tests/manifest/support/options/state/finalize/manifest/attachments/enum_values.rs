pub(super) fn apply_enum_values(
    option: zircon_runtime::plugin::PluginOptionManifest,
    enum_values: Vec<String>,
) -> zircon_runtime::plugin::PluginOptionManifest {
    if enum_values.is_empty() {
        option
    } else {
        option.with_enum_values(enum_values)
    }
}
